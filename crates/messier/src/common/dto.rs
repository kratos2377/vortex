use std::any::Any;

use crate::mongo::{event_models::SerializableEventDto, kafka_event_models::UserGameEvent};

use super::schema_create_user_game_event::{CreateUserGameMoveEventAvro, UserGameMoveAvro};




impl SerializableEventDto for UserGameEvent {
    fn as_any(&self) -> &dyn Any {
        self
    }
}

impl From<UserGameEvent> for CreateUserGameMoveEventAvro {
    fn from(user_game_event: UserGameEvent) -> Self {
        CreateUserGameMoveEventAvro {
            identifier: format!("{}", user_game_event.id),
            name: user_game_event.name,
            description: user_game_event.description,
            user_game_move: UserGameMoveAvro {
                id: user_game_event.user_game_move.id,
                user_id: user_game_event.user_game_move.user_id,
                game_id: user_game_event.user_game_move.game_id,
                version: user_game_event.user_game_move.version,
                user_move: user_game_event.user_game_move.user_move,
                socket_id: user_game_event.user_game_move.socket_id,
            },
        }
    }
}