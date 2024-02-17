use crate::{errors::{self, Error}, state::{AppDBState}};
use axum::{extract::{ State}, response::Response, Json};
use redis::{Commands, RedisResult};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use errors::Result as APIResult;
use uuid::Uuid;
#[derive(Clone, Debug, Deserialize)]
pub struct CreateLobbyPayload {
    pub user_id: String
}


#[derive(Clone, Debug, Deserialize)]
pub struct JoinLobbyPayload {
    pub user_id: String,
    pub game_id: String,
}


#[derive(Clone, Debug, Deserialize)]
pub struct VerifyGameStatusPayload {
    pub game_id: String,
    pub game_name: String,
}


#[derive(Clone, Debug, Deserialize)]
pub struct DestroyLobbyPayload {
    pub game_id: String,
}


#[derive(Clone, Debug, Deserialize)]
pub struct BrodcastGamePayload {
    pub game_id: String,
    pub user_id: String,
    pub event_name: String,
}


#[derive(Deserialize)]
pub struct WebsocketPayload {
    pub user_id: String,
    pub game_name: String,
    pub event: String
}


#[derive(Serialize , Deserialize)]

pub struct WebSocketResponse {
    pub event_name: String,
    pub game_name: String,
    pub user_id: String,
    pub event_move: String,
    pub next_user_turn: String,
}


pub async fn create_lobby(
    state: State<AppDBState>,
	payload: Json<CreateLobbyPayload>,
) -> APIResult<Json<Value>> {
    let game_id = Uuid::new_v4();
    let mut redisConnection  = state.redis_connection.lock().unwrap();

    let lobby_user_ids = vec![payload.user_id.clone()];
    let create_result: RedisResult<()> = redisConnection.set(game_id.to_string(), lobby_user_ids);

    if create_result.is_err() {
        return Err(Error::CreateLobbyError)
    }


    let body = Json(json!({
		"result": {
			"success": true
		},
        "game_id": game_id
	}));

	Ok(body)

}

pub async fn join_lobby(
    state: State<AppDBState>,
	payload: Json<JoinLobbyPayload>,
) -> APIResult<Json<Value>> { 

    let mut redisConnection  = state.redis_connection.lock().unwrap();

    let mut get_game_result = redisConnection.hkeys(&payload.game_id);
    if get_game_result.is_err() {
        return Err(Error::JoinLobbyError)
    }

   let mut lobby_user_ids: Vec<String> = get_game_result.unwrap();
   if *(&lobby_user_ids.len()) == 4 {
    return Err(Error::LobbyFull)
   }
    lobby_user_ids.push(payload.user_id.clone());

    
    let create_result: RedisResult<()> = redisConnection.set(&payload.game_id, lobby_user_ids);

    if create_result.is_err() {
        return Err(Error::CreateLobbyError)
    }

    let body = Json(json!({
		"result": {
			"success": true
		}
	}));

	Ok(body)

}

pub async fn verify_game_status(
    state: State<AppDBState>,
	payload: Json<VerifyGameStatusPayload>,
) -> APIResult<Json<Value>> { 

    if &payload.game_id == "" || &payload.game_name == "" {
        return Err(Error::MissingParamsError)
    }

    let mut redisConnection = state.redis_connection.lock().unwrap();
    let mut get_game_result = redisConnection.hkeys(&payload.game_id);

    if get_game_result.is_err() {
        return Err(Error::RedisUnwrapError)
    }

   let mut lobby_user_ids: Vec<String> = get_game_result.unwrap();
 

    
    if lobby_user_ids.len() != 2 && &payload.game_name == "chess" {
        return Err(Error::GameCannotBeStarted)
    } else if lobby_user_ids.len() < 2  {
        return Err(Error::GameCannotBeStarted)
    }

    let body = Json(json!({
		"result": {
			"success": true
		}
	}));

	Ok(body)

}

pub async fn remove_user_from_lobby(
    state: State<AppDBState>,
	payload: Json<JoinLobbyPayload>,
) -> APIResult<Json<Value>> { 

    let mut redisConnection  = state.redis_connection.lock().unwrap();

    let get_game_result = redisConnection.hkeys(&payload.game_id);
    if get_game_result.is_err() {
        return Err(Error::RemoveFromLobbyError)
    }

   let lobby_user_ids: Vec<String> = get_game_result.unwrap();
    let new_lobby_user_ids: Vec<_> = lobby_user_ids.into_iter().filter(|x| x != &payload.user_id).collect();
    let create_result: RedisResult<()> = redisConnection.set(&payload.game_id, new_lobby_user_ids);

    if create_result.is_err() {
        return Err(Error::RemoveFromLobbyError)
    }

    let body = Json(json!({
		"result": {
			"success": true
		} 
	}));

	Ok(body)

}



pub async fn destroy_lobby_and_game(
    state: State<AppDBState>,
	payload: Json<DestroyLobbyPayload>,
) -> APIResult<Json<Value>> {
    let mut redisConnection  = state.redis_connection.lock().unwrap();

    let delete_query_result: RedisResult<()> = redisConnection.del(&payload.game_id);
    if delete_query_result.is_err() {
        return Err(Error::DeleteLobbyError)
    }

    let body = Json(json!({
		"result": {
			"success": true
		}
	}));

	Ok(body)
}

