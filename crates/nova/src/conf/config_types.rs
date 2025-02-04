use serde::Deserialize;



#[derive(Debug, Deserialize)]
#[allow(unused)]
pub struct ServerConfiguration {
    pub port: u16,
}




#[derive(Debug, Deserialize)]
#[allow(unused)]
pub struct BrokerProperties {
    pub urls: String,
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
