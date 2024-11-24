use sea_orm::DatabaseConnection;
use std::sync::{Arc, Mutex};

use mongodb::Client;

#[derive(Clone)]
pub struct AppDBState {
    pub conn: DatabaseConnection,
    pub mongo_conn: Arc<Client>
}
