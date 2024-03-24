use std::{future::Future, pin::Pin, sync::Arc};

use mongodb::{error::{Error, UNKNOWN_TRANSACTION_COMMIT_RESULT}, options::{Acknowledgment, ReadConcern, TransactionOptions, WriteConcern}, Client, ClientSession};





pub async fn transactional<R,F>(db_client: Arc<Client>, f: F) -> Result<R, Error> 
where R: 'static,
      F: for<'c> Fn(
        &'c ClientSession,
      ) -> Pin<Box<dyn Future<Output = Result<R,Error>> + Send + 'c>>,
{
    let mut db_session = db_client.as_ref().start_session(None).await.unwrap();

    let options = TransactionOptions::builder().read_concern(ReadConcern::majority())
                                                .write_concern(WriteConcern::builder().w(Acknowledgment::Majority).build())
                                                .build();
    

    db_session.start_transaction(options).await.unwrap();

    let result = f(&db_session).await;

    if result.is_ok() {
        commit_transaction(&mut db_session).await.unwrap();
    } else {
        rollback_transaction(&mut db_session).await.unwrap();
    }

    result
}

pub async fn commit_transaction<'a>(db_session: &mut ClientSession) -> Result<(), Error> {
    loop {
        let result = db_session.commit_transaction().await;
        if let Err(error) = result {
            if !error.contains_label(UNKNOWN_TRANSACTION_COMMIT_RESULT) {
                return Err(error.into());
            }
        } else {
            return Ok(());
        }
    }
}

pub async fn rollback_transaction<'a>(db_session: &mut ClientSession) -> Result<(), Error> {
    Ok(db_session.abort_transaction().await?)
}
