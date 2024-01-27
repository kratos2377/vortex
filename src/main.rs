use axum::{Router};

pub mod errors;
pub mod ctx;
pub mod middlewares;

#[tokio::main]
async fn main() {
    // build our application with a route
    let routes_all = Router::new();

    // run it
    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000")
        .await
        .unwrap();
    println!("listening on {}", listener.local_addr().unwrap());
    axum::serve(listener, routes_all).await.unwrap();
}
