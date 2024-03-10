use axum::extract::State;
use rdkafka::producer::FutureProducer;
use serde::{Deserialize, Serialize};
use socketioxide::{extract::{Data, SocketRef}, handler::ConnectHandler, socket};
use tracing::info;
use std::sync::{Arc, Mutex};
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
pub fn create_ws_game_events(socket: SocketRef, state: State<Arc<FutureProducer>>) {
    
    
    socket.on("joined-room", |socket: SocketRef , Data::<String>(msg) | {
        let data: JoinedRoomPayload = serde_json::from_str(&msg).unwrap();

      let _ =   socket.broadcast().to(data.game_id).emit("new-user-joined" , msg);
    });


    socket.on("leaved-room", |socket: SocketRef ,  Data::<String>(msg)| {
        let data: LeavedRoomPayload = serde_json::from_str(&msg).unwrap();

        let _ = socket.broadcast().to(data.game_id).emit("user-left-room" , msg);
    });


    socket.on("game-event", |socket: SocketRef , Data::<String>(msg)| {
        let data: GameEventPayload = serde_json::from_str(&msg).unwrap();

       let  _ =  socket.broadcast().to(data.game_id).emit("send-user-game-event" , msg);
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