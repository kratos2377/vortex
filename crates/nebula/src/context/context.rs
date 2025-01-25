use std::sync::{Arc, Mutex};

use redis::aio::MultiplexedConnection;
use reqwest::Client;
use sea_orm::DatabaseConnection;




pub type DynContext = Arc<dyn Context>;

pub trait Context: Sync + Send {
    fn get_postgres_db_client(&self) -> DatabaseConnection;
    fn get_redis_connection(&self) -> MultiplexedConnection;
    fn get_reqwest_client(&self) -> Client;
}

#[derive(Clone)]
pub struct ContextImpl {
    pub postgres_db_client: DatabaseConnection,
    pub redis_conn: MultiplexedConnection,
    pub reqwest_client: Client
}

impl ContextImpl {
    pub fn new_dyn_context(
        postgres_db_client: DatabaseConnection,
        redis_conn: MultiplexedConnection,
        reqwest_client: Client,
    ) -> DynContext {
        let context = ContextImpl {
            postgres_db_client: postgres_db_client,
            redis_conn: redis_conn,
            reqwest_client: reqwest_client
        };
        let context: DynContext = Arc::new(context);
        context
    }
}

impl Context for ContextImpl {

    fn get_redis_connection(&self) -> MultiplexedConnection {
        self.redis_conn.clone()
    }
    
    fn get_postgres_db_client(&self) -> DatabaseConnection {
        self.postgres_db_client.clone()
    }

    fn get_reqwest_client(&self) -> Client {
        self.reqwest_client.clone()
    }
}
