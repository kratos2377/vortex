use crate::errors::Error;
use crate::models;
use crate::errors;
use crate::middlewares;
use crate::state::AppDBState;
use axum::extract::State;
use axum::Json;
use argon2::{self, Config};
use models::users::{self , Entity as Users};
use errors::Result;
use serde::Deserialize;
use serde_json::json;
use serde_json::Value;
use tower_cookies::Cookie;
use tower_cookies::Cookies;

#[derive(Clone, Debug, Deserialize)]
pub struct LoginPayload {
	usernameoremail: String,
	pwd: String,
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

pub async fn register_user() -> Result<Json<Value>> {
    todo!()
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

fn hash_password(password: String) -> String {
    let config = Config::default();
    let hash = argon2::hash_encoded(password.as_bytes(), b"salt", &config).unwrap();
    hash
}

fn validate_user_payload() -> bool {
    todo!()
}