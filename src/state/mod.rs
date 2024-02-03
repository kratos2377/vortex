use std::{collections::HashMap, sync::{Arc, Mutex}};

use redis::Connection;
use sea_orm::DatabaseConnection;
use tokio::sync::broadcast;

#[derive(Clone)]
pub struct AppDBState {
    pub conn: DatabaseConnection,
    pub from_email: String,
    pub smtp_key: String,
    pub redis_connection: Arc<Mutex<Connection>>,
    pub rooms: Arc<Mutex<HashMap<String , RoomState>>>,
}


pub struct RoomState {
    tx: broadcast::Sender<String>
}

impl RoomState {
    pub fn new() -> Self {
        Self {
            tx: broadcast::channel(10).0,
        }
    }
}