// @clrinf:generated — Cache abstraction for the clrinf Rust runtime.
//
// Provides a trait-based caching contract that can be implemented by
// different backends (in-memory, Redis, etc.). The default implementation
// uses DashMap for thread-safe in-process caching.

use std::future::Future;
use std::pin::Pin;
use std::time::Duration;

/// Result type for cache operations.
pub type CacheResult<T> = Result<T, CacheError>;

/// Errors that can occur during cache operations.
#[derive(Debug, thiserror::Error)]
pub enum CacheError {
    #[error("serialization error: {0}")]
    Serialization(String),
    #[error("connection error: {0}")]
    Connection(String),
    #[error("cache error: {0}")]
    Other(String),
}

/// Cache service trait. All cache implementations must satisfy this contract.
///
/// Values are stored as `Vec<u8>` (serialized bytes) to remain type-agnostic
/// at the trait level. Callers serialize/deserialize via `serde_json` or
/// another codec.
pub trait CachePort: Send + Sync + 'static {
    /// Store a value with an optional TTL.
    fn set(
        &self,
        key: &str,
        value: Vec<u8>,
        ttl: Option<Duration>,
    ) -> Pin<Box<dyn Future<Output = CacheResult<()>> + Send + '_>>;

    /// Retrieve a value. Returns `Ok(None)` on miss.
    fn get(&self, key: &str) -> Pin<Box<dyn Future<Output = CacheResult<Option<Vec<u8>>>> + Send + '_>>;

    /// Remove a single key.
    fn remove(&self, key: &str) -> Pin<Box<dyn Future<Output = CacheResult<()>> + Send + '_>>;

    /// Remove all keys that start with `prefix`.
    fn remove_by_prefix(&self, prefix: &str) -> Pin<Box<dyn Future<Output = CacheResult<()>> + Send + '_>>;
}
