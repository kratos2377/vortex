use serde::{Deserialize, Serialize};


#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct UserScoreUpdateEvent {
    pub user_id: String,
    pub game_id: String,
    pub score: i32,
}

