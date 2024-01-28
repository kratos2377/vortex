use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use serde::Serialize;

pub type Result<T> = core::result::Result<T, Error>;

#[derive(Clone, Debug, Serialize, strum_macros::AsRefStr)]
#[serde(tag = "type", content = "data")]
pub enum Error {
	LoginFail,
	RegistrationFail,
	MissingParamsError,
	EntityNotFound,
	PasswordIncorrect,
	UsernameAlreadyExists,
	EmailAlreadyInUse,
	PasswordLength,
	RegistrationPayloadValidationError,
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
		println!("->> {:<12} - {self:?}", "INTO_RES");

		let mut response = StatusCode::INTERNAL_SERVER_ERROR.into_response();

		response.extensions_mut().insert(self);

		response
	}
}

impl Error {
	pub fn client_status_and_error(&self) -> (StatusCode, ClientError) {
		#[allow(unreachable_patterns)]
		match self {
			Self::LoginFail => (StatusCode::FORBIDDEN, ClientError::LOGIN_FAIL),
			Self::RegistrationFail => (StatusCode::FORBIDDEN, ClientError::REGISTRATION_FAIL),

			// Missing Params Error
			Self::MissingParamsError => (StatusCode::BAD_REQUEST, ClientError::MISSING_PARAMS_ERROR),

			//Entity Not Found
			Self::EntityNotFound => (StatusCode::BAD_REQUEST, ClientError::ENTITY_NOT_FOUND),

			//Password Incorrect
			Self::PasswordIncorrect => (StatusCode::BAD_REQUEST , ClientError::PASSWORD_INCORRECT),

			//Registration Error
			Self::UsernameAlreadyExists => (StatusCode::BAD_REQUEST , ClientError::USERNAME_ALREADY_EXISTS),
			Self::EmailAlreadyInUse => (StatusCode::BAD_REQUEST, ClientError::EMAIL_IN_USE),

			// Validation Error
			Self::PasswordLength => (StatusCode::BAD_REQUEST, ClientError::PASSWORD_LENGTH_SMALL),
			Self::RegistrationPayloadValidationError => (StatusCode::BAD_REQUEST, ClientError::REGISTRATION_PAYLOAD_VALIDATION_ERROR),


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
	REGISTRATION_FAIL,
	MISSING_PARAMS_ERROR,
	ENTITY_NOT_FOUND,
	PASSWORD_INCORRECT,
	USERNAME_ALREADY_EXISTS,
	EMAIL_IN_USE,
	PASSWORD_LENGTH_SMALL,
	REGISTRATION_PAYLOAD_VALIDATION_ERROR,
	NO_AUTH,
	INVALID_PARAMS,
	SERVICE_ERROR,
}
