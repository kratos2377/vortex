
pub mod kafka;
pub mod conf;

fn main() {
    let config = conf::configuration::Configuration::load().unwrap();
    let consumers = kafka::consumer::init_consumers(&config.kafka);
}
