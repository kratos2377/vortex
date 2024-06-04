use serde::{Deserialize, Serialize};


#[derive(Deserialize , Serialize)]
pub struct UserTurnMapping {
    pub game_id: String,
    pub turn_mappings: Vec<TurnModel>,
}



#[derive(Deserialize , Serialize)]
pub struct TurnModel {
    pub count_id: i64,
    pub user_id: String,
    pub username: String,
}
