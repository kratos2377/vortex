use std::{collections::HashMap, net::SocketAddr, sync::Arc};

use axum::{response::IntoResponse, routing::get, Router};
use bson::doc;
use conf::{config_types::ServerConfiguration, configuration::Configuration};
use context::context::{ContextImpl, DynContext};
use orion::{events::kafka_event::UserGameDeletetionEvent, models::{game_model::Game, user_game_relation_model::UserGameRelation}};
use rdkafka::{consumer::StreamConsumer, Message};
use serde_json::json;
use tokio::{spawn, task::JoinHandle};
use tracing::warn;
use bson::Uuid as BsonUuid;
pub mod kafka;
pub mod conf;
pub mod context;
pub mod init_mongo_connection;

#[tokio::main]
async fn main() {
    let config = conf::configuration::Configuration::load().unwrap();

    let consumers = kafka::consumer::init_consumers(&config.kafka).unwrap();
    let mongo_db_client = Arc::new(init_mongo_connection::init_db_client(&config.mongo_db).await.unwrap());
    let context = ContextImpl::new_dyn_context(mongo_db_client);

    let user_game_deletion = init_user_and_game_deletion_kafka_consumer(
        context,
        &config, 
        consumers
    );

    start_web_server(&config.server, vec![
        user_game_deletion,
    ])
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

    let listener = tokio::net::TcpListener::bind("127.0.0.1:3006")
    .await
    .unwrap();
println!("listening on {}", listener.local_addr().unwrap());
axum::serve(listener, routing.into_make_service_with_connect_info::<SocketAddr>()).await.unwrap();

    // Shutdown tracing provider
}


pub async fn shutdown_signal(shutdown_handles: Vec<JoinHandle<()>>) {
    // let ctrl_c = async {
    //     tokio::signal::ctrl_c()
    //         .await
    //         .expect("Initialization of Ctrl+C handler failed");
    // };

    // #[cfg(unix)]
    // let terminate = async {
    //     tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
    //         .expect("Initialization of signal handler failed")
    //         .recv()
    //         .await;
    // };

    // #[cfg(not(unix))]
    // let terminate = std::future::pending::<()>();

    // tokio::select! {
    //     _ = ctrl_c => {},
    //     _ = terminate => {},
    // }

    for handle in shutdown_handles {
        handle.abort();
    }
}


fn init_routing() -> Router {
    let base_router = Router::new().route("/health", get(health));

    return base_router;

}

pub async fn health() -> impl IntoResponse {
    axum::Json(json!({ "status" : "UP" }))
}



fn init_user_and_game_deletion_kafka_consumer(
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
        do_listen( context, &stream_consumer, topic).await;
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

    loop {
        match stream_consumer.recv().await {
            Err(e) => warn!("Error: {}", e),
            Ok(message) => {
 
            let topic = message.topic();
            if topic.to_string() == topic_name {
                
                let payload = String::from_utf8(message.payload().unwrap().to_vec()).unwrap();
                let user_game_deletion_event: UserGameDeletetionEvent = serde_json::from_str(&payload).unwrap();
                let _ = user_collection.delete_one(doc! { "user_id": BsonUuid::parse_str(user_game_deletion_event.user_id.clone()).unwrap()}, None).await;
                let _ = game_collection.delete_one(doc! { "host_id": user_game_deletion_event.user_id}, None).await;
             


            }

                
            }
        }
    }
}
