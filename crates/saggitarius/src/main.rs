

pub mod conf;
pub mod job;
pub mod mongo;
pub mod context;
pub mod schedule;
pub mod event;

#[tokio::main]
async fn main() {

    let config = conf::configuration::SaggitariusConfiguration::load();

    println!("Hello, world!");
}
