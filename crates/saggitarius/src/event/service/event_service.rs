use std::sync::Arc;
use std::time::Duration;

use anyhow::Result;
use futures::future;
use futures::TryStreamExt;
use mongodb::bson::doc;
use mongodb::bson::oid::ObjectId;
use mongodb::error::Error;
use mongodb::options::DeleteOptions;
use mongodb::options::FindOptions;
use mongodb::ClientSession;
use rdkafka::message::OwnedHeaders;
use rdkafka::producer::FutureProducer;
use rdkafka::producer::FutureRecord;
use rdkafka::producer::Producer;
use rdkafka::util::Timeout;

use crate::mongo::util::get_collection;

use super::super::model::event::Event;
pub const MAX_PAGE_SIZE: usize = 500;

pub async fn find_next_page(db_session: &ClientSession) -> Result<EventList> {
    let page_size = MAX_PAGE_SIZE + 1;
    let filter = doc! {};

    let cursor = get_collection::<Event>(db_session, "event")
        .find(
            filter,
            FindOptions::builder().limit(Some(page_size as i64)).build(),
        )
        .await?;

    let events: Vec<Event> = cursor.try_collect().await?;

    let event_list = EventList {
        has_more: events.len() > MAX_PAGE_SIZE,
        events: events.into_iter().take(MAX_PAGE_SIZE).collect(),
    };

    Ok(event_list)
}

pub async fn delete_from_db(
    db_session: &ClientSession,
    events: &EventList,
) -> Result<(), Error> {
    let event_ids: Vec<ObjectId> = events
        .events
        .iter()
        .take(MAX_PAGE_SIZE)
        .map(|e| e._id)
        .collect();

    let filter = doc! {
        "_id": { "$in" : event_ids }
    };

    get_collection::<Event>(db_session, "event")
        .delete_many(filter, DeleteOptions::default())
        .await?;

    Ok(())
}

pub async fn send_to_kafka(
    producer: Arc<FutureProducer>,
    events: &EventList,
) -> Result<(), Error> {
    let events_to_send = &events.events;
    let number_of_events = events_to_send.len();

    if number_of_events > 0 {
        let mut event_ids: Vec<ObjectId> = Vec::new();


        // Start kafka transaction
        producer.begin_transaction().unwrap();

        // Send each event individually. Send a span for each message to jaeger.
        let send_result = future::try_join_all(events_to_send.iter().map(|event| {
            let trace_id = event.trace_id.clone();

            // Initialize span
            // let span = span!(Level::TRACE, "send");
            // let _ = span.enter();

            // if let Some(id) = trace_id.clone() {
            //     span.set_parent_from_b3(tracing_propagator.clone(), id);
            // }

            // Create kafka headers with trace_id
            let headers = match trace_id {
                Some(id) => OwnedHeaders::new_with_capacity(1).add(B3_SINGLE_HEADER, &id),
                _ => OwnedHeaders::default(),
            };

            // Send message to kafka
            // Instrumented<OwnedDeliveryResult>
            let delivery_result = {
                producer.send(
                    FutureRecord::to(&event.topic)
                        .payload(&event.payload)
                        .partition(event.partition)
                        .key(&event.key)
                        .headers(headers),
                    Duration::from_secs(0),
                )
            };

            // delivery_result.into_inner().expect("Couldn't send event");
            event_ids.push(event._id);

            delivery_result
        }))
        .await;

        match send_result {
            Ok(_) => (),
            Err(e) => return Err("Error Occured while publishing event"),
        }

        // Commit kafka transaction
        producer.commit_transaction(Timeout::from(Duration::from_secs(30))).unwrap();

      //  info!("Sent {} events", number_of_events);
    }

    Ok(())
}

pub struct EventList {
    pub has_more: bool,
    pub events: Vec<Event>,
}
