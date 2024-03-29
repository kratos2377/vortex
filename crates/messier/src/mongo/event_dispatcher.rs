use std::sync::Arc;

use mongodb::error::Error;
use tracing::instrument;

use crate::mongo::event_converter::handles;

use super::{event_converter::DynEventConverter, event_models::{EventDto, SerializableEventDto}};


pub struct EventDispatcher {
    pub(crate) event_converters: Vec<Arc<DynEventConverter>>,
}

impl EventDispatcher {
    pub fn new(event_converters: Vec<Arc<DynEventConverter>>) -> Self {
        EventDispatcher { event_converters }
    }

    #[instrument(name = "event_dispatcher.dispatch", skip_all)]
    pub async fn dispatch(
        &self,
        event_type: String,
        event: Box<dyn SerializableEventDto>,
    ) -> Result<Vec<EventDto>, Error> {
        let mut dtos: Vec<EventDto> = Vec::new();

        let mut handled = false;
        for converter in self.event_converters.clone().into_iter() {
            if handles(&converter, event_type.clone()) {
                handled = true;
                dtos.push(converter.handle(event_type.clone(), &event).await?);
            }
        }
        assert!(handled);

        Ok(dtos)
    }
}
