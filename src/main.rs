use std::{collections::HashMap, sync::{Arc, Mutex}};

use axum::{Router};
use dotenv::dotenv;
use sea_orm::{Database};
use tower_cookies::CookieManagerLayer;
use crate::state::AppDBState;


pub mod errors;
pub mod ctx;
pub mod middlewares;
pub mod controllers;
pub mod routes;
pub mod models;
pub mod state;
pub mod constants;

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
    let mut redis_connection = client.get_connection().unwrap();

    
  //  Migrator::up(&conn, None).await.unwrap();
    let state = AppDBState {conn: connection , from_email: from_email , smtp_key: smtp_key, redis_connection: Arc::new(Mutex::new(redis_connection)),
             rooms: Arc::new(Mutex::new(HashMap::new())) };
    // build our application with a route
    let user_routes = routes::user_routes::create_user_routes() ;
    let game_routes = routes::game_logic_routes::create_game_routes();
    let routes_all = Router::new()
                            .nest( "/api/v1/user", user_routes)
                            .nest( "/api/v1/game", game_routes)
                            .layer(CookieManagerLayer::new())
                            .with_state(state);


    // run it
    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000")
        .await
        .unwrap();
    println!("listening on {}", listener.local_addr().unwrap());
    axum::serve(listener, routes_all).await.unwrap();
}
