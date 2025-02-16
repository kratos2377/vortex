use std::{fmt, str::FromStr};

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
    pub wallet_key: String,
    pub event_type: GameBetEvent,
    pub is_player: bool
}

#[derive(Clone , Serialize , Deserialize , Eq , PartialEq)]#[serde(rename_all = "UPPERCASE")]
pub enum GameBetEvent {
    CREATE,
    UPDATE,
}

impl fmt::Display for GameBetEvent {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            GameBetEvent::CREATE => write!(f, "CREATE"),
            GameBetEvent::UPDATE => write!(f, "UPDATE"),
        }
    }
}

impl FromStr for GameBetEvent {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_uppercase().as_str() {
            "CREATE" => Ok(GameBetEvent::CREATE),
            "UPDATE" => Ok(GameBetEvent::UPDATE),
            _ => Err(format!("Invalid GameBetEvent: {}", s)),
        }
    }
}



// Generate Game Bet Events 
#[derive(Clone , Serialize , Deserialize)]
pub struct GenerateGameBetSettleEvents {
    pub game_id: String,
    pub session_id: String,
    pub winner_id: String,
    pub is_game_valid: bool
}


// GameOver Event
#[derive(Clone , Serialize , Deserialize)]
pub struct GameOverEvent {
    pub game_id: String,
    pub session_id: String,
    pub winner_id: String,
    pub is_game_valid: bool
}


// Game User Bet Settled Events / Game User bet Settled Error Events
#[derive(Clone , Serialize , Deserialize)]
pub struct GameUserBetSettleEvent {
    pub game_id: String,
    pub session_id: String,
    pub user_id: String,
    pub winner_id: String,
    pub is_game_valid: bool,
    pub is_error: bool
}


//Redis Payload
#[derive(Clone , Serialize , Deserialize)]
pub struct GameSettleBetErrorRedisPayload {
    pub game_id: String,
    pub session_id: String,
    pub winner_id: String,
    pub is_game_valid: bool
}


#[derive(Clone , Serialize , Deserialize)]
pub struct GameStatusChangeEvent {
    pub game_id: String,
    pub session_id: String,
    pub winner_id: String,
    pub is_game_valid: bool,
    pub is_error: bool
}


#[derive(Clone , Serialize , Deserialize)]
pub struct GameStakeTimeRedisPayload {
    pub game_id: String,
    pub session_id: String,
}

#[derive(Clone , Serialize , Deserialize)]
pub struct GameStakeTimeOverEvent {
    pub game_id: String,
    pub session_id: String,
}


#[derive(Clone , Serialize , Deserialize)]
pub struct GameStakeTimeOverEventResult {
    pub game_id: String,
    pub session_id: String,
    pub is_error: bool
}



#[derive(Clone , Serialize , Deserialize)]
pub struct GameBetSettleKafkaPayload {
    pub game_id: String,
    pub session_id: String,
    pub winner_id: String,
    pub user_id: String,
    pub user_betting_on: String,
    pub record_id: String,
    pub user_wallet_key: String,
    pub is_valid: bool
}


#[derive(Clone , Serialize , Deserialize)]
pub struct ExecutorGameOverEvent {
    pub game_id: String,
    pub session_id: String,
}