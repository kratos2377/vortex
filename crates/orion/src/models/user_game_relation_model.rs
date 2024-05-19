use serde::Deserialize;
use serde::Serialize;
use uuid::Uuid;




#[derive(Deserialize , Serialize , Clone)]

pub struct UserGameRelation {
    #[serde(with = "bson::serde_helpers::uuid_1_as_binary")]
    pub user_id: Uuid,
    pub game_id: Option<String>,
    pub player_type: String
}