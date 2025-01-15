use std::str::FromStr;

use crate::{errors::{self, Error}, state::AppDBState};

use axum::{extract::{State, Query}, Json};
use chrono::Utc;
use errors::Result as APIResult;
use mongodb::bson::{self, doc, DateTime};
use orion::models::{game_bet_events::GameBetStatus, game_model::Game};
use sea_orm::{QuerySelect, Set};
use serde_json::{json, Value};
use ton::models::game_bets;
use uuid::Uuid;
use sea_orm::ActiveModelTrait;
use bson::Uuid as BsonUuid;
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


pub async fn place_user_bet(
    state: State<AppDBState>,
    payload: Json<PlaceUserBetPayload>
) -> APIResult<Json<Value>> {
    if payload.user_id == "" || payload.game_id == "" || payload.game_name == "" || payload.session_id == "" || payload.user_id_betting_on == "" {
        return Err(Error::MissingParams)
    }


    let game_collection = state.mongo_conn.database("user_game_events_db").collection::<Game>("games");
    
    let game_model_res = game_collection.find_one(doc! { "id": BsonUuid::parse_str(payload.game_id.clone()).unwrap()},None).await;

    if game_model_res.is_err() {
        return Err(Error::ErrorWhileFetchingGameModel)
    }

    let game_model = game_model_res.unwrap();

    if game_model.is_none() {
        return Err(Error::ErrorWhileFetchingGameModel)
    }

    let elapsed_time = DateTime::now().checked_duration_since(game_model.unwrap().created_at).unwrap();

    if elapsed_time.as_secs() > 302 {
        return Err(Error::ErrorWhileFetchingGameModel)
    }

    let game_new_model_res = game_bets::Entity::find_by_user_id_and_game_id(Uuid::from_str(&payload.game_id).unwrap(), Uuid::from_str(&payload.user_id).unwrap()).all(&state.conn).await;


    if game_new_model_res.is_err() {
        return Err(Error::ErrorWhilePlacingBet)
    }

    if !game_new_model_res.unwrap().is_empty() {
        return Err(Error::CannotPlaceNewBetForSameGame)
    }

    let game_bets_model = game_bets::ActiveModel {
        id: Set(Uuid::new_v4()),
        user_id: Set(Uuid::parse_str(&payload.user_id).unwrap()),
        game_id: Set(Uuid::parse_str(&payload.game_id).unwrap()),
        game_name: Set(payload.game_name.clone()),
        bet_amount: Set(payload.bet_amount),
        status: Set(GameBetStatus::InProgress.to_string()),
        session_id: Set(payload.session_id.clone()),
        created_at: Set(Utc::now().naive_utc()),
        updated_at: Set(Utc::now().naive_utc()),
        user_id_betting_on: Set(Uuid::parse_str(&payload.user_id_betting_on).unwrap()),
    };
    
    
    let game_bets_insert_res = game_bets_model.insert(&state.conn).await;
    
    if game_bets_insert_res.is_err() {
        return Err(Error::ErrorWhilePlacingBet)
    }

    //Add solana contract layer to create bet

    let body = Json(json!({
        "result": {
            "success": true
        },
    }));
    
    Ok(body)
}
    
    
pub async fn update_user_bet(
    state: State<AppDBState>,
    payload: Json<UpdateUserPlaceBetPayload>
) -> APIResult<Json<Value>> {
    if payload.user_id == "" || payload.game_id == ""  {
        return Err(Error::MissingParams)
    }

    if payload.add_more_amount < 0.0  {
        return Err(Error::InvalidParams);
    }


    let game_collection = state.mongo_conn.database("user_game_events_db").collection::<Game>("games");
    
    let game_model_res = game_collection.find_one(doc! { "id": BsonUuid::parse_str(payload.game_id.clone()).unwrap()},None).await;

    if game_model_res.is_err() {
        return Err(Error::ErrorWhileFetchingGameModel)
    }

    let game_model = game_model_res.unwrap();

    if game_model.is_none() {
        return Err(Error::ErrorWhileFetchingGameModel)
    }

    let elapsed_time = DateTime::now().checked_duration_since(game_model.unwrap().created_at).unwrap();

    if elapsed_time.as_secs() > 302 {
        return Err(Error::ErrorWhileFetchingGameModel)
    }

    let game_bets_res = game_bets::Entity::find_by_id(Uuid::from_str(&payload.user_id).unwrap()).one(&state.conn).await;

    if game_bets_res.is_err() {
        return Err(Error::ErrorWhileFetchingExistingBet)
    }

    let game_bet_model = game_bets_res.unwrap();

    if game_bet_model.is_none() {
        return Err(Error::ErrorWhileFetchingExistingBet)
    }
    let game_model_unwrapped = game_bet_model.unwrap();
    let final_bet_amount = game_model_unwrapped.bet_amount + payload.add_more_amount;
    let mut game_bet_model_result: game_bets::ActiveModel = game_model_unwrapped.into();

    game_bet_model_result.bet_amount = Set(final_bet_amount);

    let update_res = game_bet_model_result.update(&state.conn).await;

    if update_res.is_err() {
        return Err(Error::ErrorWhileUpdatingGameBet)
    }

    //Add solana contract layer to update bet


    let body = Json(json!({
        "result": {
            "success": true
        },
    }));
    
    Ok(body)
}
    
