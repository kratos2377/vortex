use std::sync::{Arc, Mutex};

use axum::{handler::Handler, Router};
use migration::{Migrator, MigratorTrait};
use sea_orm::Database;
use tower::ServiceBuilder;
use tower_cookies::CookieManagerLayer;
use tower_http::cors::CorsLayer;
use crate::{conf::configuration, context::context::ContextImpl, state::AppDBState};


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
pub mod event_producer;
pub mod mongo_pool;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>>  {
    let config = configuration::Configuration::load().unwrap();
  //  dotenv().ok();

    //Connect with database
    let connection = match Database::connect(config.postgres_url.url).await {
        Ok(connection) => connection,
        Err(e) => panic!("{:?}",e)
    };

    Migrator::up(&connection, None).await?;

    let client = redis::Client::open(config.redis_url.url).unwrap();
    let redis_connection = client.get_connection().unwrap(); 
    let mongo_db_client = Arc::new(mongo_pool::init_db_client(&config.mongo_db).await.unwrap());


    let context = ContextImpl::new_dyn_context(
        mongo_db_client,
        Arc::new(Mutex::new(redis_connection)),
        connection.clone()
    );

  
  let kafka_producer = kafka::init_producer::create_new_kafka_producer(&config.kafka).unwrap();
  let state = AppDBState {conn: connection , from_email: config.email_config.from_email , smtp_key: config.email_config.smtp_key, context: context, producer: kafka_producer };

    let user_auth_routes = routes::user_auth_routes::create_user_routes() ;
    let user_logic_routes = routes::user_logic_routes::create_user_logic_routes();
    let game_routes = routes::game_logic_routes::create_game_routes();
    let routes_all = Router::new()
                            .nest( "/api/v1/auth", user_auth_routes)
                            .nest("/api/v1/user", user_logic_routes)
                            .nest( "/api/v1/game", game_routes)
                            .layer(ServiceBuilder::new()
                                    .layer(CookieManagerLayer::new())
                                    .layer(CorsLayer::permissive()))
                            .with_state(state);


    // run it
    let listener = tokio::net::TcpListener::bind("127.0.0.1:3004")
        .await
        .unwrap();
    println!("listening on {}", listener.local_addr().unwrap());
    axum::serve(listener, routes_all).await.unwrap();

    Ok(())
}
