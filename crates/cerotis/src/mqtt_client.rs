use std::time::Duration;

use crate::conf::config_types::MQTTConfiguration;


extern crate paho_mqtt as mqtt;

pub fn create_mqtt_client_for_kafka_consumer(config: &MQTTConfiguration , topic: String) -> mqtt::Client {
    let create_opts = mqtt::CreateOptionsBuilder::new()
    .server_uri(config.dflt_broker.to_string())
    .client_id(config.dflt_client.to_string() + &topic.to_string())
    .delete_oldest_messages(true)
    .finalize();

    let mut cli = mqtt::Client::new(create_opts).unwrap();


    let lwt = mqtt::MessageBuilder::new()
    .topic("connection_test")
    .payload("Consumer lost connection")
    .finalize();
let conn_opts = mqtt::ConnectOptionsBuilder::new()
    .keep_alive_interval(Duration::from_secs(20))
    .clean_session(false)
    .will_message(lwt)
    .finalize();

// Connect and wait for it to complete or fail.
if let Err(e) = cli.connect(conn_opts) {
    println!("Unable to connect:\n\t{:?}", e);

}

//subscribe_topics(&cli , topic.clone(), &config);

cli
}



// fn subscribe_topics(cli: &mqtt::Client , topic_name: String, config: &MQTTConfiguration) {
//     if topic_name == "user" {
//         if let Err(e) = cli.subscribe_many(&config.dflt_topic_user, &config.dflt_qos_user) {
//             println!("Error user subscribes topics: {:?}", e);
       
//         }
//     } else {
//         if let Err(e) = cli.subscribe_many( &config.dflt_topic_game, &config.dflt_qos_game) {
//             println!("Error game subscribes topics: {:?}", e);
       
//         }
//     }
// }