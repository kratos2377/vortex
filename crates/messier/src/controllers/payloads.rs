use serde::{Deserialize, Serialize};
use uuid::Uuid;


// Game Payloads
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


#[derive(Deserialize)]
pub struct SpectateGamePayload {
    pub user_id: String,
    pub game_id: String,
    pub socket_id: String,
}


// User Auth Payloads
#[derive(Clone, Debug, Deserialize)]
pub struct LoginPayload {
	pub usernameoremail: String,
	pub pwd: String,
}


#[derive( Clone, Debug, Deserialize)]
pub struct RegistrationPayload {
    pub first_name: String,
    pub last_name: String,
    pub email: String,
	pub username: String,
	pub password: String,
}

#[derive( Clone, Debug, Deserialize)]
pub struct VerifyUserPayload {
   pub id: String
}


#[derive( Clone, Debug, Deserialize)]
pub struct SendEmailPayload {
    pub id: String,
    pub to_email: String,
}


#[derive(Clone , Debug, Serialize)]
pub struct ResponseUser {
   pub id: Uuid,
   pub username: String,
   pub first_name: String,
   pub last_name: String,
   pub email: String,
   pub score: u64,
   pub verified: bool,
}



// User logic payload
#[derive( Clone, Debug, Deserialize)]
pub struct SendRequestPayload {
 pub user_sent_id: String,
 pub user_recieved_id: String,
}

#[derive( Clone, Debug, Deserialize)]
pub struct AcceptOrRejectRequestPayload {
   pub value: String,
   pub friend_request_relation_id: String,
}

#[derive( Clone, Debug, Deserialize)]
pub struct GetFriendsRequestPayload {
   pub user_id: String,
}


#[derive( Clone, Debug, Deserialize)]
pub struct AddWalletAddressPayload {
   pub user_id: String,
   pub wallet_address: String,
   pub wallet_name: String,
}


#[derive( Clone, Debug, Deserialize)]
pub struct DeleteWalletAddressPayload {
   pub id: String,
   pub user_id: String,
}

#[derive( Clone, Debug, Deserialize)]
pub struct GetUserWalletPayload {
   pub user_id: String,
}

#[derive( Clone, Debug, Deserialize , Serialize)]
pub struct GetOnlineFriendsPayload {
   pub user_id: String,
}


#[derive( Clone, Debug, Deserialize , Serialize)]
pub struct GetOnlineFriendsResponseModel {
   pub user_id: String,
   pub username: String,
   pub first_name: String,
   pub last_name: String,
}