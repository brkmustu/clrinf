// @clrinf:generated — Authentication abstraction for the clrinf Rust runtime.
//
// Provides JWT token creation/verification traits. The default implementation
// uses the `jsonwebtoken` crate but callers depend only on this trait.

use serde::{Deserialize, Serialize};
use std::future::Future;
use std::pin::Pin;

/// Result type for auth operations.
pub type AuthResult<T> = Result<T, AuthError>;

/// Errors that can occur during authentication.
#[derive(Debug, thiserror::Error)]
pub enum AuthError {
    #[error("invalid token: {0}")]
    InvalidToken(String),
    #[error("token expired")]
    TokenExpired,
    #[error("missing credentials")]
    MissingCredentials,
    #[error("auth error: {0}")]
    Other(String),
}

/// Standard JWT claims carried through the pipeline.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,
    pub tenant_id: String,
    pub roles: Vec<String>,
    pub exp: u64,
    pub iat: u64,
}

/// Token service trait for creating and verifying authentication tokens.
pub trait TokenService: Send + Sync + 'static {
    /// Create a signed token for the given claims.
    fn create_token(
        &self,
        claims: &Claims,
    ) -> Pin<Box<dyn Future<Output = AuthResult<String>> + Send + '_>>;

    /// Verify and decode a token, returning the embedded claims.
    fn verify_token(
        &self,
        token: &str,
    ) -> Pin<Box<dyn Future<Output = AuthResult<Claims>> + Send + '_>>;
}
