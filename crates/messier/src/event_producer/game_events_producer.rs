use futures::FutureExt;
use rdkafka::error::KafkaError;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{common::schema_create_user_game_event::SCHEMA_NAME_CREATE_USER_GAME_EVENT, context::context::DynContext, mongo::{kafka_event_models::{UserGameEvent, UserGameMove}, send_kafka_events_to_mongo::create_and_send_kafka_events, transaction::transactional}};



#[derive(Clone , Deserialize , Serialize)]
pub struct GameEventPayload {
    pub user_id: String,
    pub game_event: String,
    pub game_id: String
}



pub async fn send_game_stake_events_request() {
    todo!()
}


pub async fn send_game_move_events(
    context: &DynContext,
    game_event_payload: GameEventPayload,
    socket_id: String,
 ) -> Result<(), KafkaError> {
 
     let input = UserGameEvent {
         id: Uuid::new_v4(),
         version: 1,
         name: game_event_payload.user_id.clone(),
         description: "user_game_move".into(),
         user_game_move: UserGameMove {
             id: Uuid::new_v4(),
             game_id: game_event_payload.game_id,
             user_id: game_event_payload.user_id,
             version: 1,
             user_move: game_event_payload.game_event,
             socket_id,
             
         }
 
 
     };
 
 
     // Start transaction and execute query
     let save_user_game_move_event = transactional(context.get_mongo_db_client(), |db_session| {
         let event_dispatcher = context.get_event_dispatcher();
 
         let user_ws_event: UserGameEvent  = input.clone().into();
 
         async move {
 
             create_and_send_kafka_events(
                 db_session,
                 event_dispatcher,
                 Box::new(user_ws_event.clone()),
                 SCHEMA_NAME_CREATE_USER_GAME_EVENT,
             )
             .await.unwrap();
             Ok(user_ws_event)
         }
         .boxed()
     })
     .await.unwrap();
 
     Ok(())
 
 }
 