
use serde;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
#[allow(unused)]
pub struct KafkaConfiguration {
    pub broker: BrokerProperties,
    pub consumer: Vec<ConsumerConfiguration>,
    pub schema_registry: SchemaRegistryProperties,
    pub topic: TopicConfiguration,
}

#[derive(Debug, Deserialize)]
#[allow(unused)]
pub struct BrokerProperties {
    pub urls: String,
}

#[derive(Debug, Deserialize)]
#[allow(unused)]
pub struct ConsumerConfiguration {
    pub id: String,
    pub topic: Vec<String>,
    pub client_id: String,
    pub group_id: String,
}

#[derive(Debug, Deserialize)]
#[allow(unused)]
pub struct SchemaRegistryProperties {
    pub url: String,
}

#[derive(Clone, Debug, Deserialize)]
#[allow(unused)]
pub struct TopicConfiguration {
    pub mappings: Vec<TopicProperties>,
}

impl TopicConfiguration {
    pub fn get_mapping(&self, id: &str) -> TopicProperties {
        let mapping: Vec<TopicProperties> = self
            .mappings
            .clone()
            .into_iter()
            .filter(|t| t.id == id)
            .collect();

        mapping
            .first()
            .unwrap_or_else(|| panic!("{} topic configuration not found", id))
            .clone()
    }
}

#[derive(Clone, Debug, Deserialize)]
#[allow(unused)]
pub struct TopicProperties {
    pub id: String,
    pub topic_name: String,
    pub partitions: i32,
}

#[derive(Debug, Deserialize)]
#[allow(unused)]
pub struct LoggingConfiguration {
    pub level: LogLevelConfiguration,
}

#[derive(Debug, Deserialize)]
#[allow(unused)]
pub struct LogLevelConfiguration {
    pub root: Option<String>,
    pub directives: Vec<LoggingDirective>,
}

#[derive(Debug, Deserialize)]
#[allow(unused)]
pub struct LoggingDirective {
    pub namespace: String,
    pub level: String,
}

#[derive(Debug, Deserialize)]
#[allow(unused)]
pub struct ServerConfiguration {
    pub port: u16,
}

#[derive(Debug, Deserialize)]
#[allow(unused)]
pub struct MongoDatabaseConfiguration {
    pub url: String,
    pub connection: MongoDatabaseConnectionProperties,
}

#[derive(Debug, Deserialize)]
#[allow(unused)]
pub struct MongoDatabaseConnectionProperties {
    pub pool: MongoDatabasePoolProperties,
    pub connect_timeout: Option<u64>,
    pub idle_timeout: Option<u64>,
}

#[derive(Debug, Deserialize)]
#[allow(unused)]
pub struct MongoDatabasePoolProperties {
    pub min: Option<u32>,
    pub max: Option<u32>,
}

