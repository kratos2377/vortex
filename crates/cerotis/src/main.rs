use std::{collections::HashMap, net::SocketAddr, str::FromStr, sync::{Arc, Mutex}};

use api::health;
use axum::{routing::get, Router};
use conf::{config_types::ServerConfiguration, configuration::Configuration};
use context::context::{ContextImpl, DynContext};
use mongodb::bson::{self, doc};
use orion::{constants::{CREATE_USER_BET, FRIEND_REQUEST_EVENT, GAME_GENERAL_EVENT, GAME_INVITE_EVENT, USER_GAME_MOVE, USER_JOINED_ROOM, USER_LEFT_ROOM, USER_ONLINE_EVENT, USER_STATUS_EVENT}, events::{kafka_event::{GameBetEvent, UserFriendRequestKafkaEvent, UserGameBetEvent, UserGameDeletetionEvent}, ws_events::UserConnectionEventPayload}, models::{chess_events::{CellPosition, ChessNormalEvent, ChessPromotionEvent}, game_bet_events::GameBetStatus, game_model::Game, user_game_event::UserGameMove, user_game_relation_model::UserGameRelation, user_score_update_event::UserScoreUpdateEvent, user_turn_model::UserTurnMapping}};
use rdkafka::{consumer::StreamConsumer, Message};
use sea_orm::{prelude::Expr, ActiveValue, ColIdx, Database, EntityTrait, QueryFilter, Set, Value};
use tokio::{spawn, task::JoinHandle};
use tracing::{info, warn};
use ton::models::{self, game_bets, users};
use chrono::Utc;
use uuid::Uuid;
use sea_orm::ActiveModelTrait;
use sea_orm::ColumnTrait;
pub mod kafka;
pub mod conf;
pub mod api;
pub mod context;
pub mod mongo_pool;
pub mod fen_update;



#[tokio::main]
async fn main()  {
    let config = conf::configuration::Configuration::load().unwrap();

    let consumers = kafka::consumer::init_consumers(&config.kafka).unwrap();
    
    
    let connection = match Database::connect(config.postgres_url.url.clone()).await {
        Ok(connection) => connection,
        Err(e) => panic!("{:?}",e)
    };
    
    let client = redis::Client::open(config.redis_url.url.clone()).unwrap();
    let redis_connection = client.get_connection().unwrap(); 
    let mongo_db_client = Arc::new(mongo_pool::init_db_client(&config.mongo_db).await.unwrap());
    
    
    let context = ContextImpl::new_dyn_context(mongo_db_client,  Arc::new(Mutex::new(redis_connection)) , connection);
    
    let user_and_game_handles = init_user_and_game_kafka_consumer(
        context,
        &config, 
        consumers
    );

    start_web_server(&config.server, vec![user_and_game_handles])
    .await;


}


async fn start_web_server(
    config: &ServerConfiguration,
    shutdown_handles: Vec<JoinHandle<()>>,
) {
    // Initialize routing
    let routing = init_routing();

    // Start server
    let addr = SocketAddr::from(([0, 0, 0, 0], config.port));
    tracing::info!("listening on {addr}");

    let listener = tokio::net::TcpListener::bind("127.0.0.1:3005")
    .await
    .unwrap();
println!("listening on {}", listener.local_addr().unwrap());
axum::serve(listener, routing.into_make_service_with_connect_info::<SocketAddr>()).with_graceful_shutdown(shutdown_signal(shutdown_handles)).await.unwrap();

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


fn init_routing() -> Router {
    let base_router = Router::new().route("/health", get(health));

    return base_router;

}


fn init_user_and_game_kafka_consumer(
    context: DynContext,
    config: &Configuration,
    kafka_consumers: HashMap<String, StreamConsumer>,
) -> JoinHandle<()> {

    let mut kafka_joins: Vec<JoinHandle<()>> = vec![];

    for (key_topic , value) in kafka_consumers.into_iter() {
       let kf_join =  listen(
            context.clone(),
            config,
            value,
            key_topic
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
    config: &Configuration,
    stream_consumer: StreamConsumer,
    key_topic: String,
) -> JoinHandle<()> {
    let topic = key_topic.clone();

    // Start listener
    tokio::spawn(async move {
        do_listen( context, &stream_consumer, topic ).await;
    })
}

pub async fn do_listen(
    context: DynContext,
    stream_consumer: &StreamConsumer,
    topic_name: String,
) {

    let mongo_db = context.get_mongo_db_client().database("user_game_events_db");
    let user_collection = mongo_db.collection::<UserGameRelation>("users");
    let game_collection = mongo_db.collection::<Game>("games");
    let user_turn_collection = mongo_db.collection::<UserTurnMapping>("user_turns");

    let postgres_conn = context.get_postgres_db_client();

    loop {
        match stream_consumer.recv().await {
            Err(e) => warn!("Error: {}", e),
            Ok(message) => {
 
            let topic = message.topic();
            let payload = String::from_utf8(message.payload().unwrap().to_vec()).unwrap();

            match topic {
                "user_game_deletion" => {
                    let user_game_deletion_event_res = serde_json::from_str(&payload);
                  if(user_game_deletion_event_res.is_ok()) {
                    let user_game_deletion_event: UserGameDeletetionEvent = user_game_deletion_event_res.unwrap();
                    let _ = user_collection.delete_one(doc! { "game_id": user_game_deletion_event.game_id.clone()}, None).await;
                    let _ = game_collection.delete_one(doc! { "id": user_game_deletion_event.game_id.clone()}, None).await;
                    let _ = user_turn_collection.delete_one(doc! { "game_id": user_game_deletion_event.game_id}, None).await;
                  }
                },
                "user_score_update" => {
                    let user_score_update_event_payload = serde_json::from_str(&payload);

                    if user_score_update_event_payload.is_ok() {
                        let user_score_model_res: UserScoreUpdateEvent = user_score_update_event_payload.unwrap();
                        let _ = users::Entity::update_many().col_expr(users::Column::Score, Expr::val(user_score_model_res.score).into())
                                .filter(users::Column::Id.eq(Uuid::from_str(&user_score_model_res.user_id).unwrap()))
                                .exec(&postgres_conn)
                                .await;
                    }

                },
                CREATE_USER_BET => {
                    //If user_id is already present we should update the existing game_bet model. 
                    // But we should check if user_id_betting on is different or not
                    // 1 user can bet on 1 user only in each session not on both
                    let user_game_bet_payload = serde_json::from_str(&payload);

                    if user_game_bet_payload.is_ok() {
                        let user_game_bet_model: UserGameBetEvent = user_game_bet_payload.unwrap();
                        if user_game_bet_model.event_type == GameBetEvent::CREATE {
                            let new_bet = game_bets::ActiveModel {
                                id: Set(Uuid::new_v4()),
                                user_id: Set(Uuid::from_str(&user_game_bet_model.user_id).unwrap()),
                                game_id: Set(Uuid::from_str(&user_game_bet_model.game_id).unwrap()),
                                user_id_betting_on: Set(Uuid::from_str(&user_game_bet_model.user_id).unwrap()),
                                session_id: Set(user_game_bet_model.session_id),
                                game_name: Set("chess".to_string()),
                                encrypted_wallet: Set(hash_user_wallet_key(&user_game_bet_model.wallet_key)),
                                bet_amount: Set(user_game_bet_model.amount.into()),
                                status: Set(GameBetStatus::InProgress.to_string()),
                                created_at: Set(Utc::now().naive_utc()),
                                updated_at: Set(Utc::now().naive_utc()),
                            };
                            let _ = new_bet.insert(&postgres_conn).await;
                        } else {
                            let update_res = game_bets::Entity::find_by_user_id_game_id_and_session_id(Uuid::from_str(&user_game_bet_model.game_id).unwrap(),
                            Uuid::from_str(&user_game_bet_model.user_id).unwrap(), user_game_bet_model.session_id).one(&postgres_conn).await;


                            if update_res.is_err() {
                                info!("Error occured while fetch GameBet model for update event type")
                            } else {
                                let updated_res_model = update_res.unwrap();
                                if updated_res_model.is_none() {
                                    info!("Fetched Game Model is none")
                                } else {
                                    let mut game_bet_update_model : game_bets::ActiveModel = updated_res_model.unwrap().into();
                                    game_bet_update_model.bet_amount  = Set(game_bet_update_model.bet_amount.unwrap() + user_game_bet_model.amount as f64);

                                    let _ = game_bet_update_model.update(&postgres_conn).await;

                                }
                            }
                        }
                    }
                },
                "user_game_events" => {
                    let user_game_event_payload: UserGameMove = serde_json::from_str(&payload).unwrap();

                    // Instead of getting current state from mongo keep it in redis or in elixir process
                    let rsp = game_collection.find_one(doc! { "id": user_game_event_payload.game_id.clone()}, None).await;
                    let rsp_clone = rsp.clone();
                    // Fix the fen update fn

                   if rsp_clone.is_ok() && rsp_clone.unwrap().is_some() {
                    let game_model = rsp.unwrap().unwrap();

                    let _rsp = if user_game_event_payload.move_type == "normal" {
                        let gm_ev: ChessNormalEvent = serde_json::from_str(&user_game_event_payload.user_move).unwrap();
            
                        let old_position: CellPosition = serde_json::from_str(&gm_ev.initial_cell).unwrap();
                        let new_position: CellPosition = serde_json::from_str(&gm_ev.target_cell).unwrap();
                        let piece: Vec<char> = gm_ev.piece.chars().collect();
                
                        let updated_fen = fen_update::update_fen_with_timing(&game_model.chess_state, *piece.get(0).unwrap() , &get_chess_position(&old_position) , &get_chess_position(&new_position) , None );

                        if updated_fen.is_some() {
                            let updated_fen_rsp = updated_fen.unwrap();
                           // println!("Updated Fen is: {:?}" , updated_fen.unwrap());
                      
                            let _ = game_collection.update_one(doc! { "id": user_game_event_payload.game_id.clone()}, doc! { "$set": doc! {"chess_state": updated_fen_rsp.fen } }, None).await;
                        
                          
                        
                        }  else {
                            println!("UPDATE RESULT IS NONE");
                        }

                    } else {
                        let gm_ev: ChessPromotionEvent = serde_json::from_str(&user_game_event_payload.user_move).unwrap();
                        let old_position: CellPosition = serde_json::from_str(&gm_ev.initial_cell).unwrap();
                        let new_position: CellPosition = serde_json::from_str(&gm_ev.target_cell).unwrap();
                        let piece: Vec<char> = gm_ev.piece.chars().collect();
                        let promoted_to: Vec<char> = gm_ev.promoted_to.chars().collect();
                        let updated_fen = fen_update::update_fen_with_timing(&game_model.chess_state, *piece.get(0).unwrap() , &get_chess_position(&old_position) , &get_chess_position(&new_position) , Some(*promoted_to.get(0).unwrap()) );

                        if updated_fen.is_some() {
                            let updated_fen_rsp = updated_fen.unwrap();
                           // println!("Updated Fen is: {:?}" , updated_fen.unwrap());
                            let _ = game_collection.update_one(doc! { "id": user_game_event_payload.game_id.clone()}, doc! { "$set": doc! {"chess_state": updated_fen_rsp.fen } }, None).await;
                        } 

                    };

                   } 
                   
                }


                _ => {
                    println!("No topics found")
                   }

            }
    
                
        }
        }
}

}


fn hash_user_wallet_key(wallet_key: &String) -> String {
    
    let hash = crypter::encrypt(b"walletsecretsalt" , wallet_key).expect("failed to encrypt");
    String::from_utf8(hash).unwrap()
}

pub fn get_chess_position(cell: &CellPosition) -> String {
   format!("{}{}" , get_file_letter(cell.x) , (8-cell.y).to_string())
} 

pub fn get_file_letter(num: i64) -> String {

    match num {
        0 => "a".to_owned(),
        1 => "b".to_owned(),
        2 => "c".to_owned(),
        3 => "d".to_owned(),
        4 => "e".to_owned(),
        5 => "f".to_owned(),
        6 => "g".to_owned(),
        7 => "h".to_owned(),
        _ => "a".to_owned()
    }

}