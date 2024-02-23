use axum::http::StatusCode;
use chrono::{Utc, Duration};
use jsonwebtoken::{encode, Header, EncodingKey, TokenData, decode, DecodingKey, Validation};
use serde::{Deserialize, Serialize};

use crate::constants; 


#[derive(Serialize,Deserialize)]
pub struct Claims{
    pub exp: usize,
    pub iat: usize,
    pub user_id: String
}


pub fn encode_jwt(user_id: String) -> Result<String,StatusCode>{

    let now = Utc::now();
    let expire = Duration::days(7);

    let claim = Claims{ iat: now.timestamp() as usize, exp: (now+expire).timestamp() as usize, user_id: user_id };
    let secret = constants::environment_variables::JWT_SECRET_TOKEN.clone();

    return encode(&Header::default(), &claim, &EncodingKey::from_secret(secret.as_ref()))
    .map_err(|_| { StatusCode::INTERNAL_SERVER_ERROR });

}


pub fn decode_jwt(jwt: String) -> Result<TokenData<Claims>,StatusCode> {
    let secret = constants::environment_variables::JWT_SECRET_TOKEN.clone();
    let res: Result<TokenData<Claims>, StatusCode> = decode(&jwt,&DecodingKey::from_secret(secret.as_ref()),&Validation::default())
    .map_err(|_| { StatusCode::INTERNAL_SERVER_ERROR });
    return res;
}