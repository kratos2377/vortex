use std::str::FromStr;

use crate::{errors::{self, Error}, state::AppDBState};

use axum::{extract::{Path, Query, State}, Json};
use chrono::Utc;
use errors::Result as APIResult;
use orion::models::{game_bet_events::GameBetStatus, game_model::Game};
use sea_orm::{QuerySelect, Set};
use serde::Deserialize;
use serde_json::{json, Value};
use ton::models::game_bets;
use uuid::Uuid;
use sea_orm::ActiveModelTrait;
use super::payload::{PlaceUserBetPayload, UpdateUserPlaceBetPayload};


#[derive(Deserialize)]
pub struct UserGetBetCallParams {
    user_id: String,
    wallet_key: String,
    page: u64,
}


pub async fn get_user_bets(
    Path(params): Path<UserGetBetCallParams>,
    state: State<AppDBState>
) -> APIResult<Json<Value>> {


    if params.user_id == "" || params.wallet_key == "" {
        return Err(Error::MissingParams)
    }
    
    let game_bets_vec = 
        game_bets::Entity::find_by_user_id_and_wallet_key(params.wallet_key, Uuid::parse_str(&params.user_id).unwrap())
        .offset(Some(params.page))
        .limit(20)
        .all(&state.conn)
        .await;
    
    if game_bets_vec.is_err() {
        return Err(Error::ErrorWhileFetchingUserBets)
    }
    

let game_bet_vec_rec = game_bets_vec.unwrap();

let body = Json(json!({
    "result": {
        "success": true
    },
    "game_bets": game_bet_vec_rec,
    "has_more": game_bet_vec_rec.len() == 20
}));

Ok(body)


}

