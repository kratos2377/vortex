use serde::Deserialize;
use serde::Serialize;
use uuid::Uuid;


pub const SCHEMA_NAME_CREATE_USER_GAME_EVENT: &str = "CreateUserGameAvroV1";


// Create  User game event record
pub const RAW_SCHEMA_CREATE_USER_GAME_EVENT_V1: &str =
    include_str!("../../resources/create_user_game_v1.avsc");

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateUserGameMoveEventAvro {
    pub identifier: String,
    pub name: String,
    pub description: String,
    pub user_game_move: UserGameMoveAvro,
}


#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UserGameMoveAvro {
    pub id: Uuid,
    pub user_id: String,
    pub game_id: String,
    pub version: i64,
    pub user_move: String,
    pub move_type: String,
    pub socket_id: String,

}

