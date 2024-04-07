use std::sync::{Arc, Mutex};

use axum::{handler::Handler, Router};
use dotenv::dotenv;
use sea_orm::Database;
use socketioxide::{extract::SocketRef, socket, SocketIo};
use tower::ServiceBuilder;
use tower_cookies::CookieManagerLayer;
use tower_http::cors::CorsLayer;
use crate::{conf::configuration, context::context::ContextImpl, mongo::{event_converter::DynEventConverter, event_dispatcher::EventDispatcher, user_game_converter::UserGameEventConverter}, state::{AppDBState, WebSocketStates}};


pub mod errors;
pub mod controllers;
pub mod routes;
pub mod kafka;
pub mod common;
pub mod state;
pub mod constants;
pub mod context;
pub mod ws_events;
pub mod utils;
pub mod conf;
pub mod mongo;
pub mod event_producer;
pub mod mongo_pool;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>>  {
    let config = configuration::Configuration::load().unwrap();
    dotenv().ok();

    //Connect with database
    let connection = match Database::connect(config.postgres_url.url).await {
        Ok(connection) => connection,
        Err(e) => panic!("{:?}",e)
    };

    let client = redis::Client::open(config.redis_url.url).unwrap();
    let redis_connection = client.get_connection().unwrap(); 
    let mongo_db_client = Arc::new(mongo_pool::init_db_client(&config.mongo_db).await.unwrap());

    let user_mongo_event_converter: Arc<DynEventConverter> =
    Arc::new(Box::new(UserGameEventConverter::new(&config.kafka)?));

    //Initializing Event Dispatcher
    let event_dispatcher = EventDispatcher::new(vec![
        user_mongo_event_converter,
    ]);

    let context = ContextImpl::new_dyn_context(
        mongo_db_client,
        Arc::new(Mutex::new(redis_connection)),
        Arc::new(event_dispatcher),
        connection.clone()
    );

   // io.ns("/", ws_events::user_events::create_ws_user_events);

    
  //  Migrator::up(&conn, None).await.unwrap();
  
  let kafka_producer = kafka::init_producer::create_new_kafka_producer().unwrap();
  let kafka_prod_clone = kafka_producer.clone();
  let context_clone = context.clone();
  let state = AppDBState {conn: connection , from_email: config.email_config.from_email , smtp_key: config.email_config.smtp_key, context: context, producer: kafka_producer };
    let websocket_states = WebSocketStates { producer: kafka_prod_clone , context: context_clone };
    let (layer, io) = SocketIo::builder().with_state(websocket_states).build_layer();

    io.ns("/",   ws_events::game_events::create_ws_game_events);
    // build our application with a route
    let user_auth_routes = routes::user_auth_routes::create_user_routes() ;
    let user_logic_routes = routes::user_logic_routes::create_user_logic_routes();
    let game_routes = routes::game_logic_routes::create_game_routes();
    let routes_all = Router::new()
                            .nest( "/api/v1/auth", user_auth_routes)
                            .nest("/api/v1/user", user_logic_routes)
                            .nest( "/api/v1/game", game_routes)
                            .layer(ServiceBuilder::new()
                                    .layer(layer)
                                    .layer(CookieManagerLayer::new())
                                    .layer(CorsLayer::permissive()))
                            .with_state(state)
                            .with_state(io);


    // run it
    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000")
        .await
        .unwrap();
    println!("listening on {}", listener.local_addr().unwrap());
    axum::serve(listener, routes_all).await.unwrap();

    Ok(())
}
