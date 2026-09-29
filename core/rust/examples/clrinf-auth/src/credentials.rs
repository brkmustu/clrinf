use anyhow::{bail, Context, Result};
use argon2::{Argon2, PasswordHash, PasswordVerifier};
use clrinf_adapters::AppError;
use serde::Deserialize;
use serde_json::Value;
use std::collections::HashSet;

#[derive(Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PrincipalConfig {
    pub id: String,
    pub credential_hash_env: String,
    pub tenants: Vec<String>,
    #[serde(default)]
    pub roles: Vec<String>,
    #[serde(default)]
    pub permissions: Vec<String>,
    #[serde(default)]
    pub is_service_account: bool,
    #[serde(default)]
    pub profile: Option<Value>,
}

#[derive(Clone)]
pub struct Principal {
    pub id: String,
    pub roles: Vec<String>,
    pub permissions: Vec<String>,
    pub is_service_account: bool,
    pub profile: Option<Value>,
}

/// Credential stores must authenticate both the credential and exact tenant membership.
pub trait CredentialAdapter: Send + Sync {
    fn authenticate(
        &self,
        id: &str,
        credential: &str,
        tenant: &str,
        is_service_account: bool,
    ) -> Result<Principal, AppError>;

    fn profiles(&self, _tenant: &str) -> Vec<Value> {
        Vec::new()
    }
}

struct StoredPrincipal {
    principal: Principal,
    tenants: Vec<String>,
    hash: String,
}

pub struct ConfiguredCredentials {
    principals: Vec<StoredPrincipal>,
    expose_profiles: bool,
}

impl ConfiguredCredentials {
    pub fn from_config(configs: Vec<PrincipalConfig>, expose_profiles: bool) -> Result<Self> {
        Self::with_resolver(configs, expose_profiles, |name| {
            std::env::var(name).context("credential hash environment variable is missing")
        })
    }

    pub fn with_resolver(
        configs: Vec<PrincipalConfig>,
        expose_profiles: bool,
        resolve: impl Fn(&str) -> Result<String>,
    ) -> Result<Self> {
        let mut principals = Vec::new();
        let mut identities = HashSet::new();
        for config in configs {
            if config.id.trim().is_empty()
                || config.credential_hash_env.trim().is_empty()
                || config.tenants.is_empty()
                || config
                    .tenants
                    .iter()
                    .any(|t| t.trim().is_empty() || t == "*")
                || config
                    .roles
                    .iter()
                    .chain(&config.permissions)
                    .any(|v| v.trim().is_empty())
                || !identities.insert((config.id.clone(), config.is_service_account))
            {
                bail!("invalid or duplicate principal configuration");
            }
            let hash = resolve(&config.credential_hash_env)?;
            let parsed = PasswordHash::new(&hash)
                .map_err(|_| anyhow::anyhow!("credential must be an Argon2id PHC hash"))?;
            if parsed.algorithm.as_str() != "argon2id"
                || parsed.salt.is_none()
                || parsed.hash.is_none()
                || argon2::Params::try_from(&parsed).is_err()
            {
                bail!("credential must be a complete Argon2id PHC hash");
            }
            principals.push(StoredPrincipal {
                principal: Principal {
                    id: config.id,
                    roles: config.roles,
                    permissions: config.permissions,
                    is_service_account: config.is_service_account,
                    profile: config.profile,
                },
                tenants: config.tenants,
                hash,
            });
        }
        Ok(Self {
            principals,
            expose_profiles,
        })
    }
}

impl CredentialAdapter for ConfiguredCredentials {
    fn authenticate(
        &self,
        id: &str,
        credential: &str,
        tenant: &str,
        is_service_account: bool,
    ) -> Result<Principal, AppError> {
        let invalid = || AppError::Unauthorized("Invalid credentials or tenant".into());
        if credential.is_empty() {
            return Err(invalid());
        }
        let stored = self
            .principals
            .iter()
            .find(|p| {
                p.principal.id == id
                    && p.principal.is_service_account == is_service_account
                    && p.tenants.iter().any(|t| t == tenant)
            })
            .ok_or_else(invalid)?;
        let hash = PasswordHash::new(&stored.hash)
            .map_err(|_| AppError::Internal("Invalid configured credential hash".into()))?;
        Argon2::default()
            .verify_password(credential.as_bytes(), &hash)
            .map_err(|_| invalid())?;
        Ok(stored.principal.clone())
    }

    fn profiles(&self, tenant: &str) -> Vec<Value> {
        if !self.expose_profiles {
            return Vec::new();
        }
        self.principals
            .iter()
            .filter(|p| !p.principal.is_service_account && p.tenants.iter().any(|t| t == tenant))
            .filter_map(|p| p.principal.profile.clone())
            .collect()
    }
}
