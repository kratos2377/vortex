use std::{collections::HashMap, net::SocketAddr, sync::Arc};

use axum::Router;
use conf::{config_types::ServerConfiguration, configuration::Configuration};
use context::context::{ContextImpl, DynContext};
use rdkafka::{consumer::StreamConsumer, Message};
use routes::user_game_bet_routes;
use sea_orm::Database;
use state::AppDBState;
use tokio::{spawn, task::JoinHandle};
use tower::ServiceBuilder;
use tower_cookies::CookieManagerLayer;
use tower_http::cors::CorsLayer;
use tracing::warn;


pub mod conf;
pub mod controllers;
pub mod routes;
pub mod utils;
pub mod state;
pub mod errors;
pub mod mongo_pool;
pub mod kafka;
pub mod context;


#[tokio::main]
async fn main()-> Result<(), Box<dyn std::error::Error>>  {

    let config = conf::configuration::Configuration::load().unwrap();
    //  dotenv().ok();
  
      //Connect with database
      let connection = match Database::connect(&config.postgres_url.url).await {
          Ok(connection) => connection,
          Err(e) => panic!("{:?}",e)
      };
  
      let mongo_db_client = Arc::new(mongo_pool::init_db_client(&config.mongo_db).await.unwrap());

      let state = AppDBState {conn: connection.clone(), mongo_conn: mongo_db_client.clone()};


      let context = ContextImpl::new_dyn_context(mongo_db_client , connection);

      let consumers =  kafka::consumer::init_consumers(&config.kafka).unwrap();
    
      let game_bet_handles = init_game_bet_events_consumer(
          context,
          consumers
      );
  
      start_web_server(state, &config.server, vec![game_bet_handles])
      .await;
  


  
      Ok(())

}



async fn start_web_server(
    state: AppDBState,
    config: &ServerConfiguration,
    shutdown_handles: Vec<JoinHandle<()>>,
) {

    // Start server
    let addr = SocketAddr::from(([0, 0, 0, 0], config.port));
    tracing::info!("listening on {addr}");


    let user_game_bet_routes = routes::user_game_bet_routes::create_user_game_bet_routes() ;
    let routes_all = Router::new()
                            .nest( "/api/v1/game_bets", user_game_bet_routes)
                            .layer(ServiceBuilder::new()
                                    .layer(CookieManagerLayer::new())
                                    .layer(CorsLayer::permissive()))
                            .with_state(state);


    // run it
    let listener = tokio::net::TcpListener::bind("127.0.0.1:3020")
        .await
        .unwrap();
    println!("listening on {}", listener.local_addr().unwrap());
    axum::serve(listener, routes_all).await.unwrap();

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




fn init_game_bet_events_consumer(
    context: DynContext,
    kafka_consumers: HashMap<String, StreamConsumer>,
) -> JoinHandle<()> {

    let mut kafka_joins: Vec<JoinHandle<()>> = vec![];

    for (key_topic , value) in kafka_consumers.into_iter() {
       let kf_join =  listen(
            context.clone(),
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

    let postgres_conn = context.get_postgres_db_client();

    loop {
        match stream_consumer.recv().await {
            Err(e) => warn!("Error: {}", e),
            Ok(message) => {
 
            let topic = message.topic();
            let payload = String::from_utf8(message.payload().unwrap().to_vec()).unwrap();

            match topic {


                _ => {
                    println!("No topics found")
                   }

            }
    
                
        }
        }
}

}