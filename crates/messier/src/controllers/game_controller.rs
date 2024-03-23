use crate::{errors::{self, Error}, state::{AppDBState}};
use axum::{extract::{ State}, response::Response, Json};
use redis::{Commands, RedisResult};
use serde_json::{json, Value};
use errors::Result as APIResult;
use uuid::Uuid;

use super::payloads::{CreateLobbyPayload, DestroyLobbyPayload, JoinLobbyPayload, SpectateGamePayload, VerifyGameStatusPayload};


pub async fn create_lobby(
    state: State<AppDBState>,
	payload: Json<CreateLobbyPayload>,
) -> APIResult<Json<Value>> {
    let game_id = Uuid::new_v4();
    let arc_redis_client = state.context.get_redis_db_client();
    let mut redisConnection  = arc_redis_client.lock().unwrap();

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

    let arc_redis_client = state.context.get_redis_db_client();
    let mut redisConnection  = arc_redis_client.lock().unwrap();

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

    let arc_redis_client = state.context.get_redis_db_client();
    let mut redisConnection  = arc_redis_client.lock().unwrap();
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

    let arc_redis_client = state.context.get_redis_db_client();
    let mut redisConnection  = arc_redis_client.lock().unwrap();

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
    let arc_redis_client = state.context.get_redis_db_client();
    let mut redisConnection  = arc_redis_client.lock().unwrap();

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

pub async fn start_spectating_game(
    state: State<AppDBState>,
	payload: Json<SpectateGamePayload>,
) -> APIResult<Json<Value>> {
    let arc_redis_client = state.context.get_redis_db_client();
    let mut redisConnection  = arc_redis_client.lock().unwrap();

    
    let result: RedisResult<()> =  redisConnection.hset(&payload.game_id, &payload.user_id, &payload.socket_id) ;

    if result.is_err() {
        return Err(Error::SpectateGameJoinError)
    }


    let body = Json(json!({
		"result": {
			"success": true
		}
	}));

	Ok(body)
}

pub async fn stop_spectating_game(
    state: State<AppDBState>,
	payload: Json<SpectateGamePayload>,
) -> APIResult<Json<Value>> {
    let arc_redis_client = state.context.get_redis_db_client();
    let mut redisConnection  = arc_redis_client.lock().unwrap();

    
    let result: RedisResult<()> =  redisConnection.hdel(&payload.game_id, &payload.user_id) ;

    if result.is_err() {
        return Err(Error::SpectateGameLeaveError)
    }


    let body = Json(json!({
		"result": {
			"success": true
		}
	}));

	Ok(body)
}




