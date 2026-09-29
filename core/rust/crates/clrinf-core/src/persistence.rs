use crate::{EventEnvelope, ValidationError};
use serde_json::Value;
use std::{error::Error, fmt};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Lease {
    pub tenant_id: String,
    pub key: String,
    pub token: String,
}

#[derive(Clone, Debug, PartialEq)]
pub enum Claim {
    Acquired(Lease),
    Replay(Value),
    InProgress,
    Conflict,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum StoreError {
    Invalid(ValidationError),
    Conflict,
    InProgress,
    LeaseLost,
    DuplicateEvent,
    NotFound,
    Unavailable(String),
}

impl fmt::Display for StoreError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{self:?}")
    }
}

impl Error for StoreError {}

impl From<ValidationError> for StoreError {
    fn from(value: ValidationError) -> Self {
        Self::Invalid(value)
    }
}

/// Synchronous port. Blocking durable implementations must run off an async executor's workers.
/// `now` and lease duration are Unix seconds supplied by a trusted application clock.
pub trait IdempotencyStore: Send + Sync {
    fn claim(
        &self,
        tenant: &str,
        key: &str,
        fingerprint: &str,
        now: u64,
        lease_seconds: u64,
    ) -> Result<Claim, StoreError>;

    fn abandon(&self, lease: &Lease) -> Result<(), StoreError>;
}

pub trait OutboxStore: Send + Sync {
    fn pending(&self, tenant: &str, limit: usize) -> Result<Vec<EventEnvelope>, StoreError>;
    fn mark_published(&self, tenant: &str, event_id: &str) -> Result<(), StoreError>;
}

/// Commits the idempotency response and outbox records together or changes neither.
/// This does NOT include unrelated business state. Durable applications must enlist their
/// business writes in the same database transaction through an implementation-specific unit of work.
pub trait WorkflowStore: IdempotencyStore + OutboxStore {
    fn commit(
        &self,
        lease: &Lease,
        now: u64,
        response: Value,
        events: Vec<EventEnvelope>,
    ) -> Result<(), StoreError>;
}
