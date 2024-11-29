use std::time::Duration;

use orion::{constants::{FRIEND_REQUEST_EVENT, GAME_INVITE_EVENT}, events::kafka_event::KafkaGeneralEvent};
use rdkafka::{error::KafkaError, producer::{FutureProducer, FutureRecord, Producer}, util::Timeout};
use futures_util::future;
use crate::context::context::DynContext;


// Make these events name into const static strings
pub async fn send_event_for_user_topic(
    producer: &FutureProducer,
    context: &DynContext,
    event_name: String,
    payload: String
) -> Result<(), KafkaError> {
    let kafka_events: Vec<KafkaGeneralEvent> = match event_name.as_str() {
        FRIEND_REQUEST_EVENT => {
            create_friend_request_event(context , payload).await
        },
        GAME_INVITE_EVENT => {
            create_game_invite_event(context , payload).await
        },

        _ => {vec![]}

    };

    if kafka_events.len() == 0 {
        return Ok(())
    }

    
   producer.begin_transaction().unwrap();


    let kafka_result = future::try_join_all(kafka_events.iter().map(|event| async move {
        let delivery_result = producer
        .send(
            FutureRecord::to(&event.topic)
                    .payload(&event.payload)
                    .key(&event.key.clone()),
            Duration::from_secs(3),
        )
        .await;

    // This will be executed when the result is received.
  //  println!("Delivery status for message {} received", i);
    delivery_result

    })

    ).await;

    match kafka_result {
        Ok(_) => (),
        Err(e) => return Err(e.0.into()),
    }

    producer.commit_transaction(Timeout::from(Duration::from_secs(5))).unwrap(); 

    Ok(())

}


pub async fn create_friend_request_event(context: &DynContext, payload: String) -> Vec<KafkaGeneralEvent> {
    let kafka_general_event = KafkaGeneralEvent {
        topic: "user".to_string(),
        payload: payload,
        key: "friend-request-event".to_string(),
    };
    let results_resp: Vec<KafkaGeneralEvent> = vec![kafka_general_event];
    return results_resp
}


pub async fn create_game_invite_event(context: &DynContext, payload: String) -> Vec<KafkaGeneralEvent> {
    let kafka_general_event = KafkaGeneralEvent {
        topic: "user".to_string(),
        payload: payload,
        key: "game-invite-event".to_string(),
    };
    let results_resp: Vec<KafkaGeneralEvent> = vec![kafka_general_event];
    return results_resp
}
