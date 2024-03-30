use mongodb::{error::Error, ClientSession};
use std::sync::Arc;

use crate::mongo::event_service;

use super::{event_dispatcher::EventDispatcher, event_models::SerializableEventDto};




pub async fn create_and_send_kafka_events(
    db_session: &ClientSession,
    event_dispatcher: Arc<EventDispatcher>,
    dto: Box<dyn SerializableEventDto>,
    event_type: &str,
) -> Result<(), Error> {
    let events = event_dispatcher
.dispatch(event_type.to_string(), dto)
.await?;

assert!(!events.is_empty());

for event in events {
event_service::save(db_session, &event).await?;
}
Ok(())
}