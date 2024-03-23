use std::sync::{Arc, Mutex};

use mongodb::Client;
use redis::Connection;


pub type DynContext = Arc<dyn Context>;

pub trait Context: Sync + Send {
    fn get_mongo_db_client(&self) -> Arc<Client>;
    fn get_redis_db_client(&self) -> Arc<Mutex<Connection>>;
}

#[derive(Clone)]
pub struct ContextImpl {
    pub mongo_db_client: Arc<Client>,
    pub redis_db_client: Arc<Mutex<Connection>>,
}

impl ContextImpl {
    pub fn new_dyn_context(
        mongo_db_client: Arc<Client>,
        redis_db_client: Arc<Mutex<Connection>>,
    ) -> DynContext {
        let context = ContextImpl {
            mongo_db_client: mongo_db_client,
            redis_db_client: redis_db_client,
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
}
