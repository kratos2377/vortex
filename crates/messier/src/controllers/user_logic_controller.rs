use std::collections::HashMap;
use std::{collections::HashSet, str::FromStr};

use crate::event_producer::user_events_producer::send_event_for_user_topic;
use crate::errors::Error;
use crate::errors;
use argon2::{self, Config};
use bson::{doc, Uuid as BsonUuid};
use futures::TryStreamExt;
use orion::constants::{MONGO_DB_NAME, MONGO_GAMES_MODEL, MONGO_USERS_MODEL};
use orion::models::game_model::Game;
use orion::{constants::FRIEND_REQUEST_EVENT, models::user_game_relation_model::UserGameRelation};
use orion::events::kafka_event::UserFriendRequestKafkaEvent;
use ton::models::{self, users, users_wallet_keys};
use crate::state::AppDBState;
use models::{users_friends_requests::{self , Entity as UsersFriendsRequests}, users_friends::{self, Entity as UsersFriends}, users::{Entity as Users}};
use axum::extract::State;
use axum::Json;
use errors::Result as APIResult;
use sea_orm::{ActiveModelTrait, Condition, DbBackend, IntoActiveModel, JoinType, QueryFilter, QuerySelect, RelationTrait, Statement, TryIntoModel};
use sea_orm::EntityTrait;
use sea_orm::Set;
use serde_json::json;
use serde_json::Value;
use uuid::Uuid;
use sea_orm::ColumnTrait;

use super::payloads::{AcceptOrRejectRequestPayload, AddWalletAddressPayload, ChangeUserPasswordPayload, ChangeUserUsernamePayload, DeleteWalletAddressPayload, GetFriendsRequestPayload, GetOnlineFriendsPayload, GetOnlineFriendsResponseModel, GetUserWalletPayload, GetUsersOngoingGamesPayload, GetUsersOngoingGamesResponseModel, SendRequestPayload};

pub async fn send_request(
    state: State<AppDBState>,
	payload: Json<SendRequestPayload>,
) -> APIResult<Json<Value>> {
    if &payload.friend_username == ""  || &payload.user_id == "" || &payload.user_username == "" {
        return Err(Error::MissingParamsError);
    }

    let friend_user_result = Users::find_by_username(&payload.friend_username).one(&state.conn).await;

    if friend_user_result.is_err() {
        return Err(Error::UsernameNotFound)
    }

    let friend_user_option = friend_user_result.unwrap();

    if friend_user_option.is_none() {
        return Err(Error::UsernameNotFound)
    }
    let user_found = friend_user_option.unwrap();
    
    let user_friend_relation =  UsersFriendsRequests::find_by_user_id_and_received_id(&Uuid::from_str(&payload.user_id).unwrap(), &user_found.id.clone()).one(&state.conn).await;

    if user_friend_relation.is_err() {
        return Err(Error::ErrorWhileSendingRequest)
    }

    if user_friend_relation.unwrap().is_some() {
        return Err(Error::FriendRequestAlreadySent)
    }
    let new_request_id = Uuid::new_v4();
    let new_friend_request_relation = users_friends_requests::ActiveModel {
        id: Set(new_request_id),
        user_recieved_id: Set(user_found.id),
        user_sent_id: Set(Uuid::from_str(&payload.user_id).unwrap()),
        user_sent_username: Set(payload.user_username.clone()),
    };

    let kafka_event = UserFriendRequestKafkaEvent {
        friend_request_id: new_request_id.clone(),
        user_who_send_request_id: payload.user_id.clone(),
        user_who_send_request_username: payload.user_username.clone(),
        user_who_we_are_sending_event: user_found.id.to_string().clone(),
    };
    let friend_request_kafka_event = serde_json::to_string(&kafka_event).unwrap();
    let _ = send_event_for_user_topic(&state.producer,&state.context , FRIEND_REQUEST_EVENT.to_string() , friend_request_kafka_event ).await.unwrap();

    let _result = new_friend_request_relation.insert(&state.conn).await.unwrap();

    let body = Json(json!({
		"result": {
			"success": true
		}
	}));


     Ok(body)
}

pub async fn accept_or_reject_request(
    state: State<AppDBState>,
	Json(payload): Json<AcceptOrRejectRequestPayload>,
) -> APIResult<Json<Value>> {
    if payload.value == "" || payload.friend_request_relation_id == "" {
        return Err(Error::MissingParamsError)
    }

    let friend_request_relation  = UsersFriendsRequests::find_by_id(&Uuid::from_str(&payload.friend_request_relation_id).unwrap()).one(&state.conn).await.unwrap().expect("Some Error Occured");

    if payload.value == "-1" {
        let _delete_relation = users_friends_requests::Entity::delete_by_id(Uuid::from_str(&payload.friend_request_relation_id).unwrap()).exec(&state.conn).await;

        if _delete_relation.is_ok() {
            let body = Json(json!({
                "result": {
                    "success": true
                }
            }));
        
        
             return Ok(body);  
        }

        return Err(Error::ErrorWhileMakingRelation);
    }

    let current_user_relation = UsersFriends::find_by_user_and_friend_id(&friend_request_relation.user_sent_id , &friend_request_relation.user_recieved_id).one(&state.conn).await.unwrap();

    if current_user_relation.is_some() {
        let body = Json(json!({
            "result": {
                "success": true
            }
        }));
    
    
         return Ok(body);
    }

    let new_relation_id_v1 = Uuid::new_v4();
    let new_relation_id_v2 = Uuid::new_v4();
    let new_friend_request_relations = vec![
        users_friends::ActiveModel {
            id: Set(new_relation_id_v1),
            user_id: Set(friend_request_relation.user_sent_id),
            friend_id: Set(friend_request_relation.user_recieved_id),
    },
    users_friends::ActiveModel {
        id: Set(new_relation_id_v2),
        friend_id: Set(friend_request_relation.user_sent_id),
        user_id: Set(friend_request_relation.user_recieved_id),
},
    ];
    let _result = users_friends::Entity::insert_many(new_friend_request_relations).exec(&state.conn).await;
    let _delete_relation = users_friends_requests::Entity::delete_by_id(Uuid::from_str(&payload.friend_request_relation_id).unwrap()).exec(&state.conn).await;
    if _result.is_err() || _delete_relation.is_err() {
        return Err(Error::ErrorWhileMakingRelation)
    }

    let body = Json(json!({
		"result": {
			"success": true
		}
	}));


     Ok(body)
}


pub async fn get_user_friend_requests(
    state: State<AppDBState>,
	Json(payload): Json<GetFriendsRequestPayload>,
) -> APIResult<Json<Value>>{
    if payload.user_id == ""  {
        return Err(Error::MissingParamsError)
    }

    // Add custom Query to return all users id,username,and any other details
    let user_friends_ = users_friends_requests::Entity::find_by_user_received_id(&Uuid::from_str(&payload.user_id).unwrap()).all(&state.conn).await;

    if user_friends_.is_err() {
        return Err(Error::ErrorWhileFetchingUserFriendsRequests)
    }



    let body = Json(json!({
		"result": {
			"success": true
		},
        "friends": user_friends_.unwrap(),
	}));


     Ok(body)
}

pub async fn add_wallet_address(
    state: State<AppDBState>,
	Json(payload): Json<AddWalletAddressPayload>,
) -> APIResult<Json<Value>> {
    if &payload.wallet_address == "" || &payload.wallet_name =="" || &payload.user_id == "" {
        return Err(Error::MissingParamsError)
    }

    let new_relation_id = Uuid::new_v4();
    let user_wallet = users_wallet_keys::ActiveModel {
        id: Set(new_relation_id),
        user_id: Set(payload.user_id),
        wallet_address: Set(payload.wallet_address),
        wallet_type: Set(payload.wallet_name)
    };

    let _result = user_wallet.save(&state.conn).await;

    if _result.is_err() {
        return Err(Error::WalletAddressSaveError)
    }

    let body = Json(json!({
		"result": {
			"success": true
		}
	}));


     Ok(body)
}


pub async fn get_user_wallets(
    state: State<AppDBState>,
	Json(payload): Json<GetUserWalletPayload>,
) -> APIResult<Json<Value>> {
    if &payload.user_id == ""  {
        return Err(Error::MissingParamsError)
    }


    let _result = users_wallet_keys::Entity::find_by_userid(&payload.user_id).all(&state.conn).await.unwrap();



    let body = Json(json!({
		"result": {
			"success": true
		},
        "wallets": _result
	}));


     Ok(body)
}



pub async fn delete_wallet_address(
    state: State<AppDBState>,
	Json(payload): Json<DeleteWalletAddressPayload>,
) -> APIResult<Json<Value>>{
    if &payload.user_id == "" || &payload.id == "" {
        return Err(Error::MissingParamsError)
    }


    let _result = users_wallet_keys::Entity::delete_by_id(Uuid::from_str(&payload.id).unwrap()).exec(&state.conn).await;

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
    let mongo_db = state.context.get_mongo_db_client().database(MONGO_DB_NAME);

    let user_collection = mongo_db.collection::<UserGameRelation>(MONGO_USERS_MODEL);
    let game_collection = mongo_db.collection::<Game>(MONGO_GAMES_MODEL);
    let mut game_vec_results: Vec<GetUsersOngoingGamesResponseModel> = vec![];
    for user_model in get_user_friends_ids_vec.iter() {

        let cursor = user_collection.find(doc! { "user_id": user_model.clone().friend_id.to_string() }, None).await.unwrap();
      


        let res: Vec<UserGameRelation> = cursor.try_collect().await.unwrap();

        if res.len() == 0 {
            continue;
        }
        let user_game_rel = res.get(0);
        if user_game_rel.is_none() {
            continue;
        }

        let user_game_rel_model = user_game_rel.unwrap();

        println!("Usergame relation vec is");
        println!("{:?}" , user_game_rel_model.game_id);
      
        let game_id = &user_game_rel_model.game_id.clone();
            if !game_sets.contains(game_id) {
                game_sets.insert(game_id.to_string());
                let game_cursor = game_collection.find(doc! { "id": game_id }, None).await.unwrap();
                let game_res_future = game_cursor.try_collect().await;
                if game_res_future.is_err() {
                    continue;
                }

                let game_res: Vec<Game> = game_res_future.unwrap();
                let new_game_res = game_res.get(0).unwrap();
                let new_game = GetUsersOngoingGamesResponseModel {
                    game_id: new_game_res.id,
                    game_type: new_game_res.game_type.clone(),
                    is_staked: new_game_res.is_staked,
                    is_match: new_game_res.is_match,
                    total_money_staked: 0.0,
                    usernames_playing: vec![user_game_rel_model.username.clone()],
                };
                game_id_gamemodel.insert(game_id.clone(),  new_game);
            } else {
                 let game_model = game_id_gamemodel.get_mut(game_id).unwrap();
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



pub async fn get_user_online_friends(
    State(state): State<AppDBState>,
	Json(payload): Json<GetOnlineFriendsPayload>,
) -> APIResult<Json<Value>> {
    if &payload.user_id == ""  {
        return Err(Error::MissingParamsError)
    }


     let mut results_resp: Vec<GetOnlineFriendsResponseModel>  = vec![];
   //  let result = users_friends::Entity::find_user_online_friends(&Uuid::from_str(&payload.user_id).unwrap()).all(&state.conn).await.unwrap();


    let result = users::Entity::find().from_raw_sql(
        Statement::from_sql_and_values(DbBackend::Postgres, 
            
            r#"SELECT "u2"."id", "u2"."first_name", "u2"."last_name", "u2"."email", "u2"."password", "u2"."username",
            "u2"."verified", "u2"."score", "u2"."is_online", "u2"."created_at", "u2"."updated_at" FROM "users" "u1" JOIN "users_friends" "uf" ON "u1"."id" = "uf"."user_id" 
            JOIN "users" "u2" ON "uf"."friend_id" = "u2"."id" WHERE "u1"."id"=$1 AND "u2"."is_online"=$2"#
            , [Uuid::parse_str(&payload.user_id).unwrap().into() , true.into()])
    )
  
    .all(&state.conn).await;
        
    if result.is_err() {
        println!("The error while getting online friends is: {:?}", result.as_ref().err());
        println!("Error Is: {:?}", result.unwrap_err());
        return Err(Error::ErrorWhileFetchingUserFriends)
    }

    
            for mo in result.unwrap().iter() {
             let user_type_details: users::Model = mo.clone().try_into_model().unwrap();

                let online_friend_response=   GetOnlineFriendsResponseModel {
                    user_id: user_type_details.id.to_string(),
                    username: user_type_details.username,
                    first_name: user_type_details.first_name,
                    last_name: user_type_details.last_name,
                    is_user_online: true,
                };
         
            results_resp.push(online_friend_response);
     
        
            }
   

    let body = Json(json!({
		"result": {
			"success": true,
		},
        "friends": results_resp
	}));


     Ok(body)
}


pub async fn change_user_password(
    State(state): State<AppDBState>,
	Json(payload): Json<ChangeUserPasswordPayload>,
) -> APIResult<Json<Value>> {
    if payload.user_id == "" || payload.new_password == "" || payload.password == "" {
        return Err(Error::MissingParamsError)
    }

    if payload.new_password == payload.password {
        return Err(Error::SamePasswordAsPreviousOne)
    }

    if payload.new_password.len() < 8 {
        return Err(Error::NewPasswordLengthIsSmall)
    }


    let user = Users::find_by_id(Uuid::from_str(&payload.user_id).unwrap()).one(&state.conn).await.unwrap();
    let mut user_model: users::ActiveModel = user.unwrap().into();
    let converted_model = user_model.clone().try_into_model().unwrap();

    if !verify_password(converted_model.password, payload.password) {
        return Err(Error::PasswordIncorrect)
    }

    let new_hash_password = hash_password(&payload.new_password);

    user_model.password = Set(new_hash_password.to_owned());

    let res = user_model.update(&state.conn).await;

    if res.is_err() {
        return Err(Error::PasswordChangeError)
    }

    let body = Json(json!({
		"result": {
			"success": true,
		}
	}));


     Ok(body)

}


pub async fn change_user_username(
    State(state): State<AppDBState>,
	Json(payload): Json<ChangeUserUsernamePayload>,
) -> APIResult<Json<Value>> {
    if payload.user_id == "" || payload.username == "" {
        return Err(Error::MissingParamsError)
    }


    let user = Users::find_by_username(&payload.username).one(&state.conn).await.unwrap();

    if !user.is_none() {
        return Err(Error::UsernameAlreadyExists)
    }

    let  user_model = Users::find_by_id(Uuid::from_str(&payload.user_id).unwrap()).one(&state.conn).await;

    if user_model.is_err() {
        return Err(Error::UsernameNotFound)
    }
    let mut new_user_model = user_model.unwrap().unwrap().into_active_model();
    new_user_model.username  = Set(payload.username);


    let res = new_user_model.update(&state.conn).await;

    if res.is_err() {
        return Err(Error::UserNameChangeError)
    }

    let body = Json(json!({
		"result": {
			"success": true,
		}
	}));


     Ok(body)

}

fn verify_password(hashed_password: String , entered_password: String) -> bool {
    return argon2::verify_encoded(&hashed_password, entered_password.as_bytes()).unwrap()
}

fn hash_password(password: &String) -> String {
    let config = Config::default();
    let hash = argon2::hash_encoded(password.as_bytes(), b"secretsalt", &config).unwrap();
    hash
}