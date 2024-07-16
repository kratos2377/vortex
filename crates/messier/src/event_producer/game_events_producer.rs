use std::time::Duration;

use futures::{future, FutureExt};
use orion::{constants::{GAME_GENERAL_EVENT, USER_GAME_MOVE, USER_JOINED_ROOM}, events::kafka_event::KafkaGeneralEvent, models::user_game_event::{UserGameMove}};
use rdkafka::{error::KafkaError, producer::{FutureProducer, FutureRecord, Producer}, util::Timeout};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{common::schema_create_user_game_event::SCHEMA_NAME_CREATE_USER_GAME_EVENT, context::context::DynContext};



#[derive(Clone , Deserialize , Serialize, Debug)]
pub struct GameEventPayload {
    pub user_id: String,
    pub game_event: String,
    pub event_type: String,
    pub game_id: String
}

#[derive(Clone , Deserialize , Serialize)]
pub struct UserReadyEventPayload {
    pub user_id: String,
    pub username: String,
    pub game_id: String
}



pub async fn send_game_general_events(
    event_name: String,
    payload: String,
    producer: &FutureProducer
) -> Result<(), KafkaError> {

    let kafka_events = vec![KafkaGeneralEvent {topic:"game".to_string(), payload: payload.clone(), key: event_name }];

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


pub async fn send_game_move_events(
    context: &DynContext,
    game_event_payload: GameEventPayload,
    socket_id: String,
    producer: &FutureProducer
 ) -> Result<(), KafkaError> {
 
     let input =  UserGameMove {
             game_id: game_event_payload.game_id,
             user_id: game_event_payload.user_id,
             user_move: game_event_payload.game_event,
            move_type: game_event_payload.event_type,
     };
 
     let kafka_events = vec![KafkaGeneralEvent {topic:"user_game_events".to_string(), payload: serde_json::to_string(&input).unwrap(), key: USER_GAME_MOVE.to_string() }];
     // Start transaction and execute query
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
 