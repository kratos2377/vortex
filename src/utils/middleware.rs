
use axum::{extract::Request, http::StatusCode, middleware::Next, response::Response};

use super::{api_error::APIError, jwt::decode_jwt};





pub async fn guard(mut req: Request, next: Next) -> Result<Response,APIError> {

    let token = req.headers().get("Authorization")
    .ok_or(APIError { message: "No Auth token found".to_owned(), status_code: StatusCode::BAD_REQUEST, error_code: Some(40)  })?.to_owned();
    let token_value: Vec<&str> = token.to_str().unwrap().split_whitespace().collect();
    let claim = decode_jwt(token_value[1].to_string())
    .map_err(|err| APIError { message: "Unauthorized".to_owned(), status_code: StatusCode::UNAUTHORIZED, error_code: Some(41)  })?.claims;

    Ok(next.run(req).await)
} 