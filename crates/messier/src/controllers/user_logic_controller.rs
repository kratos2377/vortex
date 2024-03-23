use std::{str::FromStr, sync::MutexGuard};

use crate::errors::Error;
use crate::errors;
use axum_macros::debug_handler;
use redis::{Commands, Connection, RedisError, RedisResult};
use ton::models::{self, users, users_wallet_keys};
use crate::state::AppDBState;
use models::{users_friends_requests::{self , Entity as UsersFriendsRequests}, users_friends::{self, Entity as UsersFriends}, users::{Entity as Users}};
use axum::extract::State;
use axum::Json;
use errors::Result as APIResult;
use sea_orm::{ActiveModelTrait, TryIntoModel};
use sea_orm::EntityTrait;
use sea_orm::Set;
use serde::{Deserialize, Serialize};
use serde_json::json;
use serde_json::Value;
use uuid::Uuid;

use super::payloads::{AcceptOrRejectRequestPayload, AddWalletAddressPayload, DeleteWalletAddressPayload, GetFriendsRequestPayload, GetOnlineFriendsPayload, GetOnlineFriendsResponseModel, GetUserWalletPayload, SendRequestPayload};



pub async fn send_request(
    state: State<AppDBState>,
	payload: Json<SendRequestPayload>,
) -> APIResult<Json<Value>> {
    if &payload.user_recieved_id == "" || &payload.user_sent_id == "" {
        return Err(Error::MissingParamsError);
    }

    let new_request_id = Uuid::new_v4();
    let new_friend_request_relation = users_friends_requests::ActiveModel {
        id: Set(new_request_id),
        user_recieved_id: Set(Uuid::from_str(&payload.user_recieved_id).unwrap()),
        user_sent_id: Set(Uuid::from_str(&payload.user_sent_id).unwrap()),
    };

    let _result = new_friend_request_relation.save(&state.conn).await.unwrap();

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

    let friend_request_relation  = UsersFriendsRequests::find_by_id(&payload.friend_request_relation_id).one(&state.conn).await.unwrap().expect("Some Error Occured");

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

    let current_user_relation = UsersFriends::find_by_user_and_friend_id(&friend_request_relation.user_sent_id.to_string() , &friend_request_relation.user_recieved_id.to_string()).one(&state.conn).await.unwrap();

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
            userid: Set(friend_request_relation.user_sent_id),
            friendid: Set(friend_request_relation.user_recieved_id),
    },
    users_friends::ActiveModel {
        id: Set(new_relation_id_v2),
        friendid: Set(friend_request_relation.user_sent_id),
        userid: Set(friend_request_relation.user_recieved_id),
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
    let user_friends_ = users_friends::Entity::find_by_user_id(&payload.user_id).all(&state.conn).await.unwrap();



    let body = Json(json!({
		"result": {
			"success": true
		},
        "friends": user_friends_,
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
        userid: Set(payload.user_id),
        walletaddress: Set(payload.wallet_address),
        wallettype: Set(payload.wallet_name)
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


#[debug_handler]
pub async fn get_online_friends(
    State(state): State<AppDBState>,
	Json(payload): Json<GetOnlineFriendsPayload>,
) -> APIResult<Json<Value>> {
    if &payload.user_id == ""  {
        return Err(Error::MissingParamsError)
    }


     let mut results_resp: Vec<GetOnlineFriendsResponseModel>  = vec![];
     let result = users_friends::Entity::find_by_user_id(&payload.user_id).all(&state.conn).await.unwrap();



     let arc_redis_client = state.context.get_redis_db_client();
    
        // Only add keys if the user_id key is present in redis cluster
        // But we might have to use async redis because not able to pass data safely in threads in single thread
        
            for mo in result.iter() {
  let user_friend: users_friends::Model = mo.clone().try_into_model().unwrap();
                
    
        
            let friend_result = match Users::find_by_id(&user_friend.friendid.to_string())
                .one(&state.conn)
                .await
            {
                Ok(data) => data,
                Err(err) => continue, // Skip to the next iteration on error
            };

            let user_type_details = friend_result.unwrap().try_into_model().unwrap();

            let mut redisConnection  = arc_redis_client.lock().unwrap();
            let does_friend_key_exist_in_redis: RedisResult<()> = redisConnection.hkeys(user_type_details.id.to_string().clone());

            if does_friend_key_exist_in_redis.is_err() {
                continue;
            }

            

            let online_friend_response = GetOnlineFriendsResponseModel {
                user_id: user_type_details.id.to_string(),
                username: user_type_details.username,
                first_name: user_type_details.first_name,
                last_name: user_type_details.last_name,
            };

            results_resp.push(online_friend_response);
     
        
            }
   

    let body = Json(json!({
		"result": {
			"success": true,
            "users": serde_json::to_string(&results_resp).unwrap()
		}
	}));


     Ok(body)
}

