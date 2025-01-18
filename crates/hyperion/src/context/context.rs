use std::sync::{Arc, Mutex};

use mongodb::Client;
use redis::Connection;
use sea_orm::DatabaseConnection;



pub type DynContext = Arc<dyn Context>;

pub trait Context: Sync + Send {
    fn get_redis_db_client(&self) -> Arc<Mutex<Connection>>;
    fn get_postgres_db_client(&self) -> DatabaseConnection;
}

#[derive(Clone)]
pub struct ContextImpl {
    pub redis_db_client: Arc<Mutex<Connection>>,
    pub postgres_db_client: DatabaseConnection,
}

impl ContextImpl {
    pub fn new_dyn_context(
        redis_db_client: Arc<Mutex<Connection>>,
        postgres_db_client: DatabaseConnection,
    ) -> DynContext {
        let context = ContextImpl {
            redis_db_client: redis_db_client,
            postgres_db_client: postgres_db_client
        };
        let context: DynContext = Arc::new(context);
        context
    }
}

impl Context for ContextImpl {

    fn get_redis_db_client(&self) -> Arc<Mutex<Connection>> {
        self.redis_db_client.clone()
    }

    fn get_postgres_db_client(&self) -> DatabaseConnection {
        self.postgres_db_client.clone()
    }
}
