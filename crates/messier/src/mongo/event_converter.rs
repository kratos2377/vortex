use async_trait::async_trait;
use mongodb::error::Error;

use super::event_models::{EventDto, SerializableEventDto};


pub type DynEventConverter = Box<dyn EventConverter>;

#[async_trait]
pub trait EventConverter: Sync + Send {
    fn handles(&self, event_type: String) -> bool;

    #[allow(clippy::borrowed_box)]
    async fn handle(
        &self,
        event_type: String,
        event: &Box<dyn SerializableEventDto>,
    ) -> Result<EventDto, Error>;
}

pub fn handles(converter: &DynEventConverter, event_type: String) -> bool {
    converter.handles(event_type)
}
