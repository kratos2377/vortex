use std::{time::Duration};
use axum::http::Error;
use mongodb::{options::ClientOptions, Client};

use crate::conf::config_types::MongoDatabaseConfiguration;




pub async fn init_db_client(config: &MongoDatabaseConfiguration) -> Result<Client , Error> {
    
    let mut opt = ClientOptions::parse(config.url.clone()).await.unwrap();


    if let Some(max) = config.connection.pool.max {
        opt.max_pool_size = Some(max)
    }
    
    if let Some(min) = config.connection.pool.min {
        opt.min_pool_size = Some(min)
    }
    

    if let Some(connect_timeout) = config.connection.connect_timeout {
        opt.connect_timeout = Some(Duration::from_secs(connect_timeout));
    }
    

    if let Some(idle_timeout) = config.connection.idle_timeout {
        opt.max_idle_time = Some(Duration::from_secs(idle_timeout));
    }

    Ok(Client::with_options(opt).unwrap())
    

}