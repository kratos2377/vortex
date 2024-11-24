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