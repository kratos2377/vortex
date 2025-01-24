use std::sync::{Arc, Mutex};

use mongodb::Client;
use sea_orm::DatabaseConnection;



pub type DynContext = Arc<dyn Context>;

pub trait Context: Sync + Send {
    fn get_postgres_db_client(&self) -> DatabaseConnection;
}

#[derive(Clone)]
pub struct ContextImpl {
    pub postgres_db_client: DatabaseConnection,
}

impl ContextImpl {
    pub fn new_dyn_context(
        postgres_db_client: DatabaseConnection,
    ) -> DynContext {
        let context = ContextImpl {
            postgres_db_client: postgres_db_client
        };
        let context: DynContext = Arc::new(context);
        context
    }
}

impl Context for ContextImpl {


    fn get_postgres_db_client(&self) -> DatabaseConnection {
        self.postgres_db_client.clone()
    }
}
