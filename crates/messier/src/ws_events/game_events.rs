use axum::extract::State;
use rdkafka::producer::FutureProducer;
use socketioxide::extract::SocketRef;




pub async fn create_ws_game_events(socket: SocketRef, state: State<FutureProducer>) {

    // socket.on("connection", |s , |)


}