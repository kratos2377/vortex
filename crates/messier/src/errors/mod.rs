use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use migration::cli::Cli;
use opentelemetry::trace::Status;
use serde::Serialize;

pub type Result<T> = core::result::Result<T, Error>;

#[derive(Clone, Debug, Serialize, strum_macros::AsRefStr)]
#[serde(tag = "type", content = "data")]
pub enum Error {
	LoginFail,
	RegistrationFail,
	MissingParamsError,
	InvalidUserToken,
	EntityNotFound,
	SamePasswordAsPreviousOne,
	NoUserEntityFoundForToken,
	PasswordIncorrect,
	UsernameAlreadyExists,
	UsernameContainsInvalidCharachter,
	EmailAlreadyInUse,
	PasswordLength,
	NewPasswordLengthIsSmall,
	RegistrationPayloadValidationError,
	UserNameChangeError,
	SendEmailError,
	CreateLobbyError,
	PasswordChangeError,
	JoinLobbyError,
	RemoveFromLobbyError,
	DeleteLobbyError,
	GameCannotBeStarted,
	RedisUnwrapError,
	LobbyFull,
	GameInviteSendError,
	WalletAddressSaveError,
	ErrorWhileMakingRelation,
	SpectateGameJoinError,
	SpectateGameLeaveError,
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
			Self::PasswordChangeError => (StatusCode::BAD_REQUEST, ClientError::PASSWORD_CHANGE_ERROR),

			//Invalid Token
			Self::InvalidUserToken => (StatusCode::BAD_REQUEST, ClientError::INVALID_USER_TOKEN),
			Self::NoUserEntityFoundForToken => (StatusCode::BAD_REQUEST, ClientError::NO_USER_ENTITY_FOUND_FOR_TOKEN),

			//Password Incorrect
			Self::PasswordIncorrect => (StatusCode::BAD_REQUEST , ClientError::PASSWORD_INCORRECT),

			//Registration Error
			Self::UsernameAlreadyExists => (StatusCode::BAD_REQUEST , ClientError::USERNAME_ALREADY_EXISTS),
			Self::EmailAlreadyInUse => (StatusCode::BAD_REQUEST, ClientError::EMAIL_IN_USE),
			Self::UsernameContainsInvalidCharachter => (StatusCode::BAD_REQUEST, ClientError::USERNAME_CONTAINS_INVALID_CHARACTER),

			// Validation Error
			Self::PasswordLength => (StatusCode::BAD_REQUEST, ClientError::PASSWORD_LENGTH_SMALL),
			Self::RegistrationPayloadValidationError => (StatusCode::BAD_REQUEST, ClientError::REGISTRATION_PAYLOAD_VALIDATION_ERROR),

			Self::SendEmailError => (StatusCode::BAD_REQUEST, ClientError::SEND_EMAIL_ERROR),

			Self::SamePasswordAsPreviousOne => (StatusCode::BAD_REQUEST , ClientError::SAME_PASSWORD_AS_PREVIOUS_ONE),

			Self::NewPasswordLengthIsSmall => (StatusCode::BAD_REQUEST , ClientError::NEW_PASSWORD_LENGHT_IS_SMALL),

			//Create Lobby Error
			Self::CreateLobbyError => (StatusCode::BAD_REQUEST, ClientError::CREATE_LOBBY_ERROR),
			Self::JoinLobbyError => (StatusCode::BAD_REQUEST, ClientError::JOIN_LOBBY_ERROR),
			Self::RemoveFromLobbyError => (StatusCode::BAD_REQUEST, ClientError::REMOVE_USER_FROM_LOBBY_ERROR),

			Self::DeleteLobbyError => (StatusCode::BAD_REQUEST, ClientError::DELETE_LOBBY_ERROR),
			Self::UserNameChangeError => (StatusCode::BAD_REQUEST, ClientError::USERNAME_CHANGE_ERROR),


			Self::GameCannotBeStarted => (StatusCode::BAD_REQUEST, ClientError::GAME_CANNOT_BE_STARTED),

			Self::RedisUnwrapError => (StatusCode::BAD_REQUEST, ClientError::REDIS_UNWRAP_ERROR),

			Self::LobbyFull => (StatusCode::BAD_REQUEST, ClientError::LOBBY_FULL_ERROR),

			Self::ErrorWhileMakingRelation => (StatusCode::BAD_REQUEST, ClientError::ERROR_WHILE_MAKING_RELATION),

			//Address Error
			Self::WalletAddressSaveError => (StatusCode::BAD_REQUEST, ClientError::WALLET_ADDRESS_SAVE_ERROR),

			// Spectate Game join error
			Self::SpectateGameJoinError => (StatusCode::BAD_REQUEST , ClientError::SPECTATE_GAME_JOIN_ERROR),
			Self::SpectateGameLeaveError => (StatusCode::BAD_REQUEST , ClientError::SPECTATE_GAME_LEAVE_ERROR),

			//Game Invite Error
			Self::GameInviteSendError => (StatusCode::BAD_REQUEST, ClientError::GAME_INVITE_SEND_ERROR),

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
	SEND_EMAIL_ERROR,
	CREATE_LOBBY_ERROR,
	JOIN_LOBBY_ERROR,
	REMOVE_USER_FROM_LOBBY_ERROR,
	USERNAME_CHANGE_ERROR,
	INVALID_USER_TOKEN,
	DELETE_LOBBY_ERROR,
	GAME_CANNOT_BE_STARTED,
	PASSWORD_CHANGE_ERROR,
	NO_USER_ENTITY_FOUND_FOR_TOKEN,
	REDIS_UNWRAP_ERROR,
	LOBBY_FULL_ERROR,
	USERNAME_CONTAINS_INVALID_CHARACTER,
	SAME_PASSWORD_AS_PREVIOUS_ONE,
	GAME_INVITE_SEND_ERROR,
	ERROR_WHILE_MAKING_RELATION,
	WALLET_ADDRESS_SAVE_ERROR,
	SPECTATE_GAME_JOIN_ERROR,
	SPECTATE_GAME_LEAVE_ERROR,
	NEW_PASSWORD_LENGHT_IS_SMALL,
	NO_AUTH,
	INVALID_PARAMS,
	SERVICE_ERROR,
}
