use axum::body::Body;
use axum::http::{StatusCode};
use axum::response::{IntoResponse , Response};
use serde_json::Value;
use axum::Json;
use migration::cli::Cli;
use opentelemetry::trace::Status;
use serde::Serialize;
use serde_json::json;

pub type Result<T> = core::result::Result<T, Error>;


pub struct ErrorPayloadResponse {
	pub result: SuccessResponse,
	pub error_message: String,
}

pub struct SuccessResponse {
	pub success: bool
}

#[derive(Clone, Debug, Serialize, strum_macros::AsRefStr)]
#[serde(tag = "type", content = "data")]
pub enum Error {
	LoginFail,

	AuthFailNoAuthTokenCookie,
	AuthFailTokenWrongFormat,
	AuthFailCtxNotInRequestExt,

	TicketDeleteFailIdNotFound { id: u64 },
}

impl core::fmt::Display for Error {
	fn fmt(
		&self,
		fmt: &mut core::fmt::Formatter,
	) -> core::result::Result<(), core::fmt::Error> {
		write!(fmt, "{self:?}")
	}
}

impl std::error::Error for Error {}

impl IntoResponse for Error {
	fn into_response(self) -> Response {
	//	println!("->> {:<12} - {self:?}", "INTO_RES");
		let (status_code , error_message) = self.client_status_and_error();
		let json_body_string = Json(json!({
			"result": {
				"success": false
			},

			"error_message": format!("{:?}", error_message)
	
		}));

	
		let mut response = Response::builder().status(status_code).body(json_body_string.into_response().into_body()).unwrap();
		
		response.extensions_mut().insert(self);

		response
	}
}

impl Error {
	pub fn client_status_and_error(&self) -> (StatusCode, ClientError) {
		#[allow(unreachable_patterns)]
		match self {
			Self::LoginFail => (StatusCode::UNAUTHORIZED, ClientError::LOGIN_FAIL),
		

			// -- Auth.
			Self::AuthFailNoAuthTokenCookie
			| Self::AuthFailTokenWrongFormat
			| Self::AuthFailCtxNotInRequestExt => {
				(StatusCode::FORBIDDEN, ClientError::NO_AUTH)
			}

			// -- Model.
			Self::TicketDeleteFailIdNotFound { .. } => {
				(StatusCode::BAD_REQUEST, ClientError::INVALID_PARAMS)
			}

			// -- Fallback.
			_ => (
				StatusCode::INTERNAL_SERVER_ERROR,
				ClientError::SERVICE_ERROR,
			),
		}
	}
}

#[derive(Debug, strum_macros::AsRefStr)]
#[allow(non_camel_case_types)]
pub enum ClientError {
	LOGIN_FAIL,
	NO_AUTH,
	INVALID_PARAMS,
	SERVICE_ERROR,
	INVALID_EMAIL_USER_KEY
}