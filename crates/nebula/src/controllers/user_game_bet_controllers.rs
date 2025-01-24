use std::str::FromStr;

use crate::{errors::{self, Error}, state::AppDBState};

use axum::{extract::{State, Query}, Json};
use chrono::Utc;
use errors::Result as APIResult;
use orion::models::{game_bet_events::GameBetStatus, game_model::Game};
use sea_orm::{QuerySelect, Set};
use serde_json::{json, Value};
use ton::models::game_bets;
use uuid::Uuid;
use sea_orm::ActiveModelTrait;
use super::payload::{PlaceUserBetPayload, UpdateUserPlaceBetPayload};

pub async fn get_user_bets(
    state: State<AppDBState>,
    page: Query<u64>,
    user_id: Query<String>,
    game_name: Query<Option<String>>
) -> APIResult<Json<Value>> {

if user_id.0 == "" {
    return Err(Error::NoUserIdFound)
}

let game_bets_vec = if game_name.0.is_some() {
    game_bets::Entity::find_by_user_id_and_game_name(game_name.0.unwrap(), Uuid::parse_str(&user_id.0).unwrap())
    .offset(Some(page.0))
    .limit(20)
    .all(&state.conn)
    .await
} else {
    game_bets::Entity::find_by_user_id(Uuid::parse_str(&user_id.0).unwrap())
    .offset(Some(page.0))
    .limit(20)
    .all(&state.conn)
    .await
};


if game_bets_vec.is_err() {
    return Err(Error::ErrorWhileFetchingUserBets)
}


let body = Json(json!({
    "result": {
        "success": true
    },
    "game_bets": game_bets_vec.unwrap()
}));

Ok(body)


}

