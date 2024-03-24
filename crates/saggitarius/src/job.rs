use std::sync::{Arc, Mutex};

use mongodb::Client;
use rdkafka::producer::FutureProducer;





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