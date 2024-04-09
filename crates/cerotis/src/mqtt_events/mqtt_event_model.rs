use serde::Serialize;
use serde::Deserialize;


#[derive(Serialize , Deserialize , Clone , Debug)]
pub struct MQTTEventModel {
    pub event_name: String,
    pub payload: String,
}