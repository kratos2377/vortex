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

