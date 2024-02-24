use std::str::FromStr;
use std::thread::panicking;

use crate::errors::Error;
use crate::errors;
use ton::models::users_wallet_keys;
use crate::state::AppDBState;
use axum_macros::debug_handler;
use models::{users_friends_requests::{self , Entity as UsersFriendsRequests}, users_friends::{self, Entity as UsersFriends}};
use axum::extract::State;
use axum::Json;
use errors::Result;
use sea_orm::ActiveModelTrait;
use sea_orm::EntityTrait;
use sea_orm::Set;
use serde::Deserialize;
use serde_json::json;
use serde_json::Value;
use uuid::Uuid;

#[derive( Clone, Debug, Deserialize)]
pub struct SendRequestPayload {
 pub user_sent_id: String,
 pub user_recieved_id: String,
}

#[derive( Clone, Debug, Deserialize)]
pub struct AcceptOrRejectRequestPayload {
    value: String,
    friend_request_relation_id: String,
}

#[derive( Clone, Debug, Deserialize)]
pub struct GetFriendsRequestPayload {
    user_id: String,
}


#[derive( Clone, Debug, Deserialize)]
pub struct AddWalletAddressPayload {
    user_id: String,
    wallet_address: String,
    wallet_name: String,
}


#[derive( Clone, Debug, Deserialize)]
pub struct DeleteWalletAddressPayload {
    id: String,
    user_id: String,
}

#[derive( Clone, Debug, Deserialize)]
pub struct GetUserWalletPayload {
    user_id: String,
}

pub async fn send_request(
    state: State<AppDBState>,
	payload: Json<SendRequestPayload>,
) -> Result<Json<Value>> {
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
) -> Result<Json<Value>> {
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
) -> Result<Json<Value>>{
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
) -> Result<Json<Value>> {
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
) -> Result<Json<Value>> {
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
) -> Result<Json<Value>>{
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

pub async fn get_online_friends() {
    todo!()
}