use serde::{Deserialize, Serialize};
use std::{error::Error, fmt};
use uuid::Uuid;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ValidationError(pub String);

impl fmt::Display for ValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl Error for ValidationError {}

pub fn validate_identifier(value: &str, field: &str) -> Result<(), ValidationError> {
    if value.is_empty()
        || value.len() > 128
        || !value
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b"._:-".contains(&b))
    {
        return Err(ValidationError(format!(
            "{field} must contain 1-128 ASCII letters, digits, '.', '_', ':' or '-'"
        )));
    }
    Ok(())
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(try_from = "ContextWire")]
pub struct RequestContext {
    pub tenant_id: String,
    pub correlation_id: String,
    pub causation_id: String,
}

#[derive(Deserialize)]
struct ContextWire {
    tenant_id: String,
    correlation_id: String,
    causation_id: String,
}

impl TryFrom<ContextWire> for RequestContext {
    type Error = ValidationError;

    fn try_from(value: ContextWire) -> Result<Self, Self::Error> {
        Self::new(value.tenant_id, value.correlation_id, value.causation_id)
    }
}

impl RequestContext {
    pub fn new(
        tenant_id: impl Into<String>,
        correlation_id: impl Into<String>,
        causation_id: impl Into<String>,
    ) -> Result<Self, ValidationError> {
        let context = Self {
            tenant_id: tenant_id.into(),
            correlation_id: correlation_id.into(),
            causation_id: causation_id.into(),
        };
        context.validate()?;
        Ok(context)
    }

    pub fn root(tenant_id: impl Into<String>) -> Result<Self, ValidationError> {
        Self::new(
            tenant_id,
            Uuid::new_v4().to_string(),
            Uuid::new_v4().to_string(),
        )
    }

    pub fn validate(&self) -> Result<(), ValidationError> {
        validate_identifier(&self.tenant_id, "tenant_id")?;
        validate_identifier(&self.correlation_id, "correlation_id")?;
        validate_identifier(&self.causation_id, "causation_id")
    }

    pub fn child(&self, cause: impl Into<String>) -> Result<Self, ValidationError> {
        self.validate()?;
        Self::new(&self.tenant_id, &self.correlation_id, cause)
    }
}
