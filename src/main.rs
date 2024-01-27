use axum::{Router};

pub mod errors;
pub mod ctx;
pub mod middlewares;
pub mod controllers;
pub mod routes;
pub mod models;

#[tokio::main]
async fn main() {
    // build our application with a route
    let user_routes = routes::user_routes::create_user_routes() ;
 //   let game_routes = ;
    let routes_all = Router::new()
                            .merge(user_routes);


    // run it
    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000")
        .await
        .unwrap();
    println!("listening on {}", listener.local_addr().unwrap());
    axum::serve(listener, routes_all).await.unwrap();
}
