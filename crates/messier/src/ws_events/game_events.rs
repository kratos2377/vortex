
use futures::FutureExt;
use futures_util::future;
use orion::{constants::{USER_JOINED_ROOM, USER_LEFT_ROOM, USER_ONLINE_EVENT, USER_READY_EVENT}, events::ws_events::{GameMessagePayload, GameStartPayload, JoinedRoomPayload, LeavedRoomPayload, UserConnectionEventPayload, UserKafkaPayload}};
use rdkafka::{error::KafkaError, message::{Header, OwnedHeaders}, producer::{FutureProducer, FutureRecord, Producer}, util::Timeout};
use redis::{Commands, Connection, RedisResult};
use socketioxide::{extract::{Data, SocketRef, State}, handler::ConnectHandler, socket};
use tracing::info;
use uuid::Uuid;
use std::{sync::{Arc, Mutex}};

use crate::{ event_producer::{game_events_producer::{send_game_move_events, GameEventPayload, UserReadyEventPayload}, user_events_producer::send_event_for_user_topic}, kafka::model::{Event, EventList}, mongo::{kafka_event_models::{UserGameEvent, UserGameMove}, send_kafka_events_to_mongo::create_and_send_kafka_events, transaction::transactional}, state::WebSocketStates};


pub fn create_ws_game_events(socket: SocketRef) {
    
    socket.on("user-connection-event",   |socket: SocketRef, Data::<String>(msg), State(WebSocketStates { producer, context } ) | {
        let data: UserConnectionEventPayload = serde_json::from_str(&msg).unwrap();
     
         async move {
          //  produce_kafka_event_for_redis(&producer, "user".to_string() , socket.id.to_string() , data.user_id).await.unwrap();
            add_key_in_redis(context.get_redis_db_client(), data.user_id.clone(), "online".to_string()).await;
            add_key_in_redis(context.get_redis_db_client(), socket.id.to_string(), data.user_id).await;
            send_event_for_user_topic(&producer, &context, USER_ONLINE_EVENT.to_string() ,msg).await.unwrap();
         }
    });
    
    socket.on("joined-room", |socket: SocketRef , Data::<String>(msg), State(WebSocketStates { producer, context } ) | {
        let data: JoinedRoomPayload = serde_json::from_str(&msg).unwrap();

      let _ =   socket.broadcast().to(data.game_id).emit("new-user-joined" , msg.clone());

      async move  {
        send_event_for_user_topic(&producer, &context, USER_JOINED_ROOM.to_string() ,msg).await.unwrap();
      }
    });


    socket.on("leaved-room", |socket: SocketRef ,  Data::<String>(msg), State(WebSocketStates { producer, context } )| {
        let data: LeavedRoomPayload = serde_json::from_str(&msg).unwrap();

        let _ = socket.broadcast().to(data.game_id).emit("user-left-room" , msg.clone());

        async move {
            send_event_for_user_topic(&producer, &context, USER_LEFT_ROOM.to_string() ,msg).await.unwrap();
         }
    });


    socket.on("game-event",    |socket: SocketRef , Data::<String>(msg), State(WebSocketStates { producer, context } )| {
        let data: GameEventPayload = serde_json::from_str(&msg).unwrap();
        let data_clone = data.clone();
        let  _ =  socket.broadcast().to(data.game_id).emit("send-user-game-event" , msg);
      
        async move {
            send_game_move_events(&context , data_clone , socket.id.to_string()).await.unwrap();
        }

    });


    socket.on("ready-to-play-event" ,    |socket: SocketRef , Data::<String>(msg), State(WebSocketStates { producer, context } )| {
        let data: UserReadyEventPayload = serde_json::from_str(&msg).unwrap();
        let  _ =  socket.broadcast().to(data.game_id).emit("user-ready-event" , msg.clone());
      
        async move {
            send_event_for_user_topic(&producer, &context, USER_READY_EVENT.to_string() ,msg).await.unwrap();
        }

    });

    socket.on("verifying-game-status",    |socket: SocketRef , Data::<String>(msg), State(WebSocketStates { producer, context } )| {
        let data: UserReadyEventPayload = serde_json::from_str(&msg).unwrap();
        let  _ =  socket.broadcast().to(data.game_id).emit("user-ready-event" , msg.clone());
      
        // async move {
        //     send_event_for_user_topic(&producer, &context, USER_READY_EVENT.to_string() ,msg).await.unwrap();
        // }

    });


    socket.on("start-game-event", |socket: SocketRef, Data::<String>(msg)| {
        let data: GameStartPayload = serde_json::from_str(&msg).unwrap();
       // socket.game_id = data.game_id;
        let _ = socket.broadcast().to(data.game_id).emit("start-game-for-all" , msg);
    });

    socket.on("user-message-in-game", |socket: SocketRef, Data::<String>(msg)| {
        let data: GameMessagePayload = serde_json::from_str(&msg).unwrap();

        let _ = socket.broadcast().to(data.game_id).emit("someone-sent-a-message" , msg);
    });


   socket.on_disconnect(|socket: SocketRef| async move {
    //Add Events to publish disconnected events
    // Maybe i will add a redis layer for this
    //   info!("Socket.IO disconnected: {} {}", socket.id, "reason");

       socket.disconnect().ok();
});


}


pub async fn add_key_in_redis(redis_client: Arc<Mutex<Connection>> , key: String , value: String) {

    let mut redis_conn = redis_client.lock().unwrap();
    let _: RedisResult<()> = redis_conn.set(key, value);

}