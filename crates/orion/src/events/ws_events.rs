use serde::Deserialize;
use serde::Serialize;

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
    pub player_type: String,
}

#[derive(Deserialize , Serialize)]
pub struct UpdateUserStatusPayload {
    pub user_id: String,
    pub username: String,
    pub game_id: String,
    pub status: String,
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

#[derive(Deserialize, Serialize)]
pub struct GetUserTurnsMappingWSPayload {
    pub game_id: String,
}

#[derive(Deserialize, Serialize)]
pub struct ErrorMessagePayload {
    pub game_id: String,
    pub error_message: String
}

#[derive(Deserialize , Serialize)]
pub struct UserConnectionEventPayload {
    pub user_id: String,
    pub username: String,
}

#[derive(Deserialize , Serialize)]
pub struct UserKafkaPayload {
    pub user_id: String,
    pub socket_id: String,
}

#[derive(Clone , Deserialize , Serialize)]
pub struct VerifyingStatusPayload {
    pub user_id: String,
    pub game_id: String
}
