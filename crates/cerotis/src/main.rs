use std::{collections::HashMap, net::SocketAddr, sync::{Arc, Mutex}};

use api::health;
use axum::{routing::get, Router};
use conf::{config_types::{ConsumerConfiguration, KafkaConfiguration, ServerConfiguration}, configuration::Configuration};
use context::context::{ContextImpl, DynContext};
use kafka::decoder::AvroRecordDecoder;
use rdkafka::{consumer::StreamConsumer, message::{BorrowedHeaders, BorrowedMessage, Headers}, Message};
use tokio::{spawn, task::JoinHandle};
use tracing::warn;

use crate::{avro::create_user_online_avro::CreateUserOnlineAvro, controllers::user_controller::set_user_online_and_send_it_to_friends, kafka::key_avro::KeyAvro};


pub mod kafka;
pub mod conf;
pub mod api;
pub mod context;
pub mod mongo_pool;
pub mod avro;
pub mod controllers;
pub mod mqtt_client;


extern crate paho_mqtt as mqtt;

#[tokio::main]
async fn main()  {
    let config = conf::configuration::Configuration::load().unwrap();

    let mut consumers = kafka::consumer::init_consumers(&config.kafka).unwrap();
    
    let avro_decoder = AvroRecordDecoder::new(&config.kafka).unwrap();
    
    
    let client = redis::Client::open(config.redis_url.url.clone()).unwrap();
    let redis_connection = client.get_connection().unwrap(); 
    let mongo_db_client = Arc::new(mongo_pool::init_db_client(&config.mongo_db).await.unwrap());
    
    
    let context = ContextImpl::new_dyn_context(mongo_db_client,  Arc::new(Mutex::new(redis_connection)), Arc::new(avro_decoder));
    
    let user_and_game_handles = init_user_and_game_kafka_consumer(
        context,
        &config, 
        consumers
    );

    start_web_server(&config.server, vec![
        user_and_game_handles,
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
    user_topic: String,
    cli: &mqtt::Client
) {

    let decoder = context.get_avro_decoder().clone();

    loop {
        match stream_consumer.recv().await {
            Err(e) => warn!("Error: {}", e),
            Ok(message) => {
 
                    let topic = message.topic();


                    let key_result = decoder
                        .decode(message.key())
                        .await
                        .expect("Couldn't decode avro message");

                    let key = apache_avro::from_value::<KeyAvro>(&key_result.value)
                        .expect("Couldn't deserialize KeyAvro");

                    let payload_result = decoder
                        .decode(message.payload())
                        .await
                        .expect("Couldn't decode payload");

                        // Data type should tell what kind of event is recieved
                    match key.identifier.data_type.as_str() {
                            "set-user-online" => {
                                
                    let payload = apache_avro::from_value::<CreateUserOnlineAvro>(&payload_result.value)
                    .expect("Couldn't deserialize CreateUserOnlineAvro");

                        
                    set_user_online_and_send_it_to_friends(context.clone() , payload);
                            },

                            "user-game-move-event" => {

                            },

                            _ => {}
                    }

                
            }
        }
    }
}