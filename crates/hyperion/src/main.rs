use std::sync::{Arc, Mutex};

use conf::configuration;

pub mod conf;
pub mod kafka;
pub mod executor;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>>  {
    let config = configuration::Configuration::load().unwrap();
  //  dotenv().ok();

  


    Ok(())
}

