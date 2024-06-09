use orion::{constants::{FRIEND_REQUEST_EVENT, GAME_GENERAL_EVENT, GAME_INVITE_EVENT, USER_GAME_MOVE, USER_JOINED_ROOM, USER_LEFT_ROOM, USER_ONLINE_EVENT}, events::{kafka_event::{GameGeneralKafkaEvent, UserFriendRequestKafkaEvent, UserGameInviteKafkaEvent}, mqtt_subscribe_events::{MQTT_GAME_EVENTS, MQTT_USER_EVENTS}, ws_events::{JoinedRoomPayload, LeavedRoomPayload}}, models::user_game_event::UserGameEvent};

use super::mqtt_event_model::MQTTEventModel;

pub async fn send_user_online_event_event_mqtt(cli: &mqtt::Client , payload: String) {

    let user_friend_event: UserFriendRequestKafkaEvent = serde_json::from_str(&payload).unwrap();

    let mqtt_payload = serde_json::to_string(&MQTTEventModel{ event_name: USER_ONLINE_EVENT.to_string(), payload }).unwrap();
    let mqtt_user_message = mqtt::message::Message::new(MQTT_USER_EVENTS.to_string() + &user_friend_event.user_who_we_are_sending_event ,  mqtt_payload, 0);
    cli.publish(mqtt_user_message).unwrap();

}

pub async fn send_user_joined_room_event_mqtt(cli: &mqtt::Client , payload: String) {

    let joined_room_payload: JoinedRoomPayload = serde_json::from_str(&payload).unwrap();

    let mqtt_payload = serde_json::to_string(&MQTTEventModel{ event_name: USER_JOINED_ROOM.to_string(), payload }).unwrap();
    let mqtt_user_message = mqtt::message::Message::new(MQTT_GAME_EVENTS.to_string() + &joined_room_payload.game_id ,  mqtt_payload, 1);
    cli.publish(mqtt_user_message).unwrap();

} 


pub async fn send_user_left_room_event_mqtt(cli: &mqtt::Client , payload: String) {

    let leaved_room_payload: LeavedRoomPayload = serde_json::from_str(&payload).unwrap();

    let mqtt_payload = serde_json::to_string(&MQTTEventModel{ event_name: USER_LEFT_ROOM.to_string(), payload }).unwrap();
    let mqtt_user_message = mqtt::message::Message::new(MQTT_GAME_EVENTS.to_string() + &leaved_room_payload.game_id ,  mqtt_payload, 1);
    cli.publish(mqtt_user_message).unwrap();

} 


pub async fn send_user_friend_request_event_mqtt(cli: &mqtt::Client , payload: String) {

    let friend_request_payload: UserFriendRequestKafkaEvent = serde_json::from_str(&payload).unwrap();

    let mqtt_payload = serde_json::to_string(&MQTTEventModel{ event_name: FRIEND_REQUEST_EVENT.to_string(), payload }).unwrap();
    let mqtt_user_message = mqtt::message::Message::new(MQTT_USER_EVENTS.to_string() + &friend_request_payload.user_who_we_are_sending_event ,  mqtt_payload, 0);
      cli.publish(mqtt_user_message).unwrap();

} 


pub async fn send_game_invite_room_event_mqtt(cli: &mqtt::Client , payload: String) {

    let game_invite_payload: UserGameInviteKafkaEvent = serde_json::from_str(&payload).unwrap();

    let mqtt_payload = serde_json::to_string(&MQTTEventModel{ event_name: GAME_INVITE_EVENT.to_string(), payload }).unwrap();
    let mqtt_user_message = mqtt::message::Message::new(MQTT_USER_EVENTS.to_string() + &game_invite_payload.user_who_we_are_sending_event ,  mqtt_payload, 0);
    cli.publish(mqtt_user_message).unwrap();


} 

pub async fn send_game_move_event_mqtt(cli: &mqtt::Client , payload: String) {

    let game_move_payload: UserGameEvent = serde_json::from_str(&payload).unwrap();

    let mqtt_payload = serde_json::to_string(&MQTTEventModel{ event_name: USER_GAME_MOVE.to_string(), payload }).unwrap();
    let mqtt_user_message = mqtt::message::Message::new(MQTT_GAME_EVENTS.to_string() + &game_move_payload.user_game_move.game_id ,  mqtt_payload, 1);
    cli.publish(mqtt_user_message).unwrap();

} 

pub async fn send_game_general_event_mqtt(cli: &mqtt::Client , payload: String) {

    let game_general_event: GameGeneralKafkaEvent = serde_json::from_str(&payload).unwrap();

    let mqtt_payload = serde_json::to_string(&MQTTEventModel{ event_name: GAME_GENERAL_EVENT.to_string(), payload }).unwrap();
    let mqtt_user_message = mqtt::message::Message::new(MQTT_GAME_EVENTS.to_string() + &game_general_event.game_id ,  mqtt_payload, 1);
    cli.publish(mqtt_user_message).unwrap();

} 

