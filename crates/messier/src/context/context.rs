use std::sync::{Arc, Mutex};

use mongodb::Client;
use redis::Connection;
use sea_orm::DatabaseConnection;



pub type DynContext = Arc<dyn Context>;

pub trait Context: Sync + Send {
    fn get_mongo_db_client(&self) -> Arc<Client>;
    fn get_redis_db_client(&self) -> Arc<Mutex<Connection>>;
    fn get_postgres_db_client(&self) -> DatabaseConnection;
}

#[derive(Clone)]
pub struct ContextImpl {
    pub mongo_db_client: Arc<Client>,
    pub redis_db_client: Arc<Mutex<Connection>>,
    pub postgres_db_client: DatabaseConnection,
}

impl ContextImpl {
    pub fn new_dyn_context(
        mongo_db_client: Arc<Client>,
        redis_db_client: Arc<Mutex<Connection>>,
        postgres_db_client: DatabaseConnection,
    ) -> DynContext {
        let context = ContextImpl {
            mongo_db_client: mongo_db_client,
            redis_db_client: redis_db_client,
            postgres_db_client: postgres_db_client
        };
        let context: DynContext = Arc::new(context);
        context
    }
}

impl Context for ContextImpl {
    fn get_mongo_db_client(&self) -> Arc<Client> {
        self.mongo_db_client.clone()
    }


    fn get_redis_db_client(&self) -> Arc<Mutex<Connection>> {
        self.redis_db_client.clone()
    }

    fn get_postgres_db_client(&self) -> DatabaseConnection {
        self.postgres_db_client.clone()
    }
}
