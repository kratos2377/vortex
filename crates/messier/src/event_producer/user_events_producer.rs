use std::time::Duration;

use orion::events::{kafka_event::{KafkaGeneralEvent, UserFriendRequestKafkaEvent, UserGameInviteKafkaEvent, UserOnlineKafkaEvent}, ws_events::{JoinedRoomPayload, UserConnectionEventPayload}};
use rdkafka::{error::KafkaError, producer::{FutureProducer, FutureRecord, Producer}, util::Timeout};
use redis::{Commands, RedisResult};
use sea_orm::TryIntoModel;
use futures_util::future;
use ton::models::{self, users, users_wallet_keys};
use models::{users_friends_requests::{self , Entity as UsersFriendsRequests}, users_friends::{self, Entity as UsersFriends}, users::{Entity as Users}};
use crate::context::context::DynContext;




pub async fn send_user_online_event() {
todo!()
}

pub async fn send_user_friend_request_event() {
todo!()
}

pub async fn send_game_invite_request_to_user() {
    todo!()
}

// Make these events name into const static strings
pub async fn send_event_for_user_topic(
    producer: &FutureProducer,
    context: &DynContext,
    event_name: String,
    payload: String
) -> Result<(), KafkaError> {
    let kafka_events: Vec<KafkaGeneralEvent> = match event_name.as_ref() {
        "user-online-event" => {
            create_user_online_events(context , payload).await
        },
        "friend-request-event" => {
            create_friend_request_event(context , payload).await
        },
        "game-invite-event" => {
            create_game_invite_event(context , payload).await
        },
        "user-joined-room" => {
            create_user_joined_room_event(context , payload).await
        },

        "user-left-room" => {
            create_user_leaved_room_event(context , payload).await
        },

        _ => {vec![]}

    };


    producer.begin_transaction().unwrap();


    let kafka_result = future::try_join_all(kafka_events.iter().map(|event| async move {

        let delivery_result = producer
        .send(
            FutureRecord::to(&event.topic)
                    .payload(&event.payload)
                    .key(&event.key),
            Duration::from_secs(20),
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

    producer.commit_transaction(Timeout::from(Duration::from_secs(10))).unwrap(); 

    Ok(())

}


pub async fn create_user_online_events(context: &DynContext, payload: String) -> Vec<KafkaGeneralEvent> {
    let data: UserConnectionEventPayload = serde_json::from_str(&payload).unwrap();
    let connection = context.get_postgres_db_client();
    let result = users_friends::Entity::find_by_user_id(&data.user_id).all(&connection).await.unwrap();



    let arc_redis_client = context.get_redis_db_client();

    let mut results_resp: Vec<KafkaGeneralEvent> = vec![];
   
       // Only add keys if the user_id key is present in redis cluster
       // But we might have to use async redis because not able to pass data safely in threads in single thread
       
           for mo in result.iter() {
        let user_friend: users_friends::Model = mo.clone().try_into_model().unwrap();
               
   
       
           let friend_result = match Users::find_by_id(&user_friend.friendid.to_string())
               .one(&connection)
               .await
           {
               Ok(data) => data,
               Err(err) => continue, // Skip to the next iteration on error
           };

           let user_type_details = friend_result.unwrap().try_into_model().unwrap();

           let mut redisConnection  = arc_redis_client.lock().unwrap();
           let does_friend_key_exist_in_redis: RedisResult<()> = redisConnection.hkeys(user_type_details.id.to_string().clone());

           if does_friend_key_exist_in_redis.is_err() {
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
        topic: "users".to_string(),
        payload: payload,
        key: "friend-request-event".to_string(),
    };
    let results_resp: Vec<KafkaGeneralEvent> = vec![kafka_general_event];
    return results_resp
}


pub async fn create_game_invite_event(context: &DynContext, payload: String) -> Vec<KafkaGeneralEvent> {
    let kafka_general_event = KafkaGeneralEvent {
        topic: "users".to_string(),
        payload: payload,
        key: "game-invite-event".to_string(),
    };
    let results_resp: Vec<KafkaGeneralEvent> = vec![kafka_general_event];
    return results_resp
}

pub async fn create_user_joined_room_event(context: &DynContext, payload: String) -> Vec<KafkaGeneralEvent> {
    let kafka_general_event = KafkaGeneralEvent {
        topic: "users".to_string(),
        payload: payload,
        key: "user-joined-room".to_string(),
    };
    let results_resp: Vec<KafkaGeneralEvent> = vec![kafka_general_event];
    return results_resp
}

pub async fn create_user_leaved_room_event(context: &DynContext, payload: String) -> Vec<KafkaGeneralEvent> {
    let kafka_general_event = KafkaGeneralEvent {
        topic: "users".to_string(),
        payload: payload,
        key: "user-left-room".to_string(),
    };
    let results_resp: Vec<KafkaGeneralEvent> = vec![kafka_general_event];
    return results_resp
}