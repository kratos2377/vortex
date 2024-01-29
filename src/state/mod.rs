use std::sync::{Arc, Mutex};

use redis::Connection;
use sea_orm::DatabaseConnection;

#[derive(Clone)]
pub struct AppDBState {
    pub conn: DatabaseConnection,
    pub from_email: String,
    pub smtp_key: String,
    pub redis_connection: Arc<Mutex<Connection>>,
}
