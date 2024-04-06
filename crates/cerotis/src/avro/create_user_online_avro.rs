
use serde::Deserialize;
use serde::Serialize;


pub const SCHEMA_NAME_CREATE_USER_ONLINE: &str = "CreateUserOnlineAvroV1";

pub const RAW_SCHEMA_CREATE_USER_ONLINE_V1: &str = include_str!("../../resources/create_user_online_v1.avsc");

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateUserOnlineAvro {
    pub identifier: String,
    pub user_id: String,
    pub socket_id: String,
}
