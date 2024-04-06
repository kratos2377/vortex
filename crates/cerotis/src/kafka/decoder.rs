use axum::async_trait;
use rdkafka::error::KafkaError;
use schema_registry_converter::{async_impl::{avro::{AvroDecoder, AvroEncoder}, schema_registry::SrSettings}, avro_common::DecodeResult, error::SRCError};

use crate::conf::config_types::KafkaConfiguration;




pub fn init_avro_encoder<'a, 'b>(
    config: &'a KafkaConfiguration,
) -> Result<AvroEncoder<'b>, KafkaError> {
    let sr_settings = resolve_sr_settings(config)?;
    Ok(AvroEncoder::new(sr_settings))
}

pub fn init_avro_decoder<'a, 'b>(
    config: &'a KafkaConfiguration,
) -> Result<AvroDecoder<'b>, KafkaError> {
    let sr_settings = resolve_sr_settings(config)?;
    Ok(AvroDecoder::new(sr_settings))
}

pub fn resolve_sr_settings(config: &KafkaConfiguration) -> Result<SrSettings, KafkaError> {
    Ok(SrSettings::new_builder(config.schema_registry.url.clone()).build().unwrap())
}

#[async_trait]
pub trait RecordDecoder: Send + Sync {
    async fn decode(&self, bytes: Option<&[u8]>) -> Result<DecodeResult, SRCError>;
}


pub struct AvroRecordDecoder<'a> {
    pub avro_decoder: AvroDecoder<'a>,
}

impl<'a> AvroRecordDecoder<'a> {
    pub fn new<'b>(config: &'b KafkaConfiguration) -> Result<AvroRecordDecoder<'a>, KafkaError> {
        Ok(AvroRecordDecoder {
            avro_decoder: init_avro_decoder(config)?,
        })
    }
}

#[async_trait]
impl<'a> RecordDecoder for AvroRecordDecoder<'a> {
    async fn decode(&self, bytes: Option<&[u8]>) -> Result<DecodeResult, SRCError> {
        self.avro_decoder.decode(bytes).await
    }
}
