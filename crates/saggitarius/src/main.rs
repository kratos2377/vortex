use std::{net::SocketAddr, sync::Arc};

use axum::{http::Error, response::IntoResponse, routing::get, Router};
use conf::config_types::ServerConfiguration;
use kafka::init_producer;
use schedule::start_and_run_scheduled_job;
use serde_json::json;
use tokio::task::JoinHandle;

use crate::{context::ContextImpl, mongo::pool};



pub mod conf;
pub mod job;
pub mod mongo;
pub mod context;
pub mod schedule;
pub mod event;
pub mod kafka;

#[tokio::main]
async fn main() -> Result<() , Error>{

    let config = conf::configuration::SaggitariusConfiguration::load().unwrap();
    let db_client = Arc::new(pool::init_db_client(&config.mongo_database).await.unwrap());
    let context = ContextImpl::new_dyn_context(db_client);

    let producer = Arc::new(init_producer(&config.kafka).unwrap());

    // Run scheduled job to poll events from database and send them to kafka
    start_and_run_scheduled_job(context, producer).await.unwrap();

    // Start the web-server
    start_web_server(&config.server).await;

    Ok(())
}


async fn start_web_server(config: &ServerConfiguration) {
    // Initialize routing
    let routing = init_routing();

    let addr = SocketAddr::from(([0, 0, 0, 0], config.port));


    let listener = tokio::net::TcpListener::bind("127.0.0.1:3003")
    .await
    .unwrap();
println!("listening on {}", listener.local_addr().unwrap());
axum::serve(listener, routing.into_make_service_with_connect_info::<SocketAddr>()).with_graceful_shutdown(shutdown_signal()).await.unwrap();

    // Shutdown tracing provider
}


pub async fn shutdown_signal() {
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
}


fn init_routing() -> Router {
    Router::new()
        .route("/health", get(health))
}

pub async fn health() -> impl IntoResponse {
    axum::Json(json!({ "status" : "UP" }))
}
