use serde::{Deserialize, Serialize};
use uuid::Uuid;


#[derive(Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UserGameEvent {
    #[serde(with = "bson::serde_helpers::uuid_1_as_binary")]
    pub id: Uuid,
    pub user_id: String,
    pub game_id: String,
    pub version: i64,
    pub user_move: String,
    pub socket_id: String,
}


// look at event -> dto.rs