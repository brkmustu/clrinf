use crate::{context::validate_identifier, RequestContext, ValidationError};
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct ErrorEnvelope {
    pub error_code: String,
    pub message: String,
    pub correlation_id: String,
    pub tenant_id: String,
    pub retryable: bool,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "details"
    )]
    pub details: Option<Map<String, Value>>,
}

fn details<'de, D: serde::Deserializer<'de>>(
    deserializer: D,
) -> Result<Option<Map<String, Value>>, D::Error> {
    Map::deserialize(deserializer).map(Some)
}

impl ErrorEnvelope {
    pub fn new(
        context: &RequestContext,
        code: impl Into<String>,
        message: impl Into<String>,
        retryable: bool,
    ) -> Result<Self, ValidationError> {
        context.validate()?;
        let error = Self {
            error_code: code.into(),
            message: message.into(),
            correlation_id: context.correlation_id.clone(),
            tenant_id: context.tenant_id.clone(),
            retryable,
            details: None,
        };
        error.validate()?;
        Ok(error)
    }

    pub fn validate(&self) -> Result<(), ValidationError> {
        validate_identifier(&self.tenant_id, "tenant_id")?;
        validate_identifier(&self.correlation_id, "correlation_id")?;
        validate_identifier(&self.error_code, "error_code")?;
        if self.message.trim().is_empty() {
            return Err(ValidationError("message must not be empty".into()));
        }
        Ok(())
    }
}
