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
	MissingParams,
	ErrorWhileFetchingGameModel,
	NoUserIdFound,
	InvalidParams,
	CannotPlaceNewBetForSameGame,
	CannotBetSinceBetTimeHasElapsed,
	ErrorWhileFetchingUserBets,
	ErrorWhilePlacingBet,
	ErrorWhileFetchingExistingBet,
	ErrorWhileUpdatingGameBet,
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
			Self::MissingParams => (StatusCode::BAD_REQUEST, ClientError::MISSING_PARAMS),
			Self::NoUserIdFound => (StatusCode::BAD_REQUEST , ClientError::NO_USER_ID_FOUND),
			Self::ErrorWhileFetchingGameModel => (StatusCode::BAD_REQUEST, ClientError::ERROR_WHILE_FETCHING_GAME_MODEL),

			Self::ErrorWhileFetchingUserBets => (StatusCode::BAD_REQUEST, ClientError::ERROR_WHILE_FETCHING_USER_BETS),
			Self::InvalidParams => (StatusCode::BAD_REQUEST, ClientError::INVALID_PARAMS),
			Self::CannotBetSinceBetTimeHasElapsed => (StatusCode::BAD_REQUEST, ClientError::CANNOT_BET_SINCE_BET_TIME_HAS_ELAPSED),
			Self::CannotPlaceNewBetForSameGame => (StatusCode::BAD_REQUEST, ClientError::CANNOT_PLACE_NEW_BET_FOR_SAME_GAME),
			Self::ErrorWhilePlacingBet => (StatusCode::BAD_REQUEST, ClientError::ERROR_WHILE_PLACING_BET),
		
		
			Self::ErrorWhileFetchingExistingBet => (StatusCode::BAD_REQUEST, ClientError::ERROR_WHILE_FETCHING_EXISTING_BET),
			Self::ErrorWhileUpdatingGameBet => (StatusCode::BAD_REQUEST, ClientError::ERROR_WHILE_UPDATING_GAME_BET),
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
	NO_USER_ID_FOUND,
	INVALID_PARAMS,
	MISSING_PARAMS,
	SERVICE_ERROR,
	CANNOT_PLACE_NEW_BET_FOR_SAME_GAME,
	CANNOT_BET_SINCE_BET_TIME_HAS_ELAPSED,
	ERROR_WHILE_FETCHING_GAME_MODEL,
	ERROR_WHILE_FETCHING_USER_BETS,
	ERROR_WHILE_PLACING_BET,
	ERROR_WHILE_FETCHING_EXISTING_BET,
	ERROR_WHILE_UPDATING_GAME_BET,
	INVALID_EMAIL_USER_KEY
}