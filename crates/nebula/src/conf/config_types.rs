use serde::Deserialize;



#[derive(Debug, Deserialize)]
#[allow(unused)]
pub struct ServerConfiguration {
    pub port: u16,
}


#[derive(Debug, Deserialize)]
#[allow(unused)]
pub struct PostgresDatabaseUrl {
    pub url: String,
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

#[derive(Clone, Debug, Deserialize)]
#[allow(unused)]
pub struct TopicProperties {
    pub id: String,
    pub topic_name: String,
    pub partitions: i32,
}

#[derive(Debug, Deserialize)]
#[allow(unused)]
pub struct KafkaConfiguration {
    pub broker: BrokerProperties,
    pub consumer: Vec<ConsumerConfiguration>,
    pub schema_registry: SchemaRegistryProperties,
    pub topic: TopicConfiguration,
    pub producer: ProducerProperties,
}

#[derive(Debug, Deserialize)]
#[allow(unused)]
pub struct ProducerProperties {
    pub client_id: String,
    pub transactional_id: String,
}


#[derive(Debug, Deserialize)]
#[allow(unused)]
pub struct RedisConfiguration {
    pub url: String,
}
