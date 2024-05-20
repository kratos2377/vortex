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

