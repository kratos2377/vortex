use axum::{routing::post, Router};

use crate::{controllers, state::AppDBState};



pub fn create_game_routes() -> Router<AppDBState> {
   Router::new()
        .route("/create_lobby", post(controllers::game_controller::create_lobby))
        .route("/verify_game_status", post(controllers::game_controller::verify_game_status))
        .route("/join_lobby", post(controllers::game_controller::join_lobby))
        .route("/remove_user_lobby", post(controllers::game_controller::remove_user_from_lobby))
        .route("/destroy_lobby_and_game", post(controllers::game_controller::destroy_lobby_and_game))
}