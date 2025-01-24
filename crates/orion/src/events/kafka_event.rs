use serde::{Deserialize, Serialize};
use uuid::Uuid;




// Key will act as event
#[derive(Clone , Serialize , Deserialize)]

pub struct KafkaGeneralEvent {
    pub topic: String,
    pub payload: String,
    pub key: String,
}


#[derive(Clone , Serialize , Deserialize)]
pub struct UserOnlineKafkaEvent {
    pub user_who_came_online_id: String,
    pub user_who_came_online_username: String,
    pub user_who_we_are_sending_event: String,
}



#[derive(Clone , Serialize , Deserialize)]
pub struct UserFriendRequestKafkaEvent {
    pub friend_request_id: Uuid,
    pub user_who_send_request_id: String,
    pub user_who_send_request_username: String,
    pub user_who_we_are_sending_event: String,
}

#[derive(Clone , Serialize , Deserialize)]
pub struct UserGameInviteKafkaEvent {
    pub user_who_send_request_id: String,
    pub user_who_send_request_username: String,
    pub user_who_we_are_sending_event: String,
    pub game_id: String,
    pub game_name: String,
    pub game_type: String,
}

#[derive(Clone , Serialize , Deserialize , Debug)]
pub struct UserGameDeletetionEvent {
    pub user_id: String,
    pub game_id: String,
}


// Game Topic Kafka Events

#[derive(Clone , Serialize , Deserialize)]
pub struct GameGeneralKafkaEvent {
    pub message: String,
    pub game_id: String,
}


//User Bet events
#[derive(Clone , Serialize , Deserialize)]
pub struct UserGameBetEvent {
   pub user_id_who_is_betting: String,
    pub user_id: String,
    pub game_id: String,
    pub bet_type: String ,
    pub amount: f32,
    pub session_id: String,
    pub event_type: GameBetEvent
}

#[derive(Clone , Serialize , Deserialize , Eq , PartialEq)]
pub enum GameBetEvent {
    CREATE,
    UPDATE
}



// Generate Game Bet Events 
#[derive(Clone , Serialize , Deserialize)]
pub struct GenerateGameBetSettleEvents {
    pub game_id: String,
    pub session_id: String,
    pub winner_id: String,
}


// GameOver Event
#[derive(Clone , Serialize , Deserialize)]
pub struct GameOverEvent {
    pub game_id: String,
    pub session_id: String,
    pub winner_id: String,
}


// Game User Bet Settled Events / Game User bet Settled Error Events
#[derive(Clone , Serialize , Deserialize)]
pub struct GameUserBetSettleEvent {
    pub game_id: String,
    pub session_id: String,
    pub user_id: String,
    pub winner_id: String,
    pub is_error: bool
}


//Redis Payload
#[derive(Clone , Serialize , Deserialize)]
pub struct GameSettleBetErrorRedisPayload {
    pub game_id: String,
    pub session_id: String,
    pub winner_id: String,
}



#[derive(Clone , Serialize , Deserialize)]
pub struct GameBetSettleKafkaPayload {
    pub game_id: String,
    pub session_id: String,
    pub winner_id: String,
    pub user_id: String,
    pub user_betting_on: String,
    pub record_id: String,
}