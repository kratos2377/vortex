use axum::{routing::{get, post, put}, Router};

use crate::{controllers, state::AppDBState};



pub fn create_user_game_bet_routes() -> Router<AppDBState> {
    Router::new()
    .route("/get_user_bets", get(controllers::user_game_bet_controllers::get_user_bets))
    .route("/place_user_bet", post(controllers::user_game_bet_controllers::place_user_bet))
    .route("/update_user_bet", put(controllers::user_game_bet_controllers::update_user_bet))
}