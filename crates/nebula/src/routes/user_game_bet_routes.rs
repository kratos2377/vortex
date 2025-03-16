use axum::{routing::{get, post, put}, Router};

use crate::{controllers, state::AppDBState};



pub fn create_user_game_bet_routes() -> Router<AppDBState> {
    Router::new()
    .route("/get_user_bets/:user_id/:wallet_key/:page", get(controllers::user_game_bet_controllers::get_user_bets))
}