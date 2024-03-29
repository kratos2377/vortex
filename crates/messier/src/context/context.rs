use std::sync::{Arc, Mutex};

use mongodb::Client;
use redis::Connection;

use crate::mongo::event_dispatcher::EventDispatcher;


pub type DynContext = Arc<dyn Context>;

pub trait Context: Sync + Send {
    fn get_mongo_db_client(&self) -> Arc<Client>;
    fn get_redis_db_client(&self) -> Arc<Mutex<Connection>>;
    fn get_event_dispatcher(&self) -> Arc<EventDispatcher>;
}

#[derive(Clone)]
pub struct ContextImpl {
    pub mongo_db_client: Arc<Client>,
    pub redis_db_client: Arc<Mutex<Connection>>,
    pub event_dispatcher: Arc<EventDispatcher>,
}

impl ContextImpl {
    pub fn new_dyn_context(
        mongo_db_client: Arc<Client>,
        redis_db_client: Arc<Mutex<Connection>>,
        event_dispatcher: Arc<EventDispatcher>,
    ) -> DynContext {
        let context = ContextImpl {
            mongo_db_client: mongo_db_client,
            redis_db_client: redis_db_client,
            event_dispatcher: event_dispatcher,
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

    fn get_event_dispatcher(&self) -> Arc<EventDispatcher> {
        self.event_dispatcher.clone()
    }
}
