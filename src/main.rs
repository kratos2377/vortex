use std::sync::{Arc, Mutex};

use axum::Router;
use dotenv::dotenv;
use sea_orm::Database;
use socketioxide::SocketIo;
use tower::ServiceBuilder;
use tower_cookies::CookieManagerLayer;
use tower_http::cors::CorsLayer;
use crate::state::AppDBState;


pub mod errors;
pub mod controllers;
pub mod routes;
pub mod models;
pub mod state;
pub mod constants;
pub mod ws_events;
pub mod utils;

#[tokio::main]
async fn main() {
    dotenv().ok();

    //Connect with database
    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let from_email = std::env::var("FROM_EMAIL").expect("FROM_EMAIL must be set");
    let smtp_key = std::env::var("SMTP_KEY").expect("SMTP_KEY must be set");
    let redis_url = std::env::var("REDIS_URL").expect("REDIS_URL must be set");
    let connection = match Database::connect(database_url.to_string()).await {
        Ok(connection) => connection,
        Err(e) => panic!("{:?}",e)
    };

    let client = redis::Client::open(redis_url).unwrap();
    let redis_connection = client.get_connection().unwrap(); 

    let (layer, io) = SocketIo::builder().build_layer();

    io.ns("/", ws_events::game_events::create_ws_events);

    
  //  Migrator::up(&conn, None).await.unwrap();
    let state = AppDBState {conn: connection , from_email: from_email , smtp_key: smtp_key, redis_connection: Arc::new(Mutex::new(redis_connection)) };
    // build our application with a route
    let user_auth_routes = routes::user_auth_routes::create_user_routes() ;
    let game_routes = routes::game_logic_routes::create_game_routes();
    let routes_all = Router::new()
                            .nest( "/api/v1/user", user_auth_routes)
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
}
