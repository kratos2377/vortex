use axum::extract::State;
use lettre::transport::smtp::commands::Data;
use rdkafka::producer::FutureProducer;
use socketioxide::extract::SocketRef;


pub struct JoinedRoomPayload {
    pub user_id: String,
    pub username: String,
    pub game_id: String,
}

pub struct LeavedRoomPayload {
    pub user_id: String,
    pub username: String,
    pub game_id: String,
}


pub struct GameEventPayload {
    pub user_id: String,
    pub game_event: String,
    pub game_id: String
}

pub struct GameStartPayload {
    pub admin_id: String,
    pub game_name: String,
    pub game_id: String
}


pub struct GameMessagePayload {
    pub user_id: String,
    pub username: String,
    pub message: String,
    pub game_id: String
}
pub async fn create_ws_game_events(socket: SocketRef, state: State<Arc<FutureProducer>>) {


    
    socket.on("joined-room", |s: SocketRef , msg: Data::<String>| {
        let data: JoinedRoomPayload = serde_json::from_str(&msg)?;

        socket.broadcast().to(data.game_id).emit("new-user-joined" , msg)
    });


    socket.on("leaved-room", |s: SocketRef , msg: Data::<String>| {
        let data: LeavedRoomPayload = serde_json::from_str(&msg)?;

        socket.broadcast().to(data.game_id).emit("user-left-room" , msg)
    });


    socket.on("game-event", |s: SocketRef , msg: Data::<String>| {
        let data: GameEventPayload = serde_json::from_str(&msg)?;

        socket.broadcast().to(data.game_id).emit("send-user-game-event" , msg)
    });


    socket.on("start-game-event", |s: SocketRef, msg: Data::<String>| {
        let data: GameStartPayload = serde_json::from_str(&msg)?;

        socket.broadcast().to(data.game_id).emit("start-game-for-all" , msg)
    });

    socket.on("user-message-in-game", |s: SocketRef, msg: Data::<String>| {
        let data: GameMessagePayload = serde_json::from_str(&msg)?;

        socket.broadcast().to(data.game_id).emit("someone-sent-a-message" , msg)
    });



   // socket.on_disconnect(callback)


}