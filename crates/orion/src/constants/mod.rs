

pub const USER_ONLINE_EVENT: &str = "user-online-event";
pub const FRIEND_REQUEST_EVENT: &str = "friend-request-event";
pub const GAME_INVITE_EVENT: &str = "game-invite-event";
pub const USER_JOINED_ROOM: &str = "user-joined-room";
pub const USER_LEFT_ROOM: &str = "user-left-room";

pub const USER_STATUS_EVENT: &str = "user-status-event";
pub const VERIFYING_GAME_STATUS: &str = "verifying-game-status";
pub const ERROR_EVENT: &str = "error-event";

pub const USER_GAME_MOVE: &str = "user-game-move";

//Redis Key
pub const REDIS_USER_GAME_KEY: &str = "-user-game-id";
pub const REDIS_USER_PLAYER_KEY: &str = "-user-player-type";