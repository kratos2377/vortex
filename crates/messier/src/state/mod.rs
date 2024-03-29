use rdkafka::producer::FutureProducer;
use sea_orm::DatabaseConnection;

use crate::context::context::DynContext;

#[derive(Clone)]
pub struct AppDBState {
    pub conn: DatabaseConnection,
    pub from_email: String,
    pub smtp_key: String,
    pub producer: FutureProducer,
    pub context: DynContext
}


#[derive(Clone)]
pub struct WebSocketStates {
    pub producer: FutureProducer,
    pub context: DynContext
}
