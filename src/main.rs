use axum::{http::StatusCode, Router};
use dotenv::dotenv;
use sea_orm::{Database, DatabaseConnection};
use tower_cookies::CookieManagerLayer;

use crate::state::AppDBState;


pub mod errors;
pub mod ctx;
pub mod middlewares;
pub mod controllers;
pub mod routes;
pub mod models;
pub mod state;


#[tokio::main]
async fn main() {
    dotenv().ok();

    //Connect with database
    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let connection = match Database::connect(database_url.to_string()).await {
        Ok(connection) => connection,
        Err(e) => panic!("{:?}",e)
    };

    
  //  Migrator::up(&conn, None).await.unwrap();
    let state = AppDBState {conn: connection};
    // build our application with a route
    let user_routes = routes::user_routes::create_user_routes() ;
 //   let game_routes = ;
    let routes_all = Router::new()
                            .nest( "/api/v1", user_routes)
                            .layer(CookieManagerLayer::new())
                            .with_state(state);


    // run it
    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000")
        .await
        .unwrap();
    println!("listening on {}", listener.local_addr().unwrap());
    axum::serve(listener, routes_all).await.unwrap();
}
