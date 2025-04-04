use serde::{Deserialize, Serialize};



#[derive(Serialize , Deserialize)]
pub struct PlaceUserBetPayload {
pub user_id: String,
pub game_id: String,
pub game_name: String,
pub session_id: String,
pub user_id_betting_on: String,
pub bet_amount: f64
}

#[derive(Serialize , Deserialize)]
pub struct UpdateUserPlaceBetPayload {
pub user_id: String,
pub game_id: String,
pub add_more_amount: f64
}