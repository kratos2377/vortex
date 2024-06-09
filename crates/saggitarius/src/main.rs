use std::{collections::HashMap, net::SocketAddr, sync::Arc, time::Duration};

use axum::{http::Error, response::IntoResponse, routing::get, Router};
use conf::{config_types::ServerConfiguration, configuration::SaggitariusConfiguration};
use context::DynContext;
use futures::future;
use mongodb::bson::{self, doc};
use orion::{constants::USER_GAME_MOVE, events::kafka_event::KafkaGeneralEvent, models::{chess_events::{ChessNormalEvent, ChessPromotionEvent}, game_model::Game, user_game_event::UserGameEvent}};
use rdkafka::{consumer::StreamConsumer, error::KafkaError, message::BorrowedMessage, producer::{FutureProducer, FutureRecord, Producer}, util::Timeout, Message};
use serde_json::json;
use tokio::{spawn, task::JoinHandle};
use tracing::warn;
use bson::Uuid as BsonUuid;
use crate::{context::ContextImpl, mongo::pool};



pub mod conf;
pub mod mongo;
pub mod context;
pub mod event;
pub mod kafka;

#[tokio::main]
async fn main() {

    let config = conf::configuration::SaggitariusConfiguration::load().unwrap();
    let db_client = Arc::new(pool::init_db_client(&config.mongo_database).await.unwrap());
    let context = ContextImpl::new_dyn_context(db_client);

    let producer = Arc::new(kafka::producer::init_producer(&config.kafka).unwrap());
    let consumers = kafka::consumer::init_consumers(&config.kafka).unwrap();
    // Start Consumers
    let game_handles = init_game_consumer(
        context,
        &config, 
        consumers,
        &producer
    );

    // Start the web-server
    start_web_server(&config.server, vec![
        game_handles,
    ]).await;

}


async fn start_web_server(config: &ServerConfiguration,   shutdown_handles: Vec<JoinHandle<()>>,) {
    // Initialize routing
    let routing = init_routing();

    let addr = SocketAddr::from(([0, 0, 0, 0], config.port));


    let listener = tokio::net::TcpListener::bind("127.0.0.1:3003")
    .await
    .unwrap();
println!("listening on {}", listener.local_addr().unwrap());
axum::serve(listener, routing.into_make_service_with_connect_info::<SocketAddr>()).await.unwrap();

    // Shutdown tracing provider
}

// Add gracefil shutdown later
// pub async fn shutdown_signal() {
//     let ctrl_c = async {
//         tokio::signal::ctrl_c()
//             .await
//             .expect("Initialization of Ctrl+C handler failed");
//     };

//     #[cfg(unix)]
//     let terminate = async {
//         tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
//             .expect("Initialization of signal handler failed")
//             .recv()
//             .await;
//     };

//     #[cfg(not(unix))]
//     let terminate = std::future::pending::<()>();

//     tokio::select! {
//         _ = ctrl_c => {},
//         _ = terminate => {},
//     }
// }


fn init_routing() -> Router {
    Router::new()
        .route("/health", get(health))
}

pub async fn health() -> impl IntoResponse {
    axum::Json(json!({ "status" : "UP" }))
}

fn init_game_consumer(
    context: DynContext,
    config: &SaggitariusConfiguration,
    kafka_consumers: HashMap<String, StreamConsumer>,
    producer: &FutureProducer
) -> JoinHandle<()> {

    let mut kafka_joins: Vec<JoinHandle<()>> = vec![];

    for (key_topic , value) in kafka_consumers.into_iter() {
       let kf_join =  listen(
            context.clone(),
            config,
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
    config: &SaggitariusConfiguration,
    stream_consumer: StreamConsumer,
    key_topic: String,
    producer: &FutureProducer
) -> JoinHandle<()> {
    let topic = key_topic.clone();
    let prod_clone = producer.clone();
    // Start listener
    tokio::spawn(async move {
        do_listen( context, &stream_consumer, topic , &prod_clone).await;
    })
}

pub async fn do_listen(
    context: DynContext,
    stream_consumer: &StreamConsumer,
    topic_name: String,
    producer: &FutureProducer
) {
    let mongod_db = context.db_client().database("user_game_events_db");
    let game_collection = mongod_db.collection::<Game>("games");

    loop {
        match stream_consumer.recv().await {
            Err(e) => warn!("Error: {}", e),
            Ok(message) => {
 
            let topic = message.topic();
            let key_name_string = String::from_utf8(message.key().unwrap().to_vec()).unwrap();
            let payload = String::from_utf8(message.payload().unwrap().to_vec()).unwrap();
            if topic.to_string() == topic_name {
         
     
                let user_game_event_payload: UserGameEvent = serde_json::from_str(&payload).unwrap();
                match key_name_string.as_str() {
                    
                    USER_GAME_MOVE => {

                        let rsp = game_collection.find_one(doc! { "id": BsonUuid::parse_str(user_game_event_payload.user_game_move.game_id.clone()).unwrap()}, None).await;
                        let game_model = rsp.unwrap().unwrap();

                        let rsp = if user_game_event_payload.user_game_move.move_type == "normal" {
                            let gm_ev: ChessNormalEvent = serde_json::from_str(&user_game_event_payload.user_game_move.user_move).unwrap();
                            let mut ranks = game_model.current_state.split('/').map(|rank| rank.chars().collect::<Vec<char>>()).collect::<Vec<Vec<char>>>();
                            let old_position: (usize,usize) = serde_json::from_str(&gm_ev.initial_cell).unwrap();
                            let new_position: (usize,usize) = serde_json::from_str(&gm_ev.target_cell).unwrap();
                            let piece: Vec<char> = gm_ev.piece.chars().collect();
                            ranks[old_position.0 as usize][old_position.1 as usize] = '0';
                            ranks[new_position.0 as usize][new_position.1 as usize] = *piece.get(0).unwrap(); 
                    
                            let updated_fen = ranks
                                .iter()
                                .map(|rank| rank.iter().collect::<String>())
                                .collect::<Vec<String>>()
                                .join("/");

                                game_collection.update_one(doc! { "id": BsonUuid::parse_str(user_game_event_payload.user_game_move.game_id.clone()).unwrap()}, doc! { "$set": doc! {"current_state": updated_fen} }, None).await
                        } else {
                            let gm_ev: ChessPromotionEvent = serde_json::from_str(&user_game_event_payload.user_game_move.user_move).unwrap();
                            let mut ranks = game_model.current_state.split('/').map(|rank| rank.chars().collect::<Vec<char>>()).collect::<Vec<Vec<char>>>();
                            let old_position: (usize,usize) = serde_json::from_str(&gm_ev.initial_cell).unwrap();
                            let new_position: (usize,usize) = serde_json::from_str(&gm_ev.target_cell).unwrap();
                            let piece: Vec<char> = gm_ev.piece.chars().collect();
                            ranks[old_position.0 as usize][old_position.1 as usize] = '0';
                            ranks[new_position.0 as usize][new_position.1 as usize] = *piece.get(0).unwrap(); 
                    
                            let updated_fen = ranks
                                .iter()
                                .map(|rank| rank.iter().collect::<String>())
                                .collect::<Vec<String>>()
                                .join("/");

                                game_collection.update_one(doc! { "id": BsonUuid::parse_str(user_game_event_payload.user_game_move.game_id.clone()).unwrap()}, doc! { "$set": doc! {"current_state": updated_fen} }, None).await
                        };


                    }

                    _ => {}
                }
             


            }

                let _ = send_events_to_game_topic("game".to_string() , key_name_string , payload , producer).await;
            }
        }
    }
}


pub async fn send_events_to_game_topic(topic: String , event_key: String , payload: String , producer: &FutureProducer)  -> Result<(), KafkaError> {
    let kafka_events = vec![KafkaGeneralEvent {topic: topic.clone(), payload: payload, key: USER_GAME_MOVE.to_string() }];
    // Start transaction and execute query
    producer.begin_transaction().unwrap();

    let kafka_result = future::try_join_all(kafka_events.iter().map(|event| async move {
       let delivery_result = producer
       .send(
           FutureRecord::to(&event.topic)
                   .payload(&event.payload)
                   .key(&event.key.clone()),
           Duration::from_secs(3),
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

   producer.commit_transaction(Timeout::from(Duration::from_secs(5))).unwrap();

    Ok(())
}