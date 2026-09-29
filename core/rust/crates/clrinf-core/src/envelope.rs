use crate::{context::validate_identifier, RequestContext, ValidationError};
use chrono::{DateTime, SecondsFormat, Utc};
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use uuid::Uuid;

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct EventEnvelope {
    pub specversion: String,
    pub id: String,
    pub source: String,
    #[serde(rename = "type")]
    pub event_type: String,
    pub time: String,
    pub datacontenttype: String,
    pub tenantid: String,
    pub correlationid: String,
    pub causationid: String,
    pub data: Map<String, Value>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "subject"
    )]
    pub subject: Option<String>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "flag"
    )]
    pub is_error: Option<bool>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "flag"
    )]
    pub is_compensation: Option<bool>,
}

fn flag<'de, D: serde::Deserializer<'de>>(deserializer: D) -> Result<Option<bool>, D::Error> {
    bool::deserialize(deserializer).map(Some)
}

fn subject<'de, D: serde::Deserializer<'de>>(deserializer: D) -> Result<Option<String>, D::Error> {
    String::deserialize(deserializer).map(Some)
}

impl EventEnvelope {
    pub fn new(
        context: &RequestContext,
        source: impl Into<String>,
        event_type: impl Into<String>,
        data: Map<String, Value>,
    ) -> Result<Self, ValidationError> {
        context.validate()?;
        let event = Self {
            specversion: "1.0".into(),
            id: Uuid::new_v4().to_string(),
            source: source.into(),
            event_type: event_type.into(),
            time: Utc::now().to_rfc3339_opts(SecondsFormat::Millis, true),
            datacontenttype: "application/json".into(),
            tenantid: context.tenant_id.clone(),
            correlationid: context.correlation_id.clone(),
            causationid: context.causation_id.clone(),
            data,
            subject: None,
            is_error: None,
            is_compensation: None,
        };
        event.validate()?;
        Ok(event)
    }

    pub fn context(&self) -> Result<RequestContext, ValidationError> {
        RequestContext::new(&self.tenantid, &self.correlationid, &self.causationid)
    }

    pub fn validate(&self) -> Result<(), ValidationError> {
        self.context()?;
        validate_identifier(&self.id, "id")?;
        if self.specversion != "1.0" || self.datacontenttype != "application/json" {
            return Err(ValidationError(
                "unsupported envelope version or content type".into(),
            ));
        }
        for (field, value) in [("source", &self.source), ("type", &self.event_type)] {
            if value.trim().is_empty() || value.chars().any(char::is_control) {
                return Err(ValidationError(format!(
                    "{field} must be nonempty without controls"
                )));
            }
        }
        DateTime::parse_from_rfc3339(&self.time)
            .map_err(|_| ValidationError("time must be RFC3339".into()))?;
        Ok(())
    }
}
