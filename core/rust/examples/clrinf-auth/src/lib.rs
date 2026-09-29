/// clrinf-auth: domain-agnostic IAM library.
///
/// Provides:
/// - `CredentialAdapter` trait + `ConfiguredCredentials` (Argon2id)
/// - `AuthPolicy` trait + `DeclarativePolicy` (rule-based, no hardcoded roles)
/// - `JwtService` (RS256 + JWKS), `Claims`
///
/// No domain-specific roles, actions, or hardcoded permissions appear here.
/// Callers supply their own principal configs, policy rules, and RSA key material.
pub mod credentials;
pub mod policy;
pub mod tokens;

pub use credentials::{ConfiguredCredentials, CredentialAdapter, Principal, PrincipalConfig};
pub use policy::{AllowRule, AuthPolicy, DeclarativePolicy};
pub use tokens::{Claims, JwtConfig, JwtService};
