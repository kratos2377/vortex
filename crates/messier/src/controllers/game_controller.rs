use std::str::FromStr;

use crate::{context, errors::{self, Error}, event_producer::user_events_producer::send_event_for_user_topic, state::AppDBState};
use axum::{extract::{ State}, response::Response, Json};
use bson::{doc, Document};
use futures::TryStreamExt;
use orion::{constants::GAME_INVITE_EVENT, events::kafka_event::UserGameInviteKafkaEvent, models::{game_model::Game, user_game_relation_model::UserGameRelation}};
use redis::{Commands, RedisResult};
use serde_json::{json, Value};
use errors::Result as APIResult;
use uuid::Uuid;
use bson::Uuid as BsonUuid;
use super::payloads::{CreateLobbyPayload, DestroyLobbyPayload, GetGameCurrentStatePayload, GetUsersOngoingGamesPayload, GetUsersOngoingGamesResponseModel, JoinLobbyPayload, SendGameEventAPIPayload, VerifyGameStatusPayload};


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


pub async fn send_game_invite(
    state: State<AppDBState>,
	payload: Json<SendGameEventAPIPayload>,
) -> APIResult<Json<Value>> {

    if &payload.game_id == "" || &payload.game_name == "" || payload.user_receiving_id == "" || payload.user_sending_id == "" || payload.user_sending_username == "" {
        return Err(Error::MissingParamsError)
    }

    let kafka_event = UserGameInviteKafkaEvent {
        user_who_send_request_id: payload.user_sending_id.clone(),
        user_who_send_request_username: payload.user_sending_username.clone(),
        user_who_we_are_sending_event: payload.user_receiving_id.clone(),
        game_id: payload.game_id.clone(),
        game_name: payload.game_name.clone(),

    };
    let res = send_event_for_user_topic(&state.producer , &state.context , GAME_INVITE_EVENT.to_string() , serde_json::to_string(&kafka_event).unwrap() ).await;

    if res.is_err() {
        return Err(Error::GameInviteSendError)
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


pub async fn get_ongoing_games_for_user(
    state: State<AppDBState>,
    payload: Json<GetUsersOngoingGamesPayload>
) -> APIResult<Json<Value>> {
    if  payload.user_friends_ids.is_empty() {
        return Err(Error::MissingParamsError)
    }

    //Database name will change 
    let mongo_db = state.context.get_mongo_db_client().database("user_game_events_db");

    let user_collection = mongo_db.collection::<UserGameRelation>("users");
    let game_collection = mongo_db.collection::<Game>("games");
    let mut game_vec_results: Vec<GetUsersOngoingGamesResponseModel> = vec![];
    for user_id in payload.user_friends_ids.iter() {
        let cursor = user_collection.find(doc! { "user_id": BsonUuid::parse_str(user_id).unwrap() }, None).await.unwrap();
        let res: Vec<UserGameRelation> = cursor.try_collect().await.unwrap();

        if let Some(game_id) = &res.get(0).unwrap().game_id {

            let game_cursor = game_collection.find(doc! { "id": BsonUuid::parse_str(game_id).unwrap() }, None).await.unwrap();
            let game_res: Vec<Game> = game_cursor.try_collect().await.unwrap();
            let new_game_res = game_res.get(0).unwrap();
            let new_game = GetUsersOngoingGamesResponseModel {
                game_id: new_game_res.id,
                game_type: new_game_res.game_type.clone(),
                is_staked: new_game_res.is_staked,
                total_money_staked: 0.0,
            };

            game_vec_results.push(new_game);
        }

    }

    

    let body = Json(json!({
		"result": {
			"success": true,
            "games": game_vec_results
		}
	}));

	Ok(body)

}


pub async fn get_current_state_of_game(
    state: State<AppDBState>,
    payload: Json<GetGameCurrentStatePayload>
) -> APIResult<Json<Value>> {
    if  payload.game_id.to_string() == "" {
        return Err(Error::MissingParamsError)
    }

    //Database name will change 
    let mongo_db = state.context.get_mongo_db_client().database("user_game_events_db");

    let game_collection = mongo_db.collection::<Game>("games");

    let game_cursor = game_collection.find(doc! { "id": payload.game_id }, None).await.unwrap();
    let game_res: Vec<Game> = game_cursor.try_collect().await.unwrap();
    let game_res = game_res.get(0).unwrap();


    let game_current_state = if game_res.name == "chess" {
        &game_res.current_state
    } else {
        // Change this to poker state later
        &game_res.current_state
    };



    let body = Json(json!({
		"result": {
			"success": true,
            "game_state": game_current_state
		}
	}));

	Ok(body)
}



pub async fn stake_in_game(
    state: State<AppDBState>,
    payload: Json<GetGameCurrentStatePayload>
) -> APIResult<Json<Value>> {
    todo!()
}

