use std::{net::SocketAddr, time::Duration};

use axum::{response::IntoResponse, routing::get, Router};
use conf::config_types::{KafkaConfiguration, ServerConfiguration};
use context::context::ContextImpl;
use futures::{future, StreamExt};
use orion::{constants::{EXECUTOR_GAME_OVER_EVENT, EXECUTOR_GAME_STAKE_TIME_OVER_EVENT, GAME_OVER_STATUS_KEY, GAME_STAKE_TIME_OVER, GAME_STAKE_TIME_OVER_DATA, GENERATE_GAME_BET_EVENTS, SETTLE_BET_KEY, SETTLE_BET_KEY_DATA}, events::kafka_event::GenerateGameBetSettleEvents};
use rdkafka::{error::KafkaError, producer::{FutureProducer, FutureRecord, Producer}, util::Timeout};
use redis::{aio::{MultiplexedConnection, PubSub}, AsyncCommands, RedisResult};
use serde_json::json;
use tokio::task::JoinHandle;
use tower::ServiceBuilder;
use tower_cookies::CookieManagerLayer;
use tower_http::cors::CorsLayer;
use tokio::spawn;


pub mod conf;
pub mod context;
pub mod kafka;
pub mod utils;
pub mod logging_tracing;


#[tokio::main]
async fn main()-> Result<(), Box<dyn std::error::Error>>  {

    let config = conf::configuration::Configuration::load().unwrap();
    //  dotenv().ok();
    logging_tracing::init(&config)?;
    
      let mut client = redis::Client::open(config.redis.url).unwrap();
      let mut redis_conn = client.get_multiplexed_async_connection().await.unwrap();
      let mut async_pubsub_conn = client.get_async_pubsub().await.unwrap();


      let publishing_events_join_handle = init_redis_pubsub_and_produce_events(redis_conn ,  async_pubsub_conn , &config.kafka ).await;

  
      start_web_server(&config.server , vec![publishing_events_join_handle])
      .await;
  


  
      Ok(())

}



async fn start_web_server(
    config: &ServerConfiguration,
    shutdown_handles: Vec<JoinHandle<()>>
) {

    // Start server
    let addr = SocketAddr::from(([0, 0, 0, 0], config.port));
    tracing::info!("listening on {addr}");


    let routes_all = Router::new()
                            .route( "/api/v1/health", get(health))
                            .layer(ServiceBuilder::new()
                                    .layer(CookieManagerLayer::new())
                                    .layer(CorsLayer::permissive()));


    // run it
    let listener = tokio::net::TcpListener::bind("127.0.0.1:3025")
        .await
        .unwrap();
    println!("listening on {}", listener.local_addr().unwrap());
    axum::serve(listener, routes_all).with_graceful_shutdown(shutdown_signal(shutdown_handles)).await.unwrap();

        // Shutdown tracing provider
}


pub async fn health() -> impl IntoResponse {
    axum::Json(json!({ "status" : "UP" }))
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


pub async fn init_redis_pubsub_and_produce_events(redis_conn: MultiplexedConnection ,  pubsub_conn: PubSub , kafka_config: &KafkaConfiguration) -> JoinHandle<()> {

    let mut kafka_joins: Vec<JoinHandle<()>> = vec![];

       let kf_join =  start_listening_to_key_events(
        redis_conn,
            pubsub_conn,
            kafka_config
        ).await;

        kafka_joins.push(kf_join);
    

    let join_handle = spawn(async move {
        for handle in kafka_joins {
            handle.await.unwrap();
        }
    });

    return join_handle;

}





pub async fn start_listening_to_key_events(
    redis_conn: MultiplexedConnection,
    mut pubsub_conn: PubSub,
    kafka_config: &KafkaConfiguration
) -> JoinHandle<()> {

 let _ =  pubsub_conn .psubscribe("__keyspace@*__:*")
        .await
        .expect("Failed to subscribe to redis keyspace channel");
// let _ =  pubsub_conn.psubscribe(GAME_STAKE_TIME_OVER).await;


 let kafka_producer_for_settle_events = kafka::producer::create_new_kafka_producer(kafka_config).unwrap();
 let kafka_producer_for_game_over_events = kafka::producer::create_new_kafka_producer(kafka_config).unwrap();


    // Start listener
    tokio::spawn(async move {
        
        loop {
          let message_stream =   pubsub_conn.on_message().next().await;

          if message_stream.is_some() {
            println!("Reccieved some message from redis pubsub");
            let new_message = message_stream.unwrap();
            let expired_key_channel: String = new_message.get_channel().unwrap();


                       if expired_key_channel.contains(SETTLE_BET_KEY) {
                        let redis_payload = get_redis_payload_for_key(redis_conn.clone() , SETTLE_BET_KEY , expired_key_channel).await;

                        if let Some(redis_payload_val) = redis_payload {
                            
                        let _ = publish_game_bet_events_for_settlement(&kafka_producer_for_settle_events, vec![redis_payload_val]).await;
                        }
                       } else if expired_key_channel.contains(GAME_STAKE_TIME_OVER) {
                        
                        let redis_payload = get_redis_payload_for_key(redis_conn.clone() , GAME_STAKE_TIME_OVER , expired_key_channel).await;

                        if let Some(redis_payload_val) = redis_payload {
                            
                        let _ = publish_game_stake_time_over_event(&kafka_producer_for_game_over_events, vec![redis_payload_val]).await;
                        }
                       }


          }
        }
    })
}

pub async fn get_redis_payload_for_key(mut redis_conn: MultiplexedConnection , key_type: &str , input: String) -> Option<String> {
    // Need to fix this fn to get key value from key argument and then get actual redis value

    let key_id =    input.split(':')
    .last()
    // Split the last part by '_' and get the full remainder after the first part
    .and_then(|last_part| {
        // Split by '_' 
        let parts: Vec<&str> = last_part.split('_').collect();
        
        // If there are at least two parts, return everything after the first part
        if parts.len() > 1 {
            Some(parts[1..].join("_"))
        } else {
            None
        }
    }).unwrap();


    println!("The Key id is: {:?}" , key_id);

    let key = if key_type.eq(SETTLE_BET_KEY) {
            SETTLE_BET_KEY_DATA.to_string() + &key_id
    } else {
        GAME_STAKE_TIME_OVER_DATA.to_string() + &key_id
    };

    let key_res: RedisResult<String> = redis_conn.get(key).await;

    if key_res.is_err() {
        return None
    }

    return Some(key_res.unwrap())

}

pub async fn publish_game_bet_events_for_settlement(producer: &FutureProducer , kafka_events: Vec<String>) -> Result<(), KafkaError> {
    println!("PUBLISHING EVENTS FOR GAME_BET_GENERATE_EVENTS topic");

    producer.begin_transaction().unwrap();


    let kafka_result = future::try_join_all(kafka_events.iter().map(|event| async move {

        
        let delivery_result = producer
        .send(
            FutureRecord::to(GENERATE_GAME_BET_EVENTS)
                    .payload(&event)
                    .key("generate_game_bet_event"),
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


pub async fn publish_game_stake_time_over_event(producer: &FutureProducer , kafka_events: Vec<String>) -> Result<(), KafkaError> {
    println!("PUBLISHING EVENTS FOR EXECUTOR_GAME_STAKE_TIME_OVER topic");

    producer.begin_transaction().unwrap();


    let kafka_result = future::try_join_all(kafka_events.iter().map(|event| async move {

        
        let delivery_result = producer
        .send(
            FutureRecord::to(EXECUTOR_GAME_STAKE_TIME_OVER_EVENT)
                    .payload(&event)
                    .key("executor_game_stake_time_over"),
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