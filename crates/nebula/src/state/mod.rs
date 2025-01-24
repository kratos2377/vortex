use redis::aio::Connection;
use sea_orm::DatabaseConnection;
use std::sync::{Arc, Mutex};
#[derive(Clone)]
pub struct AppDBState {
    pub conn: DatabaseConnection,
}
