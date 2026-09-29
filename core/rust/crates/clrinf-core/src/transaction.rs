// @clrinf:generated — Transaction abstraction for the clrinf Rust runtime.
//
// Provides a Unit of Work pattern trait. Implementations wrap database
// transactions (SQLx, Diesel) or saga coordinators.

use std::future::Future;
use std::pin::Pin;

/// Result type for transaction operations.
pub type TxResult<T> = Result<T, TxError>;

/// Errors that can occur during transaction operations.
#[derive(Debug, thiserror::Error)]
pub enum TxError {
    #[error("transaction commit failed: {0}")]
    CommitFailed(String),
    #[error("transaction rollback: {0}")]
    Rollback(String),
    #[error("transaction error: {0}")]
    Other(String),
}

/// Unit of Work trait for transactional command handling.
///
/// A handler calls `begin()` at the start, `commit()` on success,
/// and `rollback()` on failure. The middleware can wrap this automatically
/// around command handler invocations.
pub trait UnitOfWork: Send + Sync + 'static {
    /// Begin a new transaction scope.
    fn begin(&self) -> Pin<Box<dyn Future<Output = TxResult<()>> + Send + '_>>;

    /// Commit the current transaction.
    fn commit(&self) -> Pin<Box<dyn Future<Output = TxResult<()>> + Send + '_>>;

    /// Rollback the current transaction.
    fn rollback(&self) -> Pin<Box<dyn Future<Output = TxResult<()>> + Send + '_>>;
}
