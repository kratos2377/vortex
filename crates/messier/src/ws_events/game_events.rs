use futures::future;
use orion::{constants::{REDIS_USER_GAME_KEY, REDIS_USER_PLAYER_KEY, USER_JOINED_ROOM, USER_LEFT_ROOM, USER_ONLINE_EVENT, USER_STATUS_EVENT, VERIFYING_GAME_STATUS}, events::{kafka_event::{KafkaGeneralEvent, UserGameDeletetionEvent}, ws_events::{ErrorMessagePayload, GameMessagePayload, GameStartPayload, GetUserTurnsMappingWSPayload, JoinedRoomPayload, LeavedRoomPayload, UpdateUserStatusPayload, UserConnectionEventPayload, UserKafkaPayload, VerifyingStatusPayload}}};
use rdkafka::{error::KafkaError, message::{Header, OwnedHeaders}, producer::{FutureProducer, FutureRecord, Producer}, util::Timeout};
use redis::{Commands, Connection, RedisResult};
use socketioxide::{extract::{Data, SocketRef, State}, handler::ConnectHandler, socket};
use std::{sync::{Arc, Mutex}, time::Duration};

use crate::{ event_producer::{game_events_producer::{send_game_move_events, GameEventPayload, UserReadyEventPayload}, user_events_producer::send_event_for_user_topic}, kafka::model::{Event, EventList}, state::WebSocketStates};


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
        let _ = socket.join(data.game_id.clone());
      let _ =   socket.broadcast().to(data.game_id).emit("new-user-joined" , msg.clone());

      async move  {
        send_event_for_user_topic(&producer, &context, USER_JOINED_ROOM.to_string() ,msg).await.unwrap();
      }
    });


    socket.on("leaved-room", |socket: SocketRef ,  Data::<String>(msg), State(WebSocketStates { producer, context } )| {
        let data: LeavedRoomPayload = serde_json::from_str(&msg).unwrap();
        if data.player_type == "host" {
            let _ = socket.broadcast().to(data.game_id.clone()).emit("remove-all-users", msg.clone());
        } else {
            let _ = socket.broadcast().to(data.game_id.clone()).emit("user-left-room" , msg.clone());
        }
        
        let _ = socket.leave(data.game_id);

        async move {
            send_event_for_user_topic(&producer, &context, USER_LEFT_ROOM.to_string() ,msg).await.unwrap();
         }
    });

    socket.on("update-user-status-in-room", |socket: SocketRef ,  Data::<String>(msg), State(WebSocketStates { producer, context } )| {
        let data: UpdateUserStatusPayload = serde_json::from_str(&msg).unwrap();

 
            let _ = socket.broadcast().to(data.game_id).emit("user-status-update" , msg.clone());
      
     
        // Add general kafka event for this
        async move {
            send_event_for_user_topic(&producer, &context, USER_STATUS_EVENT.to_string() ,msg).await.unwrap();
        }
    });


    socket.on("game-event",    |socket: SocketRef , Data::<String>(msg), State(WebSocketStates { producer, context } )| {
        let data: GameEventPayload = serde_json::from_str(&msg).unwrap();
        let data_clone = data.clone();

        let  _ =  socket.broadcast().to(data.game_id).emit("send-user-game-event" , msg);
      
        async move {
            send_game_move_events(&context , data_clone , socket.id.to_string() , producer).await.unwrap();
        }

    });



    socket.on("verifying-game-status",    |socket: SocketRef , Data::<String>(msg), State(WebSocketStates { producer, context } )| {
        let data: VerifyingStatusPayload = serde_json::from_str(&msg).unwrap();
        let  _ =  socket.broadcast().to(data.game_id).emit("verifying-game" , msg.clone());
      
        async move {
            send_event_for_user_topic(&producer, &context, VERIFYING_GAME_STATUS.to_string() ,msg).await.unwrap();
        }

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

    // tell lobby users to fetch user-turn-mappings 
    socket.on("get-turn-mappings" , |socket: SocketRef, Data::<String>(msg)| {
        let data: GetUserTurnsMappingWSPayload = serde_json::from_str(&msg).unwrap();

        let _ = socket.broadcast().to(data.game_id).emit("fetch-user-turn-mappings" , msg);
    });


    // Error Events
    socket.on("error-event" ,|socket: SocketRef, Data::<String>(msg)| {
        let data: ErrorMessagePayload = serde_json::from_str(&msg).unwrap();

        let _ = socket.broadcast().to(data.game_id).emit("error-event-occured" , msg);
    });

   socket.on_disconnect(|socket: SocketRef, State(WebSocketStates { producer, context } )| async move {
    
    let user_id = get_key_from_redis(context.get_redis_db_client(), socket.id.to_string()).await;
    let game_id = get_key_from_redis(context.get_redis_db_client(), user_id.clone() + REDIS_USER_GAME_KEY).await;
    let user_type = get_key_from_redis(context.get_redis_db_client(), user_id.clone() + REDIS_USER_PLAYER_KEY).await;
    
    if user_type == "host" {
        let _ = socket.broadcast().to(game_id.clone()).emit("remove-all-users", "no-data");
    }

    // Remove Keys from redis
    remove_key_from_redis(context.get_redis_db_client() , user_id.clone()).await;
    remove_key_from_redis(context.get_redis_db_client() , user_id.clone() + REDIS_USER_GAME_KEY).await;
    remove_key_from_redis(context.get_redis_db_client() , user_id.clone() + REDIS_USER_PLAYER_KEY).await;
    remove_key_from_redis(context.get_redis_db_client() , socket.id.to_string().clone()).await;

    if game_id != "nil" && game_id != "" {
    // leave socket rooms
    let _ = socket.leave(game_id.clone());

    // Send Kafka Event
    let _ = produce_user_game_deletion_kafka_event(producer , user_id.clone()).await;
    } 

    // Send USER_OFFLINE_EVENT and user leave event to room if it exists and make necessary MQTT events
       socket.disconnect().ok();
});


}


pub async fn add_key_in_redis(redis_client: Arc<Mutex<Connection>> , key: String , value: String) {
    let mut redis_conn = redis_client.lock().unwrap();
    let res: RedisResult<()> = redis_conn.set(key, value);

    if res.is_err() {
        println!("Error occured while persisting key in redis");
        println!("{:?}", res);
    }
}

pub async fn get_key_from_redis(redis_client: Arc<Mutex<Connection>> , key: String) -> String {
    let mut redis_conn = redis_client.lock().unwrap();
    let res: RedisResult<String> = redis_conn.get(key);

    if res.is_err() {
        return "nil".to_string()
    }

  res.unwrap()
}

pub async fn remove_key_from_redis(redis_client: Arc<Mutex<Connection>> , key: String) {
    let mut redis_conn = redis_client.lock().unwrap();
    let _: RedisResult<()> = redis_conn.del(key);
}

// The User-Game-Deletion event will delete events from mongo db
pub async fn produce_user_game_deletion_kafka_event(producer: &FutureProducer, user_id: String) -> Result<(), KafkaError> {
    let deletion_event = UserGameDeletetionEvent{ user_id: user_id };
    let kafka_events = vec![ 
        KafkaGeneralEvent {
            topic: "user_game_deletion".to_string(),
            payload: serde_json::to_string(&deletion_event).unwrap(),
            key: "user-game-deletion-event".to_string(),
        }
    ];
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