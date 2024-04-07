
use futures::FutureExt;
use futures_util::future;
use orion::events::ws_events::{GameMessagePayload, GameStartPayload, JoinedRoomPayload, LeavedRoomPayload, UserConnectionEventPayload, UserKafkaPayload};
use rdkafka::{error::KafkaError, message::{Header, OwnedHeaders}, producer::{FutureProducer, FutureRecord, Producer}, util::Timeout};
use serde::{Deserialize, Serialize};
use socketioxide::{extract::{Data, SocketRef, State}, handler::ConnectHandler, socket};
use tracing::info;
use uuid::Uuid;
use std::{sync::{Arc, Mutex}, time::Duration};

use crate::{ event_producer::{game_events_producer::{send_game_events, GameEventPayload}, user_events_producer::send_event_for_user_topic}, kafka::model::{Event, EventList}, mongo::{kafka_event_models::{UserGameEvent, UserGameMove}, send_kafka_events_to_mongo::create_and_send_kafka_events, transaction::transactional}, state::WebSocketStates};


pub fn create_ws_game_events(socket: SocketRef) {
    
    socket.on("user-connection-event",   |socket: SocketRef, Data::<String>(msg), State(WebSocketStates { producer, context } ) | {
        let data: UserConnectionEventPayload = serde_json::from_str(&msg).unwrap();
     
         async move {
            produce_kafka_event_for_redis(&producer, "user".to_string() , socket.id.to_string() , data.user_id).await.unwrap();
            send_event_for_user_topic(&producer, &context, "user-online-even".to_string() ,msg).await;
         }
    });
    
    socket.on("joined-room", |socket: SocketRef , Data::<String>(msg), State(WebSocketStates { producer, context } ) | {
        let data: JoinedRoomPayload = serde_json::from_str(&msg).unwrap();

      let _ =   socket.broadcast().to(data.game_id).emit("new-user-joined" , msg.clone());

      async move  {
        send_event_for_user_topic(&producer, &context, "user-joined-room".to_string() ,msg).await;
      }
    });


    socket.on("leaved-room", |socket: SocketRef ,  Data::<String>(msg), State(WebSocketStates { producer, context } )| {
        let data: LeavedRoomPayload = serde_json::from_str(&msg).unwrap();

        let _ = socket.broadcast().to(data.game_id).emit("user-left-room" , msg.clone());

        async move {
            send_event_for_user_topic(&producer, &context, "user-left-room".to_string() ,msg).await;
         }
    });


    socket.on("game-event",    |socket: SocketRef , Data::<String>(msg), State(WebSocketStates { producer, context } )| {
        let data: GameEventPayload = serde_json::from_str(&msg).unwrap();
        let data_clone = data.clone();
        let  _ =  socket.broadcast().to(data.game_id).emit("send-user-game-event" , msg);
      
        async move {
        send_game_events(&context , data_clone , socket.id.to_string()).await.unwrap();
        }

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
