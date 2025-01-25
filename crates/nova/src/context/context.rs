use std::sync::{Arc, Mutex};

use redis::aio::MultiplexedConnection;




pub type DynContext = Arc<dyn Context>;

pub trait Context: Sync + Send {
    fn get_redis_connection(&self) -> MultiplexedConnection;
}

#[derive(Clone)]
pub struct ContextImpl {
    pub redis_conn: MultiplexedConnection
}

impl ContextImpl {
    pub fn new_dyn_context(
        redis_conn: MultiplexedConnection,
    ) -> DynContext {
        let context = ContextImpl {
            redis_conn: redis_conn
        };
        let context: DynContext = Arc::new(context);
        context
    }
}

impl Context for ContextImpl {

    fn get_redis_connection(&self) -> MultiplexedConnection {
        self.redis_conn.clone()
    }
    
}
