use axum::{routing::post, Router};

use crate::{controllers, state::AppDBState};



pub fn create_game_routes() -> Router<AppDBState> {
   Router::new()
        .route("/create_lobby", post(controllers::game_controller::create_lobby))
        .route("/join_lobby", post(controllers::game_controller::join_lobby))
        .route("/remove_user_lobby", post(controllers::game_controller::remove_user_from_lobby))
        .route("/lobby/:id", post(controllers::game_controller::broadcast_game_event))
        .route("/destroy_lobby_and_game", post(controllers::game_controller::destroy_lobby_and_game))
}