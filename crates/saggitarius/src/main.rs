

pub mod conf;
pub mod job;
pub mod mongo;

#[tokio::main]
async fn main() {

    let config = conf::configuration::SaggitariusConfiguration::load();

    println!("Hello, world!");
}
