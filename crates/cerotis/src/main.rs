use std::{collections::HashMap, net::SocketAddr, sync::{Arc, Mutex}};

use api::health;
use axum::{routing::get, Router};
use conf::{config_types::ServerConfiguration, configuration::Configuration};
use context::context::{ContextImpl, DynContext};
use kafka::decoder::AvroRecordDecoder;
use mongodb::bson::{self, doc};
use mqtt_events::all_mqtt_events::{send_game_general_event_mqtt, send_game_invite_room_event_mqtt, send_game_move_event_mqtt, send_user_friend_request_event_mqtt, send_user_joined_room_event_mqtt, send_user_left_room_event_mqtt, send_user_status_room_event_mqtt};
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


extern crate paho_mqtt as mqtt;

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
    
    // let user_and_game_handles = init_user_and_game_kafka_consumer(
    //     context,
    //     &config, 
    //     consumers
    // );

    start_web_server(&config.server, vec![])
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

    let cli = mqtt_client::create_mqtt_client_for_kafka_consumer(&config.mqtt, topic.clone());
    // Start listener
    tokio::spawn(async move {
        do_listen( context, &stream_consumer, topic , &cli).await;
    })
}

pub async fn do_listen(
    context: DynContext,
    stream_consumer: &StreamConsumer,
    topic_name: String,
    cli: &mqtt::Client
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
                    let _ = user_collection.delete_one(doc! { "user_id": BsonUuid::parse_str(user_game_deletion_event.user_id.clone()).unwrap()}, None).await;
                    let _ = game_collection.delete_one(doc! { "host_id": user_game_deletion_event.user_id.clone()}, None).await;
                    let _ = user_turn_collection.delete_one(doc! { "host_id": user_game_deletion_event.user_id}, None).await;
                },
                "user_game_events" => {
                    let user_game_event_payload: UserGameMove = serde_json::from_str(&payload).unwrap();
                    
                    let rsp = game_collection.find_one(doc! { "id": BsonUuid::parse_str(user_game_event_payload.game_id.clone()).unwrap()}, None).await;
                    let rsp_clone = rsp.clone();
                    // Fix the fen update fn
                //    if !rsp_clone.is_err() && !rsp_clone.unwrap().is_none() {
                //     let game_model = rsp.unwrap().unwrap();

                //     let _rsp = if user_game_event_payload.move_type == "normal" {
                //         let gm_ev: ChessNormalEvent = serde_json::from_str(&user_game_event_payload.user_move).unwrap();
            
                //         let old_position: CellPosition = serde_json::from_str(&gm_ev.initial_cell).unwrap();
                //         let new_position: CellPosition = serde_json::from_str(&gm_ev.target_cell).unwrap();
                //         let piece: Vec<char> = gm_ev.piece.chars().collect();
                
                //         let updated_fen = update_fen(&game_model.current_state, old_position.x, old_position.y, new_position.x, new_position.y, *piece.get(0).unwrap());

                //             game_collection.update_one(doc! { "id": BsonUuid::parse_str(user_game_event_payload.game_id.clone()).unwrap()}, doc! { "$set": doc! {"current_state": updated_fen} }, None).await
                //     } else {
                //         let gm_ev: ChessPromotionEvent = serde_json::from_str(&user_game_event_payload.user_move).unwrap();
                //         let old_position: CellPosition = serde_json::from_str(&gm_ev.initial_cell).unwrap();
                //         let new_position: CellPosition = serde_json::from_str(&gm_ev.target_cell).unwrap();
                //         let piece: Vec<char> = gm_ev.piece.chars().collect();
                //         let updated_fen = update_fen(&game_model.current_state, old_position.x, old_position.y, new_position.x, new_position.y, *piece.get(0).unwrap());

                //             game_collection.update_one(doc! { "id": BsonUuid::parse_str(user_game_event_payload.game_id.clone()).unwrap()}, doc! { "$set": doc! {"current_state": updated_fen} }, None).await
                //     };

                   }
                   

                   _ => {
                    println!("No topics found")
                   }


                }

            }
            // if topic.to_string() == topic_name {
                
            //  if let Some(key_name) = message.key() {
            //     let key_name_string = String::from_utf8(key_name.to_vec()).unwrap();
            //     let payload = String::from_utf8(message.payload().unwrap().to_vec()).unwrap();
            //     match key_name_string.as_str() {
            //         USER_ONLINE_EVENT => {
            //                 change_user_online_status_in_db(&context , payload.clone()).await;
            //                // send_user_online_event_event_mqtt(cli , payload).await
            //         },

            //         USER_JOINED_ROOM => {
            //                 send_user_joined_room_event_mqtt(cli , payload).await
            //         },

            //         USER_LEFT_ROOM => {
            //             send_user_left_room_event_mqtt(cli , payload).await
            //         },

            //         FRIEND_REQUEST_EVENT => {
            //             send_user_friend_request_event_mqtt(cli , payload).await
            //         },

            //         GAME_INVITE_EVENT => {
            //             send_game_invite_room_event_mqtt(cli , payload).await
            //         },

            //         USER_GAME_MOVE => {
            //             send_game_move_event_mqtt(cli , payload).await
            //         },

            //         GAME_GENERAL_EVENT => {
            //             send_game_general_event_mqtt(cli , payload).await
            //         },

            //         USER_STATUS_EVENT => {
            //             send_user_status_room_event_mqtt(cli , payload).await
            //         },

            //         _ => {}
            //     }
            //  }


            // }

                
        }
        }
}

