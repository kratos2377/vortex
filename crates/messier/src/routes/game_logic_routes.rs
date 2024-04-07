use axum::{middleware, routing::{get, post}, Router};

use crate::{controllers, state::AppDBState, utils};



pub fn create_game_routes() -> Router<AppDBState> {
   Router::new()
        .route("/create_lobby", post(controllers::game_controller::create_lobby))
        .route("/verify_game_status", post(controllers::game_controller::verify_game_status))
        .route("/join_lobby", post(controllers::game_controller::join_lobby))
        .route("/remove_user_lobby", post(controllers::game_controller::remove_user_from_lobby))
        .route("/destroy_lobby_and_game", post(controllers::game_controller::destroy_lobby_and_game))
        .route("/get_ongoing_games_for_user", get(controllers::game_controller::get_ongoing_games_for_user))
        .route("/get_current_state_of_game", get(controllers::game_controller::get_current_state_of_game))
        .route("/stake_in_game", get(controllers::game_controller::stake_in_game))
        .route_layer(middleware::from_fn(utils::middleware::guard))
}