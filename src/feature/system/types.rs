use crate::utils::error::{Error, Result};

use super::ConfigEntity;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct ConfigDto {
    pub id: i32,
    pub allow_recovery_codes: bool,
    pub allow_refresh_tokens: bool,
    pub token_validity_seconds: i32,
    pub refresh_token_validity_seconds: i32,
    pub ai_model: String,
    pub vector_similarity_threshold: i32,
}

impl ConfigDto {
    pub fn validate(&self) -> Result<()> {
        if self.token_validity_seconds <= 0 {
            return Err(Error::Validation(
                "token_validity_seconds must be > 0".into(),
            ));
        }
        Ok(())
    }
}

impl From<ConfigEntity> for ConfigDto {
    fn from(e: ConfigEntity) -> Self {
        Self {
            id: e.id,
            allow_recovery_codes: e.allow_recovery_codes,
            allow_refresh_tokens: e.allow_refresh_tokens,
            token_validity_seconds: e.token_validity_seconds,
            refresh_token_validity_seconds: e.refresh_token_validity_seconds,
            ai_model: e.ai_model,
            vector_similarity_threshold: e.vector_similarity_threshold,
        }
    }
}

impl From<&ConfigDto> for ConfigEntity {
    fn from(d: &ConfigDto) -> Self {
        Self {
            id: d.id,
            allow_recovery_codes: d.allow_recovery_codes,
            allow_refresh_tokens: d.allow_refresh_tokens,
            token_validity_seconds: d.token_validity_seconds,
            refresh_token_validity_seconds: d.refresh_token_validity_seconds,
            ai_model: d.ai_model.clone(),
            vector_similarity_threshold: d.vector_similarity_threshold,
        }
    }
}
