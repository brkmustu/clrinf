// @clrinf:generated — Outbox abstraction for the clrinf Rust runtime.
//
// Provides a transactional outbox pattern trait matching the TypeScript
// OutboxStore/dispatchOutbox contract. Guarantees reliable event publishing.

use std::future::Future;
use std::pin::Pin;

/// Result type for outbox operations.
pub type OutboxResult<T> = Result<T, OutboxError>;

/// Errors from outbox operations.
#[derive(Debug, thiserror::Error)]
pub enum OutboxError {
    #[error("duplicate event: {0}")]
    Duplicate(String),
    #[error("outbox error: {0}")]
    Other(String),
}

/// A claimed outbox delivery to be published.
#[derive(Debug, Clone)]
pub struct OutboxDelivery {
    pub event: serde_json::Value,
    pub token: String,
}

/// Outbox store trait for reliable event publishing.
pub trait OutboxStore: Send + Sync + 'static {
    /// Append an event to the outbox.
    fn append(
        &self,
        event: serde_json::Value,
    ) -> Pin<Box<dyn Future<Output = OutboxResult<()>> + Send + '_>>;

    /// Claim up to `limit` pending events for delivery.
    fn claim(
        &self,
        limit: usize,
        lease_ms: u64,
    ) -> Pin<Box<dyn Future<Output = OutboxResult<Vec<OutboxDelivery>>> + Send + '_>>;

    /// Acknowledge successful delivery.
    fn acknowledge(
        &self,
        tenant_id: &str,
        event_id: &str,
        token: &str,
    ) -> Pin<Box<dyn Future<Output = OutboxResult<()>> + Send + '_>>;

    /// Return an event to the outbox after a failed delivery attempt.
    fn retry(
        &self,
        tenant_id: &str,
        event_id: &str,
        token: &str,
        delay_ms: u64,
    ) -> Pin<Box<dyn Future<Output = OutboxResult<()>> + Send + '_>>;
}
