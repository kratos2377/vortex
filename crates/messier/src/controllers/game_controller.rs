use std::{collections::{HashMap, HashSet}, str::FromStr};

use crate::{context, errors::{self, Error}, event_producer::user_events_producer::send_event_for_user_topic, state::AppDBState};
use axum::{extract::{ State}, response::Response, Json};
use axum_macros::debug_handler;
use bson::{doc, Document};
use futures::{StreamExt, TryStreamExt};
use mongodb::options::FindOptions;
use orion::{constants::{GAME_INVITE_EVENT, REDIS_USER_GAME_KEY, REDIS_USER_PLAYER_KEY}, events::kafka_event::UserGameInviteKafkaEvent, models::{game_model::Game, user_game_relation_model::UserGameRelation, user_turn_model::{TurnModel, UserTurnMapping}}};
use redis::{Commands, Connection, RedisResult};
use sea_orm::TryIntoModel;
use std::sync::{Arc, Mutex};
use serde_json::{json, Value};
use errors::Result as APIResult;
use ton::models;
use uuid::Uuid;
use bson::Uuid as BsonUuid;
use models::{users_friends_requests::{self , Entity as UsersFriendsRequests}, users_friends::{self, Entity as UsersFriends}, users::{Entity as Users}};
use super::payloads::{CreateLobbyPayload, DestroyLobbyPayload, GetGameCurrentStatePayload, GetLobbyPlayersPayload, GetUserTurnMappingsPayload, GetUsersOngoingGamesPayload, GetUsersOngoingGamesResponseModel, JoinLobbyPayload, RemoveGameModelsPayload, SendGameEventAPIPayload, StartGamePayload, UpdatePlayerStatusPayload, VerifyGameStatusPayload};


pub async fn create_lobby(
    state: State<AppDBState>,
	payload: Json<CreateLobbyPayload>,
) -> APIResult<Json<Value>> {

    if payload.user_id == "" || payload.game_name == "" || payload.game_type == ""  || payload.username == "" {
        return  Err(Error::MissingParamsError);
    }

    let game_id = Uuid::new_v4();
    let arc_redis_client = state.context.get_redis_db_client();
    let mongo_db = state.context.get_mongo_db_client().database("user_game_events_db");

    let create_result = set_key_from_redis_int(&arc_redis_client , game_id.to_string() + "-game-id-count" , 1);

    if create_result.is_err() {
        return Err(Error::CreateLobbyError)
    }

    let user_collection = mongo_db.collection::<UserGameRelation>("users");
    let game_collection = mongo_db.collection::<Game>("games");
    let user_turn_collection = mongo_db.collection::<UserTurnMapping>("user_turns");

    let user_doc = UserGameRelation {
        user_id: Uuid::from_str(&payload.user_id).unwrap(),
        username: payload.username.clone(),
        game_id: game_id.to_string().clone(),
        player_type: "host".to_string(),
        player_status: "not-ready".to_string(),
    };

    let game_doc = Game {
        id: game_id,
        user_count: 1,
        host_id: payload.user_id.clone(),
        name: payload.game_name.clone(),
        game_type: payload.game_name.clone(),
        is_staked: payload.game_type == "staked",
        current_state: "none".to_string(),
        state_index: 0,
        description: "none".to_string(),
        staked_money_state: None,
        poker_state: None,
    };
    let user_turn_mapping_doc = UserTurnMapping {
        game_id: game_id.to_string().clone(),
        turn_mappings: vec![TurnModel { count_id: 1, user_id: payload.user_id.clone(), username: payload.username.clone() }],
        host_id: payload.user_id.clone(),
    };
    let _: RedisResult<()> = set_key_from_redis(&arc_redis_client, payload.user_id.clone() + REDIS_USER_GAME_KEY, game_id.to_string());
    let _: RedisResult<()> = set_key_from_redis(&arc_redis_client, payload.user_id.clone() + REDIS_USER_PLAYER_KEY, "host".to_string());
   let user_mongo_result =  user_collection.insert_one(user_doc, None).await;
   let game_mongo_result =  game_collection.insert_one(game_doc, None).await;
    let user_turn_mongo_result = user_turn_collection.insert_one(user_turn_mapping_doc, None).await;

   if user_mongo_result.is_err() || game_mongo_result.is_err() || user_turn_mongo_result.is_err() {
    return Err(Error::ErrorWhileCreatingEntities)
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

    if payload.user_id == "" || payload.game_id == "" || payload.game_name == "" || payload.username == "" {
        return Err(Error::MissingParamsError)
    } 

    let arc_redis_client = &state.context.get_redis_db_client();
    let mongo_db = state.context.get_mongo_db_client().database("user_game_events_db");   
    let _: RedisResult<()> =  set_key_from_redis(&arc_redis_client, payload.user_id.clone() + REDIS_USER_GAME_KEY, payload.game_id.clone());
    let _: RedisResult<()> = set_key_from_redis(&arc_redis_client, payload.user_id.clone() + REDIS_USER_PLAYER_KEY, "player".to_string());
    let mut get_game_result = get_key_from_redis_int(arc_redis_client.clone(), payload.game_id.to_string() + "-game-id-count");
    incr_redis_key(&arc_redis_client, payload.game_id.to_string() + "-game-id-count");
    if get_game_result.is_err() {
        return Err(Error::JoinLobbyError)
    }

    let cnt = get_game_result.unwrap();
    let user_cnt_id = cnt + 1;

   let game_collection = mongo_db.collection::<Game>("games");
   let user_collection = mongo_db.collection::<UserGameRelation>("users");
   let user_turn_collection = mongo_db.collection::<UserTurnMapping>("user_turns");
   let game_res = game_collection.find(doc! { "id": BsonUuid::parse_str(payload.game_id.clone()).unwrap() }, None).await;

   if game_res.is_err() {
    return Err(Error::ErrorWhileFetchingGameDetails);
   }

   let game_model: Vec<Game> = game_res.unwrap().try_collect().await.unwrap();
   let game_res = game_model.get(0);

   if game_res.is_none() {
    return Err(Error::GameLobbyDeletedOrRequestIsInvalid)
   }
   let game = game_res.unwrap();

   if payload.game_name == "chess" && game.user_count >= 2 {
    return Err(Error::LobbyIsFull)
   }

   if game.user_count >= 10 {
    return Err(Error::LobbyIsFull)
   }

   let user_doc = UserGameRelation {
    user_id: Uuid::from_str(&payload.user_id).unwrap(),
    username: payload.username.clone(),
    game_id: payload.game_id.clone(),
    player_type: "player".to_string(),
    player_status: "not-ready".to_string(),
};




   let user_insert_res = user_collection.insert_one(user_doc, None).await;
   let game_update_doc = game_collection.update_one(doc! { "id": payload.game_id.clone() }, doc! { "$set": doc! {"user_count": game.user_count + 1} }, None).await;
   let turn_update_doc = user_turn_collection.update_one(doc! { "game_id": payload.game_id.clone()}, doc! {
    "$push": {
        "turn_mappings": {
            "count_id": user_cnt_id,
            "user_id": payload.user_id.clone(),
            "username": payload.username.clone()
        }
    } }, None).await;
    if user_insert_res.is_err() || game_update_doc.is_err() || turn_update_doc.is_err() {
        return Err(Error::ErrorWhileUpdatingMongoUserAndGame)
    }

    let body = Json(json!({
		"result": {
			"success": true
		},
        "game_host_id": game.host_id,
        "user_count_id": user_cnt_id
	}));

	Ok(body)

}


pub async fn send_game_invite(
    state: State<AppDBState>,
	payload: Json<SendGameEventAPIPayload>,
) -> APIResult<Json<Value>> {

    if &payload.game_type== "" || &payload.game_id == "" || &payload.game_name == "" || payload.user_receiving_id == "" || payload.user_sending_id == "" || payload.user_sending_username == "" {
        return Err(Error::MissingParamsError)
    }

    let kafka_event = UserGameInviteKafkaEvent {
        user_who_send_request_id: payload.user_sending_id.clone(),
        user_who_send_request_username: payload.user_sending_username.clone(),
        user_who_we_are_sending_event: payload.user_receiving_id.clone(),
        game_id: payload.game_id.clone(),
        game_name: payload.game_name.clone(),
        game_type: payload.game_type.clone(),

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

pub async fn leave_lobby(
    state: State<AppDBState>,
	payload: Json<JoinLobbyPayload>,
) -> APIResult<Json<Value>> { 

    if payload.user_id == "" || payload.game_id == "" || payload.game_name =="" {
        return Err(Error::MissingParamsError)
    }

    let arc_redis_client = state.context.get_redis_db_client();
    let _: RedisResult<()> =  delete_key_from_redis(&arc_redis_client, payload.user_id.clone() + REDIS_USER_GAME_KEY);
    let _: RedisResult<()> = delete_key_from_redis(&arc_redis_client, payload.user_id.clone() + REDIS_USER_PLAYER_KEY);

    let mongo_db = state.context.get_mongo_db_client().database("user_game_events_db");
    

    let user_collection = mongo_db.collection::<UserGameRelation>("users");
    let game_collection = mongo_db.collection::<Game>("games");
    let user_turn_collection = mongo_db.collection::<UserTurnMapping>("user_turns");

    let user_rsp = user_collection.delete_one(doc! { "game_id": payload.game_id.clone(), "user_id": payload.user_id.clone() }, None).await;
    
    let game_rsp = game_collection.update_one(doc! { "id": payload.game_id.clone() }, doc! { "$inc": { "user_count": -1 } }, None).await;
    
    let user_turn_rsp = user_turn_collection.update_one(doc! { "game_id": payload.game_id.clone()}, doc! {
        "$pull": {
            "turn_mappings": {
                "user_id": payload.user_id.clone()
            }
        }
    }, None).await;

    if user_rsp.is_err() || game_rsp.is_err() || user_turn_rsp.is_err() {
        return Err(Error::ErrorWhileLeavingLobby)
    }

    // let res = send_event_for_user_topic(&state.producer , &state.context , GAME_INVITE_EVENT.to_string() , serde_json::to_string(&kafka_event).unwrap() ).await;


    // if res.is_err() {
    //     return Err(Error::ErrorWhileSendingLeaveKafkaEvent)
    // }
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
    if payload.game_id == "" {
        return Err(Error::MissingParamsError)
    }
    let mongo_db = state.context.get_mongo_db_client().database("user_game_events_db");
    let user_collection = mongo_db.collection::<UserGameRelation>("users");
    let game_collection = mongo_db.collection::<Game>("games");
    let user_turns_collection = mongo_db.collection::<UserTurnMapping>("user_turns");


    let game_Delete_query = game_collection.delete_one(doc! { "id": BsonUuid::parse_str(payload.game_id.clone()).unwrap()}, None).await;
    let user_delete_query = user_collection.delete_many(doc! { "game_id": payload.game_id.clone()}, None).await;
    let user_turn_delete_rsp = user_turns_collection.delete_one(doc! { "game_id": payload.game_id.clone()}, None).await;
    if game_Delete_query.is_err() || user_delete_query.is_err() || user_turn_delete_rsp.is_err() {
        return Err(Error::DeleteLobbyError)
    }

    let body = Json(json!({
		"result": {
			"success": true
		}
	}));

	Ok(body)
}

pub async fn get_lobby_players(
    state: State<AppDBState>,
	payload: Json<GetLobbyPlayersPayload>,
) -> APIResult<Json<Value>> {

    if payload.game_id == "" || payload.host_user_id == "" {
        return Err(Error::MissingParamsError)
    }

    let mongo_db = state.context.get_mongo_db_client().database("user_game_events_db");
    let user_collection = mongo_db.collection::<UserGameRelation>("users");
    let user_models_resp = user_collection.find(doc! { "game_id": payload.game_id.clone()}, None).await;

    if user_models_resp.is_err() {
        return Err(Error::ErrorWhileRetrievingLobbyUsers)
    }

    let user_models: Vec<UserGameRelation> = user_models_resp.unwrap().try_collect().await.unwrap();
    let body = Json(json!({
		"result": {
			"success": true
		},
        "lobby_users": user_models
	}));

	Ok(body)
}


pub async fn get_ongoing_games_for_user(
    state: State<AppDBState>,
    payload: Json<GetUsersOngoingGamesPayload>
) -> APIResult<Json<Value>> {
    if  payload.user_id == "" {
        return Err(Error::MissingParamsError)
    }

    let get_user_friends_ids = UsersFriends::find_by_user_id(&Uuid::from_str(&payload.user_id).unwrap()).all(&state.conn).await;

    if get_user_friends_ids.is_err() {
        return Err(Error::ErrorWhileFetchingUserFriends)
    }

    let get_user_friends_ids_vec = get_user_friends_ids.unwrap();
    let mut game_sets: HashSet<String> = HashSet::new();
    let mut game_id_gamemodel: HashMap<String, GetUsersOngoingGamesResponseModel> = HashMap::new();
    //Database name will change 
    let mongo_db = state.context.get_mongo_db_client().database("user_game_events_db");

    let user_collection = mongo_db.collection::<UserGameRelation>("users");
    let game_collection = mongo_db.collection::<Game>("games");
    let mut game_vec_results: Vec<GetUsersOngoingGamesResponseModel> = vec![];
    for user_model in get_user_friends_ids_vec.iter() {
        let usr_model = user_model.clone().try_into_model().unwrap();
        let cursor = user_collection.find(doc! { "user_id": BsonUuid::parse_str(usr_model.friend_id.to_string()).unwrap() }, None).await;
      
        if cursor.is_err() {
            continue;
        }
        let res: Vec<UserGameRelation> = cursor.unwrap().try_collect().await.unwrap();

        if res.len() == 0 {
            continue;
        }
        let user_game_rel = res.get(0);
        if user_game_rel.is_none() {
            continue;
        }

        let user_game_rel_model = user_game_rel.unwrap();
      
        let game_id = &user_game_rel_model.game_id.clone();
            if !game_sets.contains(game_id) {
                game_sets.insert(game_id.to_string());
                let game_cursor = game_collection.find(doc! { "id": BsonUuid::parse_str(game_id).unwrap() }, None).await.unwrap();
                let game_res: Vec<Game> = game_cursor.try_collect().await.unwrap();
                let new_game_res = game_res.get(0).unwrap();
                let new_game = GetUsersOngoingGamesResponseModel {
                    game_id: new_game_res.id,
                    game_type: new_game_res.game_type.clone(),
                    is_staked: new_game_res.is_staked,
                    total_money_staked: 0.0,
                    usernames_playing: vec![user_game_rel_model.username.clone()],
                };
                game_id_gamemodel.insert(game_id.clone(),  new_game);
            } else {
                 let mut game_model = game_id_gamemodel.get_mut(game_id).unwrap();
                 game_model.usernames_playing.push(user_game_rel_model.username.clone())
            }
            
        

    }

    for (_,value ) in game_id_gamemodel {
        game_vec_results.push(value);
    }

    

    let body = Json(json!({
		"result": {
			"success": true,
		},
        "games": game_vec_results
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
		},
        "game_state": game_current_state
	}));

	Ok(body)
}


pub async fn update_player_status(
    state: State<AppDBState>,
    payload: Json<UpdatePlayerStatusPayload>
) -> APIResult<Json<Value>> {
        if &payload.game_id == "" || &payload.game_name == "" || &payload.user_id == "" {
            return Err(Error::MissingParamsError)
        }
        let status_up = &payload.status.to_ascii_lowercase();
        if  status_up != "ready" && status_up != "not-ready" {
            return Err(Error::InvalidStatusSendAsPayload)
        }

        let mongo_db = state.context.get_mongo_db_client().database("user_game_events_db");
        let user_collection = mongo_db.collection::<UserGameRelation>("users");
        let user_model = user_collection.update_one(doc! { "user_id": BsonUuid::parse_str(payload.user_id.clone()).unwrap(), "game_id": payload.game_id.clone()}, doc! { "$set": doc! {"player_status": status_up} } ,None).await;
       
if user_model.is_err() {
    return Err(Error::ErrorWhileUpdatingPlayerStatus)
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
    payload: Json<VerifyGameStatusPayload>
) -> APIResult<Json<Value>> {
        if &payload.game_id == "" || &payload.host_user_id == "" || &payload.game_name == "" {
            return Err(Error::MissingParamsError)
        }
   

        let mongo_db = state.context.get_mongo_db_client().database("user_game_events_db");
        let game_collection = mongo_db.collection::<Game>("games");
        let game_model = game_collection.find_one(doc! { "host_id": payload.host_user_id.clone(), "id": BsonUuid::parse_str(payload.game_id.clone()).unwrap(), "name": payload.game_name.clone()},None).await;
       
if game_model.is_err() {
    return Err(Error::GameNotFound)
}

        let body = Json(json!({
            "result": {
                "success": true
            }
        }));
    
        Ok(body)
}

pub async fn remove_game_models(
    state: State<AppDBState>,
    payload: Json<RemoveGameModelsPayload>
) -> APIResult<Json<Value>> {
        if &payload.game_id == "" || &payload.host_user_id == "" || &payload.game_name == "" || payload.user_id == "" {
            return Err(Error::MissingParamsError)
        }
   
        let  redis_conn = state.context.get_redis_db_client();
        let _: RedisResult<()> = delete_key_from_redis(&redis_conn, payload.user_id.clone() + REDIS_USER_GAME_KEY);

        let mongo_db = state.context.get_mongo_db_client().database("user_game_events_db");
        let user_collection = mongo_db.collection::<UserGameRelation>("users");
        let game_collection = mongo_db.collection::<Game>("games");
        let user_model = user_collection.find_one(doc! { "user_id": BsonUuid::parse_str(payload.user_id.clone()).unwrap() }, None).await.unwrap().unwrap();
       
       if user_model.player_type == "host" {
        let _ = game_collection.delete_one(doc! { "host_id": payload.host_user_id.clone(), "id": BsonUuid::parse_str(payload.game_id.clone()).unwrap(), "name": payload.game_name.clone()},None).await;
       }
       
       let user_rsp = user_collection.delete_one(doc! { "user_id": BsonUuid::parse_str(payload.user_id.clone()).unwrap(), "game_id": payload.game_id.clone()}, None).await;
       
if user_rsp.is_err() {
    return Err(Error::GameNotFound)
}

        let body = Json(json!({
            "result": {
                "success": true
            }
        }));
    
        Ok(body)
}


pub async fn start_game(
    state: State<AppDBState>,
    payload: Json<StartGamePayload>
) -> APIResult<Json<Value>> {
        if &payload.game_id == "" || &payload.game_name == "" {
            return Err(Error::MissingParamsError)
        }

        let mongo_db = state.context.get_mongo_db_client().database("user_game_events_db");
        let user_collection = mongo_db.collection::<UserGameRelation>("users");
        let user_vec = user_collection.find(doc! { "game_id": payload.game_id.clone()}, None).await;
       
if user_vec.is_err() {
    return Err(Error::ErrorWhileRetrievingPlayersStatus)
}
let user_vec: Vec<UserGameRelation> = user_vec.unwrap().try_collect().await.unwrap();



for user in user_vec.iter() {
    if user.player_status == "not-ready" {
        return Err(Error::NotAllPlayersHaveReadyStatus);
    }
}

        let body = Json(json!({
            "result": {
                "success": true
            }
        }));
    
        Ok(body)
}

pub async fn get_user_turn_mappings(
    state: State<AppDBState>,
    payload: Json<GetUserTurnMappingsPayload>
) -> APIResult<Json<Value>> {

    if payload.game_id == "" {
        return Err(Error::MissingParamsError)
    }

    let mongo_db = state.context.get_mongo_db_client().database("user_game_events_db");
    let user_turns_collection = mongo_db.collection::<UserTurnMapping>("user_turns");


    let sort = doc! {
        "turn_mappings.count_id": 1 // 1 for ascending, -1 for descending
    };
    let options = FindOptions::builder()
        .sort(sort)
        .build();

    let user_turn_model = user_turns_collection.find(doc! { "game_id": payload.game_id.clone() }, options).await;

    if user_turn_model.is_err() {
        return Err(Error::ErrorWhileFetchingUserTurns)
    } 

    let res: Vec<UserTurnMapping> = user_turn_model.unwrap().try_collect().await.unwrap();
    if res.len() == 0 {
        return Err(Error::NoMappingFound)
    }

    let body = Json(json!({
        "result": {
            "success": true
        },
        "user_turns": res.get(0).unwrap()
    }));

    Ok(body)
}


pub async fn stake_in_game(
    state: State<AppDBState>,
    payload: Json<GetGameCurrentStatePayload>
) -> APIResult<Json<Value>> {
    todo!()
}


pub fn get_key_from_redis(redis_client: Arc<Mutex<Connection>>, key: String) -> RedisResult<String> {
    let mut rd_conn = redis_client.lock().unwrap();
    rd_conn.get(key)
}

pub fn get_key_from_redis_int(redis_client: Arc<Mutex<Connection>>, key: String) -> RedisResult<i64> {
    let mut rd_conn = redis_client.lock().unwrap();
    rd_conn.get(key)
}

pub fn set_key_from_redis(redis_client: &Arc<Mutex<Connection>>, key: String , value: String) -> RedisResult<()> {
    let mut rd_conn = redis_client.lock().unwrap();
    rd_conn.set(key , value)
}

pub fn set_key_from_redis_int(redis_client: &Arc<Mutex<Connection>>, key: String , value: i64) -> RedisResult<()> {
    let mut rd_conn = redis_client.lock().unwrap();
    rd_conn.set(key , value)
}

pub fn incr_redis_key(redis_client: &Arc<Mutex<Connection>> , key: String) {
    let mut rd_conn = redis_client.lock().unwrap();
    let _: RedisResult<i64> =rd_conn.incr(key, 1);
}


pub fn delete_key_from_redis(redis_client: &Arc<Mutex<Connection>>, key: String) -> RedisResult<()> {
    let mut rd_conn = redis_client.lock().unwrap();
    rd_conn.del(key)
}
