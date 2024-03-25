use std::sync::{Arc};

use futures::FutureExt;
use mongodb::{error::Error, Client};
use rdkafka::producer::FutureProducer;
use tokio::sync::Mutex;

use crate::{event::service::event_service, mongo::transaction::transactional};

pub async fn poll_and_send(
    job_synchronization_mutex: Arc<Mutex<bool>>,
    client: Arc<Client>,
    producer: Arc<FutureProducer>,
) -> Result<() , Error> {
    let lock = job_synchronization_mutex.try_lock();

    if lock.is_ok() {
        let mut more_events_to_send = true;
        while more_events_to_send {
            more_events_to_send =
                find_send_delete(client.clone(), producer.clone())
                    .await.unwrap();
        }
    }

    Ok(())
}


async fn find_send_delete(
    client: Arc<Client>,
    producer: Arc<FutureProducer>,
) -> Result<bool, Error> {
    transactional(client, |db_session| {
        let producer = producer.clone();

        async move {
            let event_list = event_service::find_next_page(db_session).await;

            match event_list {
                Ok(events) => {
                    // Skip further processing if there are no events to send
                    if events.events.is_empty() {
                        return Ok(false);
                    }

                    // Send data
                    event_service::send_to_kafka(
                        producer.clone(),
                        &events,
                    )
                    .await.unwrap();

                    // Delete sent events
                    event_service::delete_from_db(db_session, &events).await?;

                    // Send signal to continue without waiting
                    Ok(events.has_more)
                }
                Err(e) => {
                    Ok(false)
                }
            }
        }
        .boxed()
    })
    .await
}
