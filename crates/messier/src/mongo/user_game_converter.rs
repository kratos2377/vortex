use std::{io::Cursor, sync::Arc};

use axum::async_trait;
use mongodb::error::Error;
use murmur3::murmur3_32;
use schema_registry_converter::{async_impl::avro::AvroEncoder, schema_registry_common::SubjectNameStrategy};
use tracing::instrument;
use uuid::Uuid;

use crate::{common::{schema_create_user_game_event::{ CreateUserGameMoveEventAvro, SCHEMA_NAME_CREATE_USER_GAME_EVENT}, schema_key::{IdentifierAvro, KeyAvro}}, conf::config_types::{KafkaConfiguration, TopicProperties}, kafka};

use super::{event_converter::EventConverter, event_models::{EventDto, SerializableEventDto}, kafka_event_models::UserGameEvent};



pub struct UserGameEventConverter<'a> {
    pub(crate) avro_encoder: Arc<AvroEncoder<'a>>,
    pub(crate) topic_configuration: TopicProperties,
}

impl<'a> UserGameEventConverter<'a> {
    pub fn new<'b>(
        config: &'b KafkaConfiguration,
    ) -> Result<UserGameEventConverter<'a>, Error> {
        Ok(UserGameEventConverter {
            avro_encoder: Arc::new(kafka::avro_decoder::init_avro_encoder(config).unwrap()),
            topic_configuration: config.topic.get_mapping("events"),
        })
    }
}

#[async_trait]
impl<'a> EventConverter for UserGameEventConverter<'a> {
    fn handles(&self, event_type: String) -> bool {
        matches!(
            event_type.as_str(),
            SCHEMA_NAME_CREATE_USER_GAME_EVENT
        )
    }

    #[instrument(name = "create_user_game_event_converter.handle", skip_all)]
    async fn handle(
        &self,
        event_type: String,
        event: &Box<dyn SerializableEventDto>,
    ) -> Result<EventDto, Error> {
        let user_game_event = event
            .as_any()
            .downcast_ref::<UserGameEvent>()
            .unwrap_or_else(|| panic!("Unexpected event type detected: {}", event_type));

        // Determine kafka partition
        let partition = partition_of(user_game_event.id, self.topic_configuration.partitions)
            .expect("Invalid partition number detected");

        // Serialize value
        let value_sns = SubjectNameStrategy::RecordNameStrategy(event_type.clone());
        let serialized_value: Vec<u8> = if event_type == *SCHEMA_NAME_CREATE_USER_GAME_EVENT {
            let create_user_game_avro: CreateUserGameMoveEventAvro =
                user_game_event.clone().into();
            self.avro_encoder
                .encode_struct(create_user_game_avro, &value_sns)
                .await.unwrap()
        } else {
            panic!("Unhandled event type: {:?}", event_type);
        };

        // Serialize key
        let key_avro = KeyAvro {
            context_identifier: format!("{}", user_game_event.id),
            identifier: IdentifierAvro {
                data_type: "user_game_event".to_owned(),
                identifier: format!("{}", user_game_event.id),
                version: user_game_event.version,
            },
        };
        let key_sns = SubjectNameStrategy::RecordNameStrategy("KeyAvro".to_string());

        let serialized_key = self.avro_encoder.encode_struct(key_avro, &key_sns).await.unwrap();

        // Get topic
        let topic = self.topic_configuration.topic_name.clone();

        // Return dto with required parameters to send it with kafka
        Ok(EventDto {
            topic,
            partition,
            key: serialized_key,
            payload: serialized_value,
        })
    }
}

pub fn partition_of(identifier: Uuid, num_partitions: i32) -> std::io::Result<i32> {
    Ok(
        murmur3_32(&mut Cursor::new(identifier.as_bytes()), 0)?.rem_euclid(num_partitions as u32)
            as i32,
    )
}
