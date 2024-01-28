use axum::{routing::post, Router};

use crate::{controllers, state::AppDBState};



pub fn create_user_routes() -> Router<AppDBState> {
   Router::new()
        .route("/login", post(controllers::user_controller::login_user))
        .route("/registration", post(controllers::user_controller::register_user))
        .route("/send_email", post(controllers::user_controller::send_email))
        .route("/verify_user", post(controllers::user_controller::verify_user))
}