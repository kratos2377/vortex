use crate::models;
use crate::errors;
use axum::Json;
use models::user_model::UserModel;
use errors::Result;
use serde_json::Value;


pub async fn login_user() -> Result<Json<Value>> {
todo!()
}

pub async fn register_user() -> Result<Json<Value>> {
    todo!()
}

pub async fn send_email() -> Result<Json<Value>> {
    todo!()
}

pub async fn verify_user() -> Result<Json<Value>> {
todo!()
}