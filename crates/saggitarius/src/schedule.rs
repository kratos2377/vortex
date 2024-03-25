
use std::sync::Arc;
use std::time::Duration;

use futures::FutureExt;
use mongodb::error::Error;
use rdkafka::producer::FutureProducer;
use tokio::sync::Mutex;
use tokio_cron_scheduler::{Job, JobScheduler};
use crate::context::DynContext;
use crate::job;




pub async fn start_and_run_scheduled_job(
    context: DynContext,
    producer: Arc<FutureProducer>
) -> Result<(), Error> {

    let sync_job_mutex = Arc::new(Mutex::new(false));

    let job = Job::new_repeated_async(Duration::from_secs(1), move |_job_id, _lock| {
        let job_synchronization_mutex = sync_job_mutex.clone();
        let producer = producer.clone();
        let db_client = context.db_client();

        async move {
            job::poll_and_send(
                job_synchronization_mutex,
                db_client,
                producer.clone(),
            )
            .await
            .expect("Scheduled job failed");
        }
        .boxed()
    }).unwrap();

    let scheduler = JobScheduler::new().await.unwrap();
    scheduler.add(job).await.unwrap();

    #[cfg(feature = "signal")]
    scheduler.shutdown_on_ctrl_c();

    scheduler.start().await.unwrap();

    Ok(())
}