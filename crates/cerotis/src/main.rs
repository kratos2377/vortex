use std::{borrow::BorrowMut, collections::HashMap, net::SocketAddr};

use api::health;
use axum::{routing::get, Router};
use conf::config_types::{ConsumerConfiguration, KafkaConfiguration, ServerConfiguration};
use rdkafka::consumer::StreamConsumer;
use tokio::{spawn, task::JoinHandle};


pub mod kafka;
pub mod conf;
pub mod api;

#[tokio::main]
async fn main()  {
    let config = conf::configuration::Configuration::load().unwrap();
    let mut consumers = kafka::consumer::init_consumers(&config.kafka).unwrap();
    let user_handle = init_user_kafka_consumer(
        &config.kafka,
        consumers
    );

    start_web_server(&config.server, vec![
        user_handle,
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
axum::serve(listener, routing.into_make_service_with_connect_info::<SocketAddr>()).await.unwrap();

    // Shutdown tracing provider
}

fn init_routing() -> Router {
    let base_router = Router::new().route("/health", get(health));

    return base_router;

}


fn init_user_kafka_consumer(
    config: &KafkaConfiguration,
    kafka_consumers: HashMap<String, StreamConsumer>,
) -> JoinHandle<()> {

    let mut kafka_joins: Vec<JoinHandle<()>> = vec![];

    for (_ , value) in kafka_consumers.into_iter() {
       let kf_join =  listen(
            config,
            value
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
    config: &KafkaConfiguration,
    stream_consumer: StreamConsumer,
) -> JoinHandle<()> {
    let topic = get_user_topic_name(config);
    // Start listener
    tokio::spawn(async move {
        do_listen( &stream_consumer, topic).await;
    })
}

pub async fn do_listen(
    stream_consumer: &StreamConsumer,
    user_topic: String,
) {

    loop {
        match stream_consumer.recv().await {
            Err(e) =>println!("NEW ERROR OCCURED, times 1, {:?}" , e),
            Ok(message) => {
               println!("THE NEW MESSAGE HAS BEEN REVCIEVED");
               println!("{:?}", message)
            }
        }
    }
}


fn get_user_topic_name(config: &KafkaConfiguration) -> String {
    // Get consumer configuration
    let consumer_config: Vec<&ConsumerConfiguration> = config
        .consumer
        .iter()
        .filter(|c| c.id.clone() == "user" || c.id.clone() == "game")
        .collect();

    // Get topic name
    let topic = consumer_config
        .first()
        .expect("user consumer configuration not found")
        .topic
        .clone()
        .first()
        .expect("user topic not found in consumer configuration")
        .clone();

    topic
}


