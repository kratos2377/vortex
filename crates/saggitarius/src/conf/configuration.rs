use std::{env, sync::atomic::AtomicU16};
use std::sync::atomic::Ordering::SeqCst;
use config::{Config, ConfigError, File};
use serde::Deserialize;

use super::config_types::{KafkaConfiguration, MongoDatabaseConfiguration, ServerConfiguration};


pub static SERVER_PORT: AtomicU16 = AtomicU16::new(0);

#[derive(Debug, Deserialize)]
#[allow(unused)]
pub struct SaggitariusConfiguration {
    pub mongo_database: MongoDatabaseConfiguration,
    pub kafka: KafkaConfiguration,
    pub server: ServerConfiguration, 
}

impl SaggitariusConfiguration {
    pub fn load() -> Result<Self, ConfigError> {
        let profiles_raw_string = env::var("RUST_PROFILES_ACTIVE").unwrap_or_default();
        let active_profiles: Vec<&str> = profiles_raw_string
            .split(',')
            .into_iter()
            .map(|p| p.trim())
            .filter(|p| !(*p).is_empty())
            .collect();

        // Load always properties of application.yml
        let mut builder =
            Config::builder().add_source(File::with_name("resources/application.yml"));

        // Load property files for profiles
        for profile in active_profiles {
            builder = builder.add_source(
                File::with_name(&format!("resources/application-{}.yml", profile)).required(false),
            );
        }

        let parsed_config: Result<SaggitariusConfiguration, ConfigError> = builder.build().unwrap().try_deserialize();

        // Set server port statically
        if let Ok(config) = &parsed_config {
            SERVER_PORT.store(config.server.port, SeqCst);
        }

        // Return config
        parsed_config
    }
}
