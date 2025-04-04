use axum::{middleware, routing::{post,get,put}, Router};

use crate::{controllers, state::AppDBState, utils};



pub fn create_user_logic_routes() -> Router<AppDBState> {
    Router::new()
    .route("/send_request", post(controllers::user_logic_controller::send_request))
    .route("/get_user_friend_requests", get(controllers::user_logic_controller::get_user_friend_requests))
    .route("/accept_or_reject_request", put(controllers::user_logic_controller::accept_or_reject_request))
    .route("/add_wallet_address", post(controllers::user_logic_controller::add_wallet_address))
    .route("/get_user_wallets", post(controllers::user_logic_controller::get_user_wallets))
    .route("/delete_wallet_address", post(controllers::user_logic_controller::delete_wallet_address))
    .route("/get_user_online_friends", get(controllers::user_logic_controller::get_user_online_friends))
    .route("/change_user_password", put(controllers::user_logic_controller::change_user_password))
    .route("/change_user_username", put(controllers::user_logic_controller::change_user_username))
    .route("/get_ongoing_games_for_user", get(controllers::user_logic_controller::get_ongoing_games_for_user))
    .route_layer(middleware::from_fn(utils::middleware::guard))

}