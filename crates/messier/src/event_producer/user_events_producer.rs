use std::{str::FromStr, time::Duration};

use orion::{constants::{FRIEND_REQUEST_EVENT, GAME_INVITE_EVENT, USER_JOINED_ROOM, USER_LEFT_ROOM, USER_ONLINE_EVENT, USER_STATUS_EVENT, VERIFYING_GAME_STATUS}, events::{kafka_event::{KafkaGeneralEvent, UserFriendRequestKafkaEvent, UserGameInviteKafkaEvent, UserOnlineKafkaEvent}, ws_events::{JoinedRoomPayload, UserConnectionEventPayload}}};
use rdkafka::{error::KafkaError, producer::{FutureProducer, FutureRecord, Producer}, util::Timeout};
use redis::{Commands, RedisResult};
use sea_orm::TryIntoModel;
use futures_util::future;
use ton::models::{self, users, users_wallet_keys};
use models::{users_friends_requests::{self , Entity as UsersFriendsRequests}, users_friends::{self, Entity as UsersFriends}, users::{Entity as Users}};
use uuid::Uuid;
use crate::context::context::DynContext;


// Make these events name into const static strings
pub async fn send_event_for_user_topic(
    producer: &FutureProducer,
    context: &DynContext,
    event_name: String,
    payload: String
) -> Result<(), KafkaError> {
    let kafka_events: Vec<KafkaGeneralEvent> = match event_name.as_str() {
        USER_ONLINE_EVENT => {
            create_user_online_events(context , payload).await
        },
        FRIEND_REQUEST_EVENT => {
            create_friend_request_event(context , payload).await
        },
        GAME_INVITE_EVENT => {
            create_game_invite_event(context , payload).await
        },
        USER_JOINED_ROOM => {
            create_user_joined_room_event(context , payload).await
        },

        USER_LEFT_ROOM => {
            create_user_leaved_room_event(context , payload).await
        },

        USER_STATUS_EVENT => {
            create_user_status_room_event(context , payload).await
        },

        VERIFYING_GAME_STATUS => {
            vec![]
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

// pub async fn create_user_general_event(event_name: &str, context: &DynContext, payload: String ) -> Vec<KafkaGeneralEvent> {
//    let new_kafka_event = KafkaGeneralEvent {
//         topic: "user".to_string(),
//         payload: serde_json::to_string(&new_user_online_kafka_event).unwrap(),
//         key: event_name.to_string(),
//     };

//     return vec![new_kafka_event]
   
// }

pub async fn create_user_online_events(context: &DynContext, payload: String) -> Vec<KafkaGeneralEvent> {
    let data: UserConnectionEventPayload = serde_json::from_str(&payload).unwrap();
    let connection = context.get_postgres_db_client();
    let rsp = users_friends::Entity::find_by_user_id(&Uuid::from_str(&data.user_id).unwrap()).all(&connection).await;

    if rsp.is_err() {
        return Vec::new()
    }


    let result = rsp.unwrap();

    let arc_redis_client = context.get_redis_db_client();

    let mut results_resp: Vec<KafkaGeneralEvent> = vec![];
   
       // Only add keys if the user_id key is present in redis cluster
       // But we might have to use async redis because not able to pass data safely in threads in single thread
       
           for mo in result.iter() {
        let user_friend: users_friends::Model = mo.clone().try_into_model().unwrap();
               
   
       
           let friend_result = match Users::find_by_id(user_friend.friend_id)
               .one(&connection)
               .await
           {
               Ok(data) => data,
               Err(err) => continue, // Skip to the next iteration on error
           };

           let user_type_details = friend_result.unwrap().try_into_model().unwrap();

           let mut redisConnection  = arc_redis_client.lock().unwrap();
           let does_friend_key_exist_in_redis: RedisResult<String> = redisConnection.hkeys(user_type_details.id.to_string().clone());

           if does_friend_key_exist_in_redis.is_err() || does_friend_key_exist_in_redis.unwrap() != "online" {
            continue;
           }

            let new_user_online_kafka_event = UserOnlineKafkaEvent {
                user_who_came_online_id: data.user_id.clone(),
                user_who_came_online_username: data.username.clone(),
                user_who_we_are_sending_event: user_type_details.id.to_string(),
            };
               let new_kafka_event = KafkaGeneralEvent {
                topic: "user".to_string(),
                payload: serde_json::to_string(&new_user_online_kafka_event).unwrap(),
                key: "user-online-event".to_string(),
            };
           

          

           results_resp.push(new_kafka_event);
    
       
           }

           return results_resp

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

pub async fn create_user_joined_room_event(context: &DynContext, payload: String) -> Vec<KafkaGeneralEvent> {
    let kafka_general_event = KafkaGeneralEvent {
        topic: "user".to_string(),
        payload: payload,
        key: "user-joined-room".to_string(),
    };
    let results_resp: Vec<KafkaGeneralEvent> = vec![kafka_general_event];
    return results_resp
}

pub async fn create_user_leaved_room_event(context: &DynContext, payload: String) -> Vec<KafkaGeneralEvent> {
    let kafka_general_event = KafkaGeneralEvent {
        topic: "user".to_string(),
        payload: payload,
        key: "user-left-room".to_string(),
    };
    let results_resp: Vec<KafkaGeneralEvent> = vec![kafka_general_event];
    return results_resp
}

pub async fn create_user_status_room_event(context: &DynContext, payload: String) -> Vec<KafkaGeneralEvent> {
    let kafka_general_event = KafkaGeneralEvent {
        topic: "user".to_string(),
        payload: payload,
        key: USER_STATUS_EVENT.to_string(),
    };
    let results_resp: Vec<KafkaGeneralEvent> = vec![kafka_general_event];
    return results_resp
}