
use futures_util::future;
use rdkafka::{error::KafkaError, message::{Header, OwnedHeaders}, producer::{FutureProducer, FutureRecord, Producer}, util::Timeout};
use serde::{Deserialize, Serialize};
use socketioxide::{extract::{Data, SocketRef, State}, handler::ConnectHandler, socket};
use tracing::info;
use uuid::Uuid;
use std::{sync::{Arc, Mutex}, time::Duration};

use crate::{context::context::DynContext, kafka::model::{Event, EventList}, state::WebSocketStates};
#[derive(Deserialize , Serialize)]
pub struct JoinedRoomPayload {
    pub user_id: String,
    pub username: String,
    pub game_id: String,
}

#[derive(Deserialize , Serialize)]
pub struct LeavedRoomPayload {
    pub user_id: String,
    pub username: String,
    pub game_id: String,
}

#[derive(Deserialize , Serialize)]
pub struct GameEventPayload {
    pub user_id: String,
    pub game_event: String,
    pub game_id: String
}

#[derive(Deserialize , Serialize)]
pub struct GameStartPayload {
    pub admin_id: String,
    pub game_name: String,
    pub game_id: String
}

#[derive(Deserialize , Serialize)]
pub struct GameMessagePayload {
    pub user_id: String,
    pub username: String,
    pub message: String,
    pub game_id: String
}

#[derive(Deserialize , Serialize)]
pub struct UserConnectionEventPayload {
    pub user_id: String,
}

#[derive(Deserialize , Serialize)]
pub struct UserKafkaPayload {
    pub user_id: String,
    pub socket_id: String,
}

pub fn create_ws_game_events(socket: SocketRef) {
    
    socket.on("user-connection-event",   |socket: SocketRef, Data::<String>(msg), State(WebSocketStates { producer, context } ) | {
        let data: UserConnectionEventPayload = serde_json::from_str(&msg).unwrap();
     
         produce_kafka_event_for_redis(&producer, "user".to_string() , socket.id.to_string() , data.user_id);
    });
    
    socket.on("joined-room", |socket: SocketRef , Data::<String>(msg) | {
        let data: JoinedRoomPayload = serde_json::from_str(&msg).unwrap();

      let _ =   socket.broadcast().to(data.game_id).emit("new-user-joined" , msg);
    });


    socket.on("leaved-room", |socket: SocketRef ,  Data::<String>(msg)| {
        let data: LeavedRoomPayload = serde_json::from_str(&msg).unwrap();

        let _ = socket.broadcast().to(data.game_id).emit("user-left-room" , msg);
    });


    socket.on("game-event",    |socket: SocketRef , Data::<String>(msg), State(WebSocketStates { producer, context } )| {
        let data: GameEventPayload = serde_json::from_str(&msg).unwrap();

        let  _ =  socket.broadcast().to(data.game_id).emit("send-user-game-event" , msg);
        produce_kafka_event_for_mongo(&producer, "user".to_string(), socket.id.to_string(), "user_id".to_string());

    });


    socket.on("start-game-event", |socket: SocketRef, Data::<String>(msg)| {
        let data: GameStartPayload = serde_json::from_str(&msg).unwrap();

        let _ = socket.broadcast().to(data.game_id).emit("start-game-for-all" , msg);
    });

    socket.on("user-message-in-game", |socket: SocketRef, Data::<String>(msg)| {
        let data: GameMessagePayload = serde_json::from_str(&msg).unwrap();

        let _ = socket.broadcast().to(data.game_id).emit("someone-sent-a-message" , msg);
    });


   // socket.on_disconnect(callback)

   socket.on_disconnect(|socket: SocketRef| async move {
       info!("Socket.IO disconnected: {} {}", socket.id, "reason");
       socket.disconnect().ok();
});


}



// Publish that user has joined to user friends
async fn produce_kafka_event_for_redis(producer: &FutureProducer, topic: String , socket_id: String , user_id: String) -> Result<(), KafkaError> {
    producer.begin_transaction().unwrap();


    //Create a future to be published
    //producer.send(record, Duration::from_secs(10));
    let user_online_payload = UserKafkaPayload {
            user_id: user_id,
            socket_id: socket_id
    };

    let user_string = serde_json::to_string(&user_online_payload).unwrap();

    // Change partition and key
    let new_event = Event {
        _id: Uuid::new_v4().to_string(),
        topic: topic,
        partition: 2,
        key: "random key".into(),
        payload: user_string.into(),
        event_name: "user-online".into()
    };

    let new_event_list = EventList {
        has_more: false,
        events: vec![new_event]
    };
    let kafka_result = future::try_join_all(new_event_list.events.iter().map(|event| async move {

        let delivery_result = producer
        .send(
            FutureRecord::to(&event.topic)
                    .payload(&event.payload)
                    .key(&event.key),
            Duration::from_secs(30),
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


pub async fn produce_kafka_event_for_mongo(producer: &FutureProducer, topic: String , socket_id: String , user_id: String) -> Result<(), KafkaError> {
//     producer.begin_transaction().unwrap();


//     //Create a future to be published
//     //producer.send(record, Duration::from_secs(10));
//     let user_online_payload = UserKafkaPayload {
//             user_id: user_id,
//             socket_id: socket_id
//     };

//     let user_string = serde_json::to_string(&user_online_payload).unwrap();

//     // Change partition and key
//     let new_event = Event {
//         _id: Uuid::new_v4().to_string(),
//         topic: topic,
//         partition: 2,
//         key: "random key".into(),
//         payload: user_string.into(),
//         event_name: "user-online".into()
//     };

//     let new_event_list = EventList {
//         has_more: false,
//         events: vec![new_event]
//     };
//     let kafka_result = future::try_join_all(new_event_list.events.iter().map(|event| async move {

//         let delivery_result = producer
//         .send(
//             FutureRecord::to(&event.topic)
//                     .payload(&event.payload)
//                     .key(&event.key),
//             Duration::from_secs(30),
//         )
//         .await;

//     // This will be executed when the result is received.
//   //  println!("Delivery status for message {} received", i);
//     delivery_result

//     })

//     ).await;

//     match kafka_result {
//         Ok(_) => (),
//         Err(e) => return Err(e.0.into()),
//     }

//     producer.commit_transaction(Timeout::from(Duration::from_secs(10))).unwrap(); 

//     Ok(())  


let events = event_dispatcher
.dispatch(event_type.to_string(), dto)
.await?;

assert!(!events.is_empty());

for event in events {
event_service::save(db_session, &event).await?;
}
Ok(())
}