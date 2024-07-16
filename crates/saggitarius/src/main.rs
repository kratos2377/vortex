use std::{collections::HashMap, net::SocketAddr, sync::Arc, time::Duration};

use axum::{http::Error, response::IntoResponse, routing::get, Router};
use conf::{config_types::ServerConfiguration, configuration::SaggitariusConfiguration};
use context::DynContext;
use futures::future;
use mongodb::bson::{self, doc};
use orion::{constants::USER_GAME_MOVE, events::kafka_event::KafkaGeneralEvent, models::{chess_events::{CellPosition, ChessNormalEvent, ChessPromotionEvent}, game_model::Game, user_game_event::UserGameMove}};
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
         
     
                let user_game_event_payload: UserGameMove = serde_json::from_str(&payload).unwrap();
                match key_name_string.as_str() {
                    
                    USER_GAME_MOVE => {
                        let rsp = game_collection.find_one(doc! { "id": BsonUuid::parse_str(user_game_event_payload.game_id.clone()).unwrap()}, None).await;
                        let rsp_clone = rsp.clone();
                       if !rsp_clone.is_err() && !rsp_clone.unwrap().is_none() {
                        let game_model = rsp.unwrap().unwrap();

                        let _rsp = if user_game_event_payload.move_type == "normal" {
                            let gm_ev: ChessNormalEvent = serde_json::from_str(&user_game_event_payload.user_move).unwrap();
                
                            let old_position: CellPosition = serde_json::from_str(&gm_ev.initial_cell).unwrap();
                            let new_position: CellPosition = serde_json::from_str(&gm_ev.target_cell).unwrap();
                            let piece: Vec<char> = gm_ev.piece.chars().collect();
                    
                            let updated_fen = update_fen(&game_model.current_state, old_position.x, old_position.y, new_position.x, new_position.y, *piece.get(0).unwrap());

                                game_collection.update_one(doc! { "id": BsonUuid::parse_str(user_game_event_payload.game_id.clone()).unwrap()}, doc! { "$set": doc! {"current_state": updated_fen} }, None).await
                        } else {
                            let gm_ev: ChessPromotionEvent = serde_json::from_str(&user_game_event_payload.user_move).unwrap();
                            let old_position: CellPosition = serde_json::from_str(&gm_ev.initial_cell).unwrap();
                            let new_position: CellPosition = serde_json::from_str(&gm_ev.target_cell).unwrap();
                            let piece: Vec<char> = gm_ev.piece.chars().collect();
                            let updated_fen = update_fen(&game_model.current_state, old_position.x, old_position.y, new_position.x, new_position.y, *piece.get(0).unwrap());

                                game_collection.update_one(doc! { "id": BsonUuid::parse_str(user_game_event_payload.game_id.clone()).unwrap()}, doc! { "$set": doc! {"current_state": updated_fen} }, None).await
                        };

                       }
                       
                       

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


// FEN conversion functions
pub fn update_fen(fen: &str, initial_rank: i64 , initial_file: i64, target_rank: i64, target_file: i64, piece: char) -> String {
    let parts: Vec<&str> = fen.split_whitespace().collect();
    let mut piece_placement = parts[0].to_string();
    let active_color = parts[1];
    let mut castling_availability = parts[2].to_string();
    let mut en_passant_target = parts[3].to_string();
    let mut halfmove_clock: i32 = parts[4].parse().unwrap();
    let mut fullmove_number: i32 = parts[5].parse().unwrap();

    // Convert board to 2D array for easier manipulation
    let mut board: Vec<Vec<char>> = piece_placement
        .split('/')
        .map(|row| {
            let mut new_row = Vec::new();
            for ch in row.chars() {
                if ch.is_digit(10) {
                    let count = ch.to_digit(10).unwrap();
                    for _ in 0..count {
                        new_row.push('1');
                    }
                } else {
                    new_row.push(ch);
                }
            }
            new_row
        })
        .collect();

    let initial_rank = initial_rank;
    let initial_file = initial_file;
    let target_rank = target_rank;
    let target_file = target_file;

    // Move the piece on the board
    board[initial_rank as usize][initial_file as usize] = '1';
    board[target_rank as usize][target_file as usize] = piece;

    // Reconstruct the piece placement string
    piece_placement = board
        .iter()
        .map(|row| {
            let mut new_row = String::new();
            let mut empty_count = 0;
            for &square in row {
                if square == '1' {
                    empty_count += 1;
                } else {
                    if empty_count > 0 {
                        new_row.push_str(&empty_count.to_string());
                        empty_count = 0;
                    }
                    new_row.push(square);
                }
            }
            if empty_count > 0 {
                new_row.push_str(&empty_count.to_string());
            }
            new_row
        })
        .collect::<Vec<String>>()
        .join("/");

    // Toggle active color
    let active_color = if active_color == "w" { "b" } else { "w" };

    // Handle castling rights (simplified version, real implementation needs more detail)
    if piece.to_ascii_lowercase() == 'k' {
        if piece == 'K' {
            castling_availability = castling_availability.replace('K', "").replace('Q', "");
        } else {
            castling_availability = castling_availability.replace('k', "").replace('q', "");
        }
    }
    if initial_rank == 0 && initial_file == 0 || target_rank == 0 && target_file == 0 {
        castling_availability = castling_availability.replace('Q', "");
    }
    if initial_rank == 0 && initial_file == 7 || target_rank == 0 && target_file == 7 {
        castling_availability = castling_availability.replace('K', "");
    }
    if initial_rank == 7 && initial_file == 0 || target_rank == 7 && target_file == 0 {
        castling_availability = castling_availability.replace('q', "");
    }
    if initial_rank == 7 && initial_file == 7 || target_rank == 7 && target_file == 7 {
        castling_availability = castling_availability.replace('k', "");
    }
    if castling_availability.is_empty() {
        castling_availability = "-".to_string();
    }

    // Handle en passant target (simplified, real implementation needs more detail)
    if piece.to_ascii_lowercase() == 'p' && (initial_rank as i32 - target_rank as i32).abs() == 2 {
        en_passant_target = format!(
            "{}{}",
            (b'a' + target_file as u8) as char,
            8 - ((initial_rank + target_rank) / 2)
        );
    } else {
        en_passant_target = "-".to_string();
    }

    // Update halfmove clock and fullmove number
    if piece.to_ascii_lowercase() == 'p' || board[target_rank as usize][target_file as usize] != '1' {
        halfmove_clock = 0;
    } else {
        halfmove_clock += 1;
    }
    if active_color == "w" {
        fullmove_number += 1;
    }

    // Construct the new FEN string
    format!(
        "{} {} {} {} {} {}",
        piece_placement, active_color, castling_availability, en_passant_target, halfmove_clock, fullmove_number
    )

}
