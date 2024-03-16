use serde::Deserialize;

#[derive(Clone, Debug, Eq, PartialEq, Deserialize)]
pub struct Event {
    pub _id: String,
    pub topic: String,
    pub partition: i32,
    #[serde(with = "serde_bytes")]
    pub key: Vec<u8>,
    #[serde(with = "serde_bytes")]
    pub payload: Vec<u8>,
    pub event_name: String,
}


pub struct EventList {
    pub has_more: bool,
    pub events: Vec<Event>,
}
