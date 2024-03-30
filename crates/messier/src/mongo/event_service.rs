use mongodb::error::Error;
use serde::Serialize;
use mongodb::options::InsertOneOptions;
use mongodb::{ClientSession, Collection};
use tracing::instrument;
use opentelemetry::trace::TraceContextExt;
use tracing_opentelemetry::OpenTelemetrySpanExt;

use super::event_models::EventDto;




#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct Model {
    pub topic: String,
    pub partition: i32,
    #[serde(with = "serde_bytes")]
    pub key: Vec<u8>,
    #[serde(with = "serde_bytes")]
    pub payload: Vec<u8>,
    pub trace_id: Option<String>,
}


#[instrument(name = "event.service.save", skip_all)]
pub async fn save(db_session: &ClientSession, event: &EventDto) -> Result<(), Error> {
    let trace_id = get_b3_trace_id();

    // Build the entity from dto
    let e = Model {
        key: event.key.clone(),
        payload: event.payload.clone(),
        partition: event.partition,
        topic: event.topic.clone(),
        trace_id,
    };

    tracing::debug!("Save event to topic: {:?}", e.topic);

    // Save entity
    get_collection(db_session, "event")
        .insert_one(e, InsertOneOptions::default())
        .await?;

    Ok(())
}


pub fn get_collection<T>(db_session: &ClientSession, collection_name: &str) -> Collection<T> {
    let database = db_session
        .client()
        .default_database()
        .expect("No default db specified");

    database.collection::<T>(collection_name)
}


pub fn get_b3_trace_id() -> Option<String> {
    // Code partially taken from
    // axum_tracing_opentelemetry::find_current_trace_id();

    let context = tracing::Span::current().context();
    let span = context.span();
    let span_context = span.span_context();

    let span_id = span_context
        .is_valid()
        .then(|| span_context.span_id().to_string());

    let trace_id = span_context
        .is_valid()
        .then(|| span_context.trace_id().to_string());

    if trace_id.is_none() || span_id.is_none() {
        None
    } else {
        // https://github.com/openzipkin/b3-propagation
        Some(format!("{}-{}-1", trace_id.unwrap(), span_id.unwrap()))
    }
}
