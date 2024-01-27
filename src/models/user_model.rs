use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};


#[derive(Debug, Deserialize , Serialize)]
pub struct UserModel {
id: String,
password: String,
username: String,
first_name: String,
last_name: String,
score: i64,
created_at: NaiveDateTime,
updated_at: NaiveDateTime,
}