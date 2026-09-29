// @clrinf:generated — Idempotency abstraction for the clrinf Rust runtime.
//
// Provides an idempotency store trait matching the TypeScript MemoryIdempotencyStore
// contract. Ensures at-most-once processing of duplicate requests.

use std::future::Future;
use std::pin::Pin;

/// Result type for idempotency operations.
pub type IdempotencyResult<T> = Result<T, IdempotencyError>;

/// Errors from idempotency operations.
#[derive(Debug, thiserror::Error)]
pub enum IdempotencyError {
    #[error("capacity exceeded")]
    CapacityExceeded,
    #[error("claim not owned")]
    NotOwned,
    #[error("idempotency error: {0}")]
    Other(String),
}

/// Idempotency key identifying a specific operation.
#[derive(Debug, Clone)]
pub struct IdempotencyKey {
    pub tenant_id: String,
    pub operation: String,
    pub key: String,
}

/// Result of attempting to claim an idempotency slot.
#[derive(Debug, Clone)]
pub enum IdempotencyClaim {
    /// Slot acquired, caller should proceed with the token.
    Acquired { token: String },
    /// A previous execution already completed; the stored result is returned.
    Completed { result: serde_json::Value },
    /// Another worker is currently processing this operation.
    Busy,
}

/// Idempotency store trait. Implementations provide at-most-once guarantees.
pub trait IdempotencyStore: Send + Sync + 'static {
    fn claim(
        &self,
        key: &IdempotencyKey,
    ) -> Pin<Box<dyn Future<Output = IdempotencyResult<IdempotencyClaim>> + Send + '_>>;

    fn complete(
        &self,
        key: &IdempotencyKey,
        token: &str,
        result: serde_json::Value,
    ) -> Pin<Box<dyn Future<Output = IdempotencyResult<()>> + Send + '_>>;

    fn release(
        &self,
        key: &IdempotencyKey,
        token: &str,
    ) -> Pin<Box<dyn Future<Output = IdempotencyResult<()>> + Send + '_>>;
}
