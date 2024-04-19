use crate::constants::environment_variables::SMTP_HOST;
use crate::controllers::payloads::ResponseUser;
use crate::errors::Error;
use crate::errors;
use crate::state::AppDBState;
use crate::utils::api_error::APIError;
use crate::utils::jwt::{decode_jwt, encode_jwt};
use axum::extract::State;
use axum::http::StatusCode;
use axum::Json;
use argon2::{self, Config};
use chrono::Utc;
use lazy_regex::Regex;
use ton::models::users::{self , Entity as Users};
use errors::Result;
use sea_orm::ActiveModelTrait;
use lettre::transport::smtp::authentication::Credentials;
use lettre::{Message, SmtpTransport, Transport};
use sea_orm::Set;
use sea_orm::TryIntoModel;
use serde_json::json;
use serde_json::Value;
use tower_cookies::Cookies;
use uuid::Uuid;

use super::payloads::{LoginPayload, RegistrationPayload, SendEmailPayload, VerifyTokenPayload, VerifyUserPayload};


pub async fn verify_token(
    state: State<AppDBState>,
	payload: Json<VerifyTokenPayload>,
) -> Result<Json<Value>> {


    if payload.token == "" {
        return Err(Error::MissingParamsError)
     }

     let res = decode_jwt(payload.token.clone());

     if res.is_err() {
        return Err(Error::InvalidUserToken)
     }
 
     let token_data = res.unwrap();

     let user_data = Users::find_by_id(&token_data.claims.user_id).one(&state.conn).await.unwrap();

     if user_data.is_none() {
        return Err(Error::NoUserEntityFoundForToken)
     }

     let user_model = user_data.unwrap();
 
 
     let body = Json(json!({
         "result": {
             "success": true
         },
 
         "user_data":  user_model
     }));
 
     Ok(body)

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

    // let mut cookie = Cookie::new(middlewares::AUTH_TOKEN, user_found.id.to_string() + ".exp.sign");
	// cookie.set_http_only(true);
	// cookie.set_path("/");
	// cookies.add(cookie);

    let generated_jwt_token = encode_jwt(user_found.id.to_string())
    .map_err(|_| APIError { message: "Failed while Creating JWT token".to_owned(), status_code: StatusCode::UNAUTHORIZED, error_code: Some(41) }).unwrap();


    let body = Json(json!({
		"result": {
			"success": true
		},

        "token":  generated_jwt_token
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

     if payload.username.contains("@") {
        return  Err(Error::UsernameContainsInvalidCharachter)
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

     let _result = new_user.insert(&state.conn).await.unwrap();
     let recieved_user = _result.try_into_model().unwrap();

     let generated_jwt_token = encode_jwt(recieved_user.id.to_string())
     .map_err(|_| APIError { message: "Failed while Creating JWT token".to_owned(), status_code: StatusCode::UNAUTHORIZED, error_code: Some(41) }).unwrap();

     let body = Json(json!({
		"result": {
			"success": true
		},
        "user": ResponseUser {
            id: recieved_user.id,
            first_name: recieved_user.first_name,
            last_name: recieved_user.last_name,
            username: recieved_user.username,
            score: recieved_user.score,
            verified: recieved_user.verified,
            email: recieved_user.email
        },
        "token": generated_jwt_token

	}));


     Ok(body)
}

pub async fn send_email(
    state: State<AppDBState>,
	Json(payload): Json<SendEmailPayload>,
) -> Result<Json<Value>> {
    let email: Message = Message::builder()
    .from(state.from_email.parse().unwrap())
    .to(payload.to_email.parse().unwrap())
    .subject("Your subject")
    .body("Your body".to_string())
    .unwrap();

let creds: Credentials = Credentials::new(state.from_email.to_string(), state.smtp_key.to_string());

// Open a remote connection to gmail
let mailer: SmtpTransport = SmtpTransport::relay(SMTP_HOST)
    .unwrap()
    .credentials(creds)
    .build();

// Send the email
match mailer.send(&email) {
    Ok(_) => {
        let body = Json(json!({
            "result": {
                "success": true
            },
    
        }));

        Ok(body)
    
    },
    Err(e) => Err(Error::SendEmailError),
}
}

pub async fn verify_user(
    state: State<AppDBState>,
	Json(payload): Json<VerifyUserPayload>,
) -> Result<Json<Value>> {
    let user = Users::find_by_id(&payload.id).one(&state.conn).await.unwrap();
     let mut user_recieved = if let Some(user) = user {
        user
     } else {
        return Err(Error::EntityNotFound)
    };

    user_recieved.verified = true;

    let user_active_model = users::ActiveModel {
        id: Set(user_recieved.id),
        username: Set(user_recieved.username),
        first_name: Set(user_recieved.first_name),
        last_name: Set(user_recieved.last_name),
        password: Set(user_recieved.password),
        created_at: Set(user_recieved.created_at),
        updated_at: Set(user_recieved.updated_at),
        score: Set(user_recieved.score),
        email: Set(user_recieved.email),
        verified: Set(user_recieved.verified),
    };

    user_active_model.save(&state.conn).await.unwrap();
    let body = Json(json!({
		"result": {
			"success": true
		},
   	}));

    Ok(body)
}


fn verify_password(hashed_password: String , entered_password: String) -> bool {
    return argon2::verify_encoded(&hashed_password, entered_password.as_bytes()).unwrap()
}

fn hash_password(password: &String) -> String {
    let config = Config::default();
    let hash = argon2::hash_encoded(password.as_bytes(), b"secretsalt", &config).unwrap();
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