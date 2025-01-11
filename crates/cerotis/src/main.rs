use std::{collections::HashMap, net::SocketAddr, sync::{Arc, Mutex}};

use api::health;
use axum::{routing::get, Router};
use conf::{config_types::ServerConfiguration, configuration::Configuration};
use context::context::{ContextImpl, DynContext};
use kafka::decoder::AvroRecordDecoder;
use mongodb::bson::{self, doc};
use orion::{constants::{FRIEND_REQUEST_EVENT, GAME_GENERAL_EVENT, GAME_INVITE_EVENT, USER_GAME_MOVE, USER_JOINED_ROOM, USER_LEFT_ROOM, USER_ONLINE_EVENT, USER_STATUS_EVENT}, events::{kafka_event::{UserFriendRequestKafkaEvent, UserGameDeletetionEvent}, ws_events::UserConnectionEventPayload}, models::{chess_events::{CellPosition, ChessNormalEvent, ChessPromotionEvent}, game_model::Game, user_game_event::UserGameMove, user_game_relation_model::UserGameRelation, user_turn_model::UserTurnMapping}};
use rdkafka::{consumer::StreamConsumer, Message};
use sea_orm::{ColIdx, Database, Set};
use tokio::{spawn, task::JoinHandle};
use tracing::warn;
use ton::models;
use bson::Uuid as BsonUuid;


pub mod kafka;
pub mod conf;
pub mod api;
pub mod context;
pub mod mongo_pool;
pub mod avro;
pub mod mqtt_client;
pub mod mqtt_events;



#[tokio::main]
async fn main()  {
    let config = conf::configuration::Configuration::load().unwrap();

    let consumers = kafka::consumer::init_consumers(&config.kafka).unwrap();
    
    let avro_decoder = AvroRecordDecoder::new(&config.kafka).unwrap();
    
    let connection = match Database::connect(config.postgres_url.url.clone()).await {
        Ok(connection) => connection,
        Err(e) => panic!("{:?}",e)
    };
    
    let client = redis::Client::open(config.redis_url.url.clone()).unwrap();
    let redis_connection = client.get_connection().unwrap(); 
    let mongo_db_client = Arc::new(mongo_pool::init_db_client(&config.mongo_db).await.unwrap());
    
    
    let context = ContextImpl::new_dyn_context(mongo_db_client,  Arc::new(Mutex::new(redis_connection)), Arc::new(avro_decoder) , connection);
    
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

    loop {
        match stream_consumer.recv().await {
            Err(e) => warn!("Error: {}", e),
            Ok(message) => {
 
            let topic = message.topic();
            let payload = String::from_utf8(message.payload().unwrap().to_vec()).unwrap();

            match topic {
                "user_game_deletion" => {
                    let user_game_deletion_event: UserGameDeletetionEvent = serde_json::from_str(&payload).unwrap();
                    let _ = user_collection.delete_one(doc! { "user_id": user_game_deletion_event.user_id.clone()}, None).await;
                    let _ = game_collection.delete_one(doc! { "host_id": user_game_deletion_event.user_id.clone()}, None).await;
                    let _ = user_turn_collection.delete_one(doc! { "host_id": user_game_deletion_event.user_id}, None).await;
                },
                "user_game_events" => {
                    println!("User Game Events Recieved");
                    let user_game_event_payload: UserGameMove = serde_json::from_str(&payload).unwrap();

                        println!("GAME_ID is: {:?}" , user_game_event_payload.game_id.clone());

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
                
                        let updated_fen = update_fen(&game_model.chess_state, old_position.x, old_position.y, new_position.x, new_position.y, *piece.get(0).unwrap());
                        println!("Updated Fen is: {:?}" , updated_fen);
                            game_collection.update_one(doc! { "id": BsonUuid::parse_str(user_game_event_payload.game_id.clone()).unwrap()}, doc! { "$set": doc! {"chess_state": updated_fen} }, None).await
                    } else {
                        let gm_ev: ChessPromotionEvent = serde_json::from_str(&user_game_event_payload.user_move).unwrap();
                        let old_position: CellPosition = serde_json::from_str(&gm_ev.initial_cell).unwrap();
                        let new_position: CellPosition = serde_json::from_str(&gm_ev.target_cell).unwrap();
                        let piece: Vec<char> = gm_ev.piece.chars().collect();
                        let updated_fen = update_fen(&game_model.chess_state, old_position.x, old_position.y, new_position.x, new_position.y, *piece.get(0).unwrap());

                            game_collection.update_one(doc! { "id": BsonUuid::parse_str(user_game_event_payload.game_id.clone()).unwrap()}, doc! { "$set": doc! {"chess_state": updated_fen} }, None).await
                    };

                   } else {
                    println!("Some error recieved before update: {:?}", rsp.unwrap().is_none())
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