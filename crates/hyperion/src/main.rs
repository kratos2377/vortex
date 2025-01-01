use std::sync::{Arc, Mutex};

use conf::configuration;
use context::context::ContextImpl;
use sea_orm::Database;


pub mod mongo_pool;
pub mod conf;
pub mod context;
pub mod kafka;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>>  {
    let config = configuration::Configuration::load().unwrap();
  //  dotenv().ok();

    //Connect with database
    let connection = match Database::connect(config.postgres_url.url).await {
        Ok(connection) => connection,
        Err(e) => panic!("{:?}",e)
    };


    let client = redis::Client::open(config.redis_url.url).unwrap();
    let redis_connection = client.get_connection().unwrap(); 
    let mongo_db_client = Arc::new(mongo_pool::init_db_client(&config.mongo_db).await.unwrap());


    //Replace mutex redis connection

    let context = ContextImpl::new_dyn_context(
        mongo_db_client,
        Arc::new(Mutex::new(redis_connection)),
        connection.clone()
    );


    Ok(())
}

