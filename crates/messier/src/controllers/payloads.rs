use serde::{Deserialize, Serialize};
use uuid::Uuid;


// Game Payloads
#[derive(Clone, Debug, Deserialize)]
pub struct CreateLobbyPayload {
    pub user_id: String,
    pub username: String,
    pub game_type: String,
    pub game_name: String,
}


#[derive(Clone, Debug, Deserialize)]
pub struct JoinLobbyPayload {
    pub user_id: String,
    pub username: String,
    pub game_id: String,
    pub game_name: String,
}

#[derive(Clone, Debug, Deserialize)]
pub struct LeaveLobbyPayload {
    pub user_id: String,
    pub game_id: String,
    pub game_name: String,
}




#[derive(Clone, Debug, Deserialize)]
pub struct UpdatePlayerStatusPayload {
    pub game_id: String,
    pub user_id: String,
    pub game_name: String,
    pub status: String,
}

#[derive(Clone, Debug, Deserialize)]
pub struct VerifyGameStatusPayload {
    pub game_id: String,
    pub host_user_id: String,
    pub game_name: String,
    pub user_id: String,
}

#[derive(Clone, Debug, Deserialize)]
pub struct StartGamePayload {
    pub game_id: String,
    pub game_name: String,
}


#[derive(Clone, Debug, Deserialize)]
pub struct RemoveGameModelsPayload {
    pub game_id: String,
    pub host_user_id: String,
    pub game_name: String,
    pub user_id: String,
}


#[derive(Clone, Debug, Deserialize)]
pub struct SendGameEventAPIPayload {
    pub game_id: String,
    pub game_name: String,
    pub user_sending_username: String,
    pub user_sending_id: String,
    pub user_receiving_id: String,
    pub game_type: String,
}



#[derive(Clone, Debug, Deserialize)]
pub struct DestroyLobbyPayload {
    pub game_id: String,
}

#[derive(Clone, Debug, Deserialize)]
pub struct GetLobbyPlayersPayload {
    pub game_id: String,
    pub host_user_id: String,
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

#[derive(Clone, Debug, Deserialize)]
pub struct VerifyTokenPayload {
	pub token: String,
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
   pub id: String,
   pub user_key: String,
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
   pub score: i32,
   pub verified: bool,
}



// User logic payload
#[derive( Clone, Debug, Deserialize)]
pub struct SendRequestPayload {
   pub user_id: String,
 pub friend_username: String,
 pub user_username: String,
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
   pub is_user_online: bool,
   pub username: String,
   pub first_name: String,
   pub last_name: String,
}


#[derive( Clone, Debug, Deserialize , Serialize)]
pub struct GetUsersOngoingGamesPayload {
    pub user_id: String,
}



#[derive( Clone, Debug, Deserialize , Serialize)]
pub struct GetUsersOngoingGamesResponseModel {
   pub game_id: Uuid,
   pub game_type: String,
   pub is_staked: bool,
   pub total_money_staked: f64,
   pub is_match: bool,
   pub usernames_playing: Vec<String>
}

#[derive( Clone, Debug, Deserialize , Serialize)]
pub struct GetGameCurrentStatePayload {
   pub game_id: Uuid,
}

#[derive(Clone, Debug, Deserialize , Serialize)]
pub struct GetUserTurnMappingsPayload {
    pub game_id: String,
}

#[derive(Clone, Debug, Deserialize , Serialize)]
pub struct GetGameDetailsPayload {
    pub game_id: String,
}


#[derive( Clone, Debug, Deserialize , Serialize)]
pub struct ChangeUserPasswordPayload {
   pub user_id: String,
   pub password: String,
   pub new_password: String,
}


#[derive( Clone, Debug, Deserialize , Serialize)]
pub struct ChangeUserUsernamePayload {
   pub user_id: String,
   pub username: String
}


