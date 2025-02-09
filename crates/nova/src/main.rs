use std::{net::SocketAddr, time::Duration};

use axum::{response::IntoResponse, routing::get, Router};
use conf::config_types::ServerConfiguration;
use context::context::ContextImpl;
use futures::{future, StreamExt};
use orion::{constants::{GENERATE_GAME_BET_EVENTS, SETTLE_BET_KEY}, events::kafka_event::GenerateGameBetSettleEvents};
use rdkafka::{error::KafkaError, producer::{FutureProducer, FutureRecord, Producer}, util::Timeout};
use redis::aio::{MultiplexedConnection, PubSub};
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


#[tokio::main]
async fn main()-> Result<(), Box<dyn std::error::Error>>  {

    let config = conf::configuration::Configuration::load().unwrap();
    //  dotenv().ok();
  
      

      let mut client = redis::Client::open(config.redis.url).unwrap();
      let mut async_pubsub_conn = client.get_async_pubsub().await.unwrap();



      let kafka_producer = kafka::producer::create_new_kafka_producer(&config.kafka).unwrap();



      let publishing_events_join_handle = init_redis_pubsub_and_produce_events(kafka_producer , async_pubsub_conn).await;

      

  
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


pub async fn init_redis_pubsub_and_produce_events(producer: FutureProducer , pubsub_conn: PubSub) -> JoinHandle<()> {

    let mut kafka_joins: Vec<JoinHandle<()>> = vec![];

       let kf_join =  start_listening_to_key_events(
            pubsub_conn,
            &producer
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
    mut pubsub_conn: PubSub,
    producer: &FutureProducer
) -> JoinHandle<()> {

 let res =  pubsub_conn.psubscribe("*").await;


  let new_producer = producer.clone();


    // Start listener
    tokio::spawn(async move {
        loop {
          let message_stream =   pubsub_conn.on_message().next().await;

          if message_stream.is_some() {
            println!("Reccieved some message from redis pubsub");
            let new_message = message_stream.unwrap();
            let payload: String= new_message.get_payload().unwrap();
            let prod = new_producer.clone();

                let _ = tokio::spawn(async move {


                        let _ =publish_game_bet_events_for_settlement(&prod, vec![payload]).await;

                }).await;

          }
        }
    })
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