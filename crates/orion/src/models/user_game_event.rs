use serde::{Deserialize, Serialize};
use uuid::Uuid;


#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct UserGameMove {
    pub user_id: String,
    pub game_id: String,
    pub move_type: String,
    pub user_move: String,
}

