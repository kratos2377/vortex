use std::{collections::HashMap, net::SocketAddr, str::FromStr, sync::Arc, time::Duration};

use axum::Router;
use conf::{config_types::ServerConfiguration, configuration::Configuration};
use context::context::{ContextImpl, DynContext};
use kafka::producer;
use orion::{constants::{CREATE_USER_BET, GAME_BET_SETTLED, GAME_BET_SETTLED_ERROR, GAME_OVER_EVENT, GENERATE_GAME_BET_EVENTS, SETTLE_BET_KEY, START_GAME_SETTLE_EVENT}, events::kafka_event::{GameBetSettleKafkaPayload, GameOverEvent, GameSettleBetErrorRedisPayload, GameUserBetSettleEvent, GenerateGameBetSettleEvents}, models::game_bet_events::GameBetStatus};
use rdkafka::{consumer::StreamConsumer, error::KafkaError, message::ToBytes, producer::{FutureProducer, FutureRecord, Producer}, util::Timeout, Message};
use redis::{AsyncCommands, RedisResult, SetOptions, ToRedisArgs};
use reqwest::Client;
use sea_orm::ColumnTrait;
use sea_orm::{prelude::Expr, Condition};
use sea_orm::{Database, EntityTrait, QueryFilter, QuerySelect, Set, TransactionTrait};
use state::AppDBState;
use sea_orm::ActiveModelTrait;
use tokio::{spawn, task::JoinHandle};
use ton::models::game_bets;
use tower::ServiceBuilder;
use tower_cookies::CookieManagerLayer;
use tower_http::cors::CorsLayer;
use tracing::{error, info, warn};
use utils::get_solana_rpc_node_health;
use uuid::Uuid;
use futures_util::future;

pub mod conf;
pub mod controllers;
pub mod routes;
pub mod utils;
pub mod state;
pub mod errors;
pub mod kafka;
pub mod context;


#[tokio::main]
async fn main()-> Result<(), Box<dyn std::error::Error>>  {

    let config = conf::configuration::Configuration::load().unwrap();
    //  dotenv().ok();
  
      //Connect with database
      let connection = match Database::connect(&config.postgres_url.url).await {
          Ok(connection) => connection,
          Err(e) => panic!("{:?}",e)
      };
  
      

      let client = redis::Client::open(config.redis.url).unwrap();
      let multiplex_redis_conn = client.get_multiplexed_async_connection().await.unwrap();
      let state = AppDBState {conn: connection.clone()};


      let reqwest_client = Client::new();


      let context = ContextImpl::new_dyn_context( connection , multiplex_redis_conn , reqwest_client);

      let consumers =  kafka::consumer::init_consumers(&config.kafka).unwrap();

      let kafka_producer = kafka::producer::create_new_kafka_producer(&config.kafka).unwrap();
      
    
      let game_bet_handles = init_game_bet_events_consumer(
          context,
          consumers,
          &kafka_producer
      );
  
      start_web_server(state, &config.server, vec![game_bet_handles])
      .await;
  


  
      Ok(())

}



async fn start_web_server(
    state: AppDBState,
    config: &ServerConfiguration,
    shutdown_handles: Vec<JoinHandle<()>>,
) {

    // Start server
    let addr = SocketAddr::from(([0, 0, 0, 0], config.port));
    tracing::info!("listening on {addr}");


    let user_game_bet_routes = routes::user_game_bet_routes::create_user_game_bet_routes() ;
    let routes_all = Router::new()
                            .nest( "/api/v1/game_bets", user_game_bet_routes)
                            .layer(ServiceBuilder::new()
                                    .layer(CookieManagerLayer::new())
                                    .layer(CorsLayer::permissive()))
                            .with_state(state);


    // run it
    let listener = tokio::net::TcpListener::bind("127.0.0.1:3020")
        .await
        .unwrap();
    println!("listening on {}", listener.local_addr().unwrap());
    axum::serve(listener, routes_all).await.unwrap();

        // Shutdown tracing provider
}


pub async fn shutdown_signal(shutdown_handles: Vec<JoinHandle<()>>) {
    let ctrl_c = async {
        tokio::signal::ctrl_c()
            .await
            .expect("Initialization of Ctrl+C handler failed");
    };

    #[cfg(unix)]
    let terminate = async {
        tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
            .expect("Initialization of signal handler failed")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }

    for handle in shutdown_handles {
        handle.abort();
    }
}




fn init_game_bet_events_consumer(
    context: DynContext,
    kafka_consumers: HashMap<String, StreamConsumer>,
    producer: &FutureProducer
) -> JoinHandle<()> {

    let mut kafka_joins: Vec<JoinHandle<()>> = vec![];

    for (key_topic , value) in kafka_consumers.into_iter() {
       let kf_join =  listen(
            context.clone(),
            value,
            key_topic,
            producer
        );

        kafka_joins.push(kf_join);
    }

    let join_handle = spawn(async move {
        for handle in kafka_joins {
            handle.await.unwrap();
        }
    });

    return join_handle;
    

}


pub fn listen(
    context: DynContext,
    stream_consumer: StreamConsumer,
    key_topic: String,
    producer: &FutureProducer
) -> JoinHandle<()> {
    let topic = key_topic.clone();
    let new_producer = producer.clone();

    // Start listener
    tokio::spawn(async move {
        do_listen( context, &stream_consumer, topic, new_producer ).await;
    })
}

pub async fn do_listen(
    context: DynContext,
    stream_consumer: &StreamConsumer,
    topic_name: String,
    producer: FutureProducer
) {

    let postgres_conn = context.get_postgres_db_client();

    loop {
        match stream_consumer.recv().await {
            Err(e) => warn!("Error: {}", e),
            Ok(message) => {
 
            let topic = message.topic();
            let payload = String::from_utf8(message.payload().unwrap().to_vec()).unwrap();

            match topic {

                GENERATE_GAME_BET_EVENTS => {
                    let generate_game_bet_event_payload = serde_json::from_str(&payload);

                    if generate_game_bet_event_payload.is_err() {
                            error!("Error Occured while parsing GenerateGameBetEvent");
                    }  else {
                            let game_bet_res_model: GenerateGameBetSettleEvents = generate_game_bet_event_payload.unwrap();

                            // Get 2K records from postgres -> Update status and publish these records to hyperion topic for consumption

                        if get_solana_rpc_node_health(context.get_reqwest_client()).await {
                            info!("Node health successful. Generating game bet events");
                            let tx =     postgres_conn.begin().await.unwrap();



                            let mut game_bets = game_bets::Entity::find_by_game_id_and_session_id_with_progress(Uuid::from_str(&game_bet_res_model.game_id).unwrap(),
                             game_bet_res_model.session_id.clone() , GameBetStatus::InProgress.to_string()).limit(2000).all(&postgres_conn).await;
                      
                      
                        if game_bets.is_err() {
                            error!("Error while fetching GameBets")
                        } else {
                            let game_bets_vec = game_bets.unwrap();
                            
                            if game_bets_vec.len() > 0 {
                                let mut records_ids = vec![];
                                let mut kafka_game_events_vec = vec![];
        
                                for bet in game_bets_vec {
                                    records_ids.push(bet.id.clone());
        
                                    let kafka_game_bet_payload = GameBetSettleKafkaPayload {
                                        game_id: bet.game_id.to_string().clone(),
                                        session_id: bet.session_id.clone(),
                                        winner_id: game_bet_res_model.winner_id.clone(),
                                        user_id: bet.user_id.to_string().clone(),
                                        user_betting_on: bet.user_id_betting_on.to_string().clone(),
                                        record_id: bet.id.to_string(),
                                        user_wallet_key: decrypt_user_wallet(bet.encrypted_wallet),
                                        is_valid: game_bet_res_model.is_game_valid,
                                    };
        
                                    kafka_game_events_vec.push(kafka_game_bet_payload);
                                }
        
                                let _ = game_bets::Entity::update_many()
                                            .col_expr(game_bets::Column::Status, Expr::value(GameBetStatus::ToSettle.to_string()))
                                            .filter(
                                                Condition::all()
                                                .add(game_bets::Column::Id.is_in(records_ids))
                                            )
                                            .exec(&postgres_conn)
                                            .await;
        
                                            let _ = tx.commit().await;
                                
                                let _ = publish_game_bet_events_for_settlement(&producer , kafka_game_events_vec ).await;
                           
                                
                                let redis_payload = GameSettleBetErrorRedisPayload {
                                    game_id: game_bet_res_model.game_id.clone(),
                                    session_id: game_bet_res_model.session_id.clone(),
                                    winner_id: game_bet_res_model.winner_id.clone(),
                                    is_game_valid: game_bet_res_model.is_game_valid,
                                };
    
    
                                let mut redis_conn = context.get_redis_connection();
    
                                let opts = SetOptions::default().with_expiration(redis::SetExpiry::EX(900));
                                let redis_rsp: RedisResult<()> =  redis_conn.set_options(SETTLE_BET_KEY.to_string() + &game_bet_res_model.game_id + "_" + &game_bet_res_model.session_id, serde_json::to_string(&redis_payload).unwrap() ,opts).await;
                           
                            }
    
                        }
                        } else {
                            let redis_payload = GameSettleBetErrorRedisPayload {
                                game_id: game_bet_res_model.game_id.clone(),
                                session_id: game_bet_res_model.session_id.clone(),
                                winner_id: game_bet_res_model.winner_id.clone(),
                                is_game_valid: game_bet_res_model.is_game_valid.clone(),
                            };


                            let mut redis_conn = context.get_redis_connection();

                            let opts = SetOptions::default().with_expiration(redis::SetExpiry::EX(300));
                            let redis_rsp: RedisResult<()> =  redis_conn.set_options(SETTLE_BET_KEY.to_string() + &game_bet_res_model.game_id + "_" + &game_bet_res_model.session_id, serde_json::to_string(&redis_payload).unwrap() ,opts).await;
                        }
                  
                    }
                },

                GAME_OVER_EVENT => {

                    
                    let game_over_event_payload = serde_json::from_str(&payload);

                    if game_over_event_payload.is_err() {
                        error!("Error Occured while parsing GameOverEvent");
                    } else {
                        let game_over_event_model: GameOverEvent = game_over_event_payload.unwrap();


                        let redis_payload = GameSettleBetErrorRedisPayload {
                            game_id: game_over_event_model.game_id.clone(),
                            session_id: game_over_event_model.session_id.clone(),
                            winner_id: game_over_event_model.winner_id.clone(),
                            is_game_valid: game_over_event_model.is_game_valid.clone()
                        };

                        let mut redis_conn = context.get_redis_connection();

                        let opts = SetOptions::default().with_expiration(redis::SetExpiry::EX(300));
                        let redis_rsp: RedisResult<()> =  redis_conn.set_options(SETTLE_BET_KEY.to_string() + &game_over_event_model.game_id + "_" + &game_over_event_model.session_id, serde_json::to_string(&redis_payload).unwrap() ,opts).await;
                    }

                },

                GAME_BET_SETTLED => {

                                
                    let game_bet_settle_payload = serde_json::from_str(&payload);

                    if game_bet_settle_payload.is_err() {
                        error!("Error Occured while parsing GameUserBetSettleEvent");
                    } else {
                        let game_over_event_model: GameUserBetSettleEvent = game_bet_settle_payload.unwrap();


                        let get_game_bet_model =   game_bets::Entity::find_by_user_id_game_id_and_session_id(Uuid::from_str(&game_over_event_model.game_id).unwrap(),
                        Uuid::from_str(&game_over_event_model.user_id).unwrap(), game_over_event_model.session_id).one(&postgres_conn).await;


                        if get_game_bet_model.is_err() {
                            error!("Error while unwraping GameBetModel");
                        } else {

                            let mut game_model_res: game_bets::ActiveModel = get_game_bet_model.unwrap().unwrap().into();

                            game_model_res.status = Set(GameBetStatus::Settled.to_string());

                            let _ = game_model_res.update(&postgres_conn).await;

                        }
                            

                    }


                },

                GAME_BET_SETTLED_ERROR => {

                                          
                    let game_bet_settle_payload = serde_json::from_str(&payload);

                    if game_bet_settle_payload.is_err() {
                        error!("Error Occured while parsing GameUserBetSettleEvent");
                    } else {
                        let game_over_event_model: GameUserBetSettleEvent = game_bet_settle_payload.unwrap();


                        let get_game_bet_model =   game_bets::Entity::find_by_user_id_game_id_and_session_id(Uuid::from_str(&game_over_event_model.game_id).unwrap(),
                        Uuid::from_str(&game_over_event_model.user_id).unwrap(), game_over_event_model.session_id.clone()).one(&postgres_conn).await;


                        if get_game_bet_model.is_err() {
                            error!("Error while unwraping GameBetModel");
                        } else {

                            let mut game_model_res: game_bets::ActiveModel = get_game_bet_model.unwrap().unwrap().into();

                            game_model_res.status = Set(GameBetStatus::InProgress.to_string());

                            let _ = game_model_res.update(&postgres_conn).await;

                            let redis_payload = GameSettleBetErrorRedisPayload {
                                game_id: game_over_event_model.game_id.clone(),
                                session_id: game_over_event_model.session_id.clone(),
                                winner_id: game_over_event_model.winner_id.clone(),
                                is_game_valid: game_over_event_model.is_game_valid.clone(),
                            };

                            let mut redis_conn = context.get_redis_connection();

                            let opts = SetOptions::default().with_expiration(redis::SetExpiry::EX(300));
                            let redis_rsp: RedisResult<()> =  redis_conn.set_options(SETTLE_BET_KEY.to_string() + &game_over_event_model.game_id + "_" + &game_over_event_model.session_id, serde_json::to_string(&redis_payload).unwrap() ,opts).await;


                        }


                    }




                },
            

                _ => {
                    println!("No topics found")
                   }

            }
    
                
        }
        }
}

}


pub fn decrypt_user_wallet(encrypted_wallet_key: String) -> String {
    let decrypted = crypter::decrypt(b"walletsecretsalt", encrypted_wallet_key.to_bytes()).expect("Failed to decrypt");
    String::from_utf8(decrypted).unwrap()
}


pub async fn publish_game_bet_events_for_settlement(producer: &FutureProducer , kafka_events: Vec<GameBetSettleKafkaPayload>) -> Result<(), KafkaError> {


    producer.begin_transaction().unwrap();


    let kafka_result = future::try_join_all(kafka_events.iter().map(|event| async move {
        let converted_string_event = serde_json::to_string(event).unwrap();
        
        let delivery_result = producer
        .send(
            FutureRecord::to(START_GAME_SETTLE_EVENT)
                    .payload(&converted_string_event)
                    .key("settle_event"),
            Duration::from_secs(2),
        )
        .await;

    // This will be executed when the result is received.
  //  println!("Delivery status for message {} received", i);
    delivery_result

    })

    ).await;

    match kafka_result {
        Ok(_) => (),
        Err(e) => return Err(e.0.into()),
    }

    producer.commit_transaction(Timeout::from(Duration::from_secs(1))).unwrap(); 

    Ok(())

}