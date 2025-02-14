

pub const USER_ONLINE_EVENT: &str = "user-online-event";
pub const FRIEND_REQUEST_EVENT: &str = "friend-request-event";
pub const GAME_INVITE_EVENT: &str = "game-invite-event";
pub const USER_JOINED_ROOM: &str = "user-joined-room";
pub const USER_LEFT_ROOM: &str = "user-left-room";

pub const USER_STATUS_EVENT: &str = "user-status-event";
pub const VERIFYING_GAME_STATUS: &str = "verifying-game-status";
pub const ERROR_EVENT: &str = "error-event";

//Game Events
pub const USER_GAME_MOVE: &str = "user-game-move";
pub const GAME_GENERAL_EVENT: &str = "game-general-event";
//Redis Key
pub const REDIS_USER_GAME_KEY: &str = "-user-game-id";
pub const REDIS_USER_PLAYER_KEY: &str = "-user-player-type";


//Mongo Collections
pub const MONGO_DB_NAME: &str = "user_game_events_db";
pub const MONGO_USERS_MODEL: &str = "users";
pub const MONGO_GAMES_MODEL: &str = "games";
pub const MONGO_USER_TURNS_MODEL: &str = "user_turns";


//Game Bet Related Kafka Topics
pub const START_GAME_SETTLE_EVENT: &str = "start_game_settle_game_event";
pub const GAME_OVER_EVENT: &str = "game_over_event";
pub const EXECUTOR_GAME_OVER_EVENT: &str = "executor_game_over_event";
pub const CREATE_USER_BET: &str = "create_user_bet";
pub const USER_SCORE_UPDATE: &str = "user_score_update";
pub const USER_GAME_DELETION: &str = "user_game_deletion";
pub const USER_GAME_EVENTS: &str = "user_game_events";
pub const GENERATE_GAME_BET_EVENTS: &str = "generate_game_bet_events";
pub const GAME_SESSION_COMPLETED: &str = "game_session_completed";
pub const GAME_BET_SETTLED: &str = "game_bet_settled";
pub const GAME_BET_SETTLED_ERROR: &str = "game_bet_settled_error";


//Redis Keys
pub const SETTLE_BET_KEY: &str = "GameSettle_";
pub const CHESS_STATE_REDIS_KEY: &str = "ChessState_";