use anyhow::{bail, Result};
use serde::Deserialize;

use crate::tokens::Claims;

pub trait AuthPolicy: Send + Sync {
    fn is_allowed(&self, claims: &Claims, action: &str, resource: &str) -> bool;
}

#[derive(Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AllowRule {
    pub tenant: String,
    pub principal: String,
    pub action: String,
    pub resource: String,
    #[serde(default)]
    pub role: Option<String>,
    #[serde(default)]
    pub permission: Option<String>,
    #[serde(default)]
    pub is_service_account: Option<bool>,
}

pub struct DeclarativePolicy {
    rules: Vec<AllowRule>,
    allow_wildcards: bool,
}

impl DeclarativePolicy {
    pub fn new(rules: Vec<AllowRule>, allow_wildcards: bool) -> Result<Self> {
        for rule in &rules {
            for selector in [&rule.tenant, &rule.principal, &rule.action, &rule.resource] {
                if selector.trim().is_empty() || (selector == "*" && !allow_wildcards) {
                    bail!("empty policy selector or wildcard without allow_wildcards");
                }
            }
            if rule
                .role
                .iter()
                .chain(rule.permission.iter())
                .any(|v| v.trim().is_empty())
            {
                bail!("empty role or permission constraint");
            }
        }
        Ok(Self {
            rules,
            allow_wildcards,
        })
    }

    fn matches(&self, selector: &str, value: &str) -> bool {
        selector == value || (self.allow_wildcards && selector == "*")
    }
}

impl AuthPolicy for DeclarativePolicy {
    fn is_allowed(&self, claims: &Claims, action: &str, resource: &str) -> bool {
        if action.trim().is_empty() || resource.trim().is_empty() {
            return false;
        }
        self.rules.iter().any(|rule| {
            self.matches(&rule.tenant, &claims.tenant_id)
                && self.matches(&rule.principal, &claims.sub)
                && self.matches(&rule.action, action)
                && self.matches(&rule.resource, resource)
                && rule
                    .role
                    .as_ref()
                    .is_none_or(|role| claims.roles.contains(role))
                && rule
                    .permission
                    .as_ref()
                    .is_none_or(|p| claims.permissions.contains(p))
                && rule
                    .is_service_account
                    .is_none_or(|s| claims.is_service_account == s)
        })
    }
}
