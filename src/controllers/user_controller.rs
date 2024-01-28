use crate::errors::Error;
use crate::models;
use crate::errors;
use crate::middlewares;
use crate::state::AppDBState;
use axum::extract::State;
use axum::Json;
use argon2::{self, Config};
use chrono::Utc;
use lazy_regex::Regex;
use models::users::{self , Entity as Users};
use errors::Result;
use sea_orm::ActiveModelTrait;
use sea_orm::ActiveValue;
use sea_orm::Set;
use serde::Deserialize;
use serde::Serialize;
use serde_json::json;
use serde_json::Value;
use tower_cookies::Cookie;
use tower_cookies::Cookies;
use uuid::Uuid;

#[derive(Clone, Debug, Deserialize)]
pub struct LoginPayload {
	usernameoremail: String,
	pwd: String,
}


#[derive( Clone, Debug, Deserialize)]
pub struct RegistrationPayload {
    first_name: String,
    last_name: String,
    email: String,
	username: String,
	password: String,
}


#[derive(Clone , Debug, Serialize)]
pub struct ResponseUser {
    id: Uuid,
    username: String,
    first_name: String,
    last_name: String,
    email: String,
    score: u64,
    verified: bool,
}



pub async fn login_user(
    state: State<AppDBState>,
    cookies: Cookies,
	payload: Json<LoginPayload>,
) -> Result<Json<Value>> {
    if payload.usernameoremail == "" || payload.pwd == "" {
       return Err(Error::MissingParamsError)
    }

    let mut user_found;
    if payload.usernameoremail.contains("@") {
      let user = Users::find_by_email(&payload.usernameoremail).one(&state.conn).await.unwrap();

      if let Some(user) = user {
            user_found = user.clone()
      } else {
        return Err(Error::EntityNotFound)
      }
    } else {
        let user = Users::find_by_username(&payload.usernameoremail).one(&state.conn).await.unwrap();

        if let Some(user) = user {
              user_found = user.clone()
        } else {
          return Err(Error::EntityNotFound)
        }
    }

    if !verify_password(user_found.password , payload.pwd.clone()) {
        return Err(Error::PasswordIncorrect)
    }

    let mut cookie = Cookie::new(middlewares::AUTH_TOKEN, user_found.id.to_string() + ".exp.sign");
	cookie.set_http_only(true);
	cookie.set_path("/");
	cookies.add(cookie);

    let body = Json(json!({
		"result": {
			"success": true
		}
	}));

	Ok(body)
                                                                                                                     
}

pub async fn register_user(
    state: State<AppDBState>,
	Json(payload): Json<RegistrationPayload>,
) -> Result<Json<Value>> {
    if payload.first_name == "" || payload.last_name == "" || payload.email == "" || payload.username == "" || payload.password == "" {
        return Err(Error::MissingParamsError)
     }

     let user_by_username = Users::find_by_username(&payload.username).one(&state.conn).await.unwrap();

     if let Some(user_by_username) = user_by_username {
        return Err(Error::UsernameAlreadyExists)
     }

     let user_by_email = Users::find_by_email(&payload.email).one(&state.conn).await.unwrap();

     if let Some(user_by_email) = user_by_email {
        return Err(Error::EmailAlreadyInUse)
     }

     if payload.password.len() < 8 {
        return Err(Error::PasswordLength)
     }

     if !validate_user_payload(&payload) {
        return Err(Error::RegistrationPayloadValidationError)
     }
     let user_id = Uuid::new_v4();
     let hashed_password = hash_password(&payload.password);
     let new_user = users::ActiveModel {
        id: Set(user_id),
        password: Set(hashed_password),
        first_name: Set(payload.first_name),
        last_name: Set(payload.last_name),
        username: Set(payload.username),
        email: Set(payload.email),
        verified: Set(false),
        score: Set(0),
        created_at: Set(Utc::now().naive_utc()),
        updated_at: Set(Utc::now().naive_utc())
     };

     let _result = new_user.save(&state.conn).await.unwrap();
     // figure out a way to serialize Active Model or send data 
     let body = Json(json!({
		"result": {
			"success": true
		},
        // "user": ResponseUser {
        //     id: user_id,
        //     first_name: payload.first_name,
        //     last_name: payload.last_name,
        //     username: payload.username,
        //     score: 0,
        //     verified: false,
        //     email: payload.email
        // }

	}));


     Ok(body)
}

pub async fn send_email() -> Result<Json<Value>> {
    todo!()
}

pub async fn verify_user() -> Result<Json<Value>> {
todo!()
}


fn verify_password(hashed_password: String , entered_password: String) -> bool {
    return argon2::verify_encoded(&hashed_password, entered_password.as_bytes()).unwrap()
}

fn hash_password(password: &String) -> String {
    let config = Config::default();
    let hash = argon2::hash_encoded(password.as_bytes(), b"salt", &config).unwrap();
    hash
}

fn validate_user_payload(payload: &RegistrationPayload) -> bool {
    let username_regex = Regex::new(r"[@\s]").unwrap();
    let email_regex = Regex::new(r"^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}$").unwrap();
    if username_regex.is_match(&payload.username) {
        return false
    }

    if !email_regex.is_match(&payload.email) {
        return false
    }

    return true
}

// pub fn return_value_from_active_value<T>(value: &ActiveValue<T>) -> T {
//     let value_got: T = match value {
//         ActiveValue::Set(value_got_from_db) => value_got_from_db.clone(),
//         ActiveValue::NotSet => T::new(),
//         ActiveValue::Unchanged(_) => todo!(),

//     };

//     return value_got
// }