// @clrinf:generated — Multi-Tenant Authorization abstraction for the clrinf Rust runtime.
//
// Provides policy-based access control traits with default multi-tenant isolation,
// Cedar-compatible ABAC/RBAC rule evaluation, and explicit forbid guarantees.

use super::auth::Claims;
use std::future::Future;
use std::pin::Pin;

/// Result type for authorization operations.
pub type AuthzResult = Result<AuthzDecision, AuthzError>;

/// Authorization decision with detailed auditing context.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AuthzDecision {
    Allow {
        reason: Option<String>,
    },
    Deny {
        reason: String,
        is_tenant_violation: bool,
    },
}

impl AuthzDecision {
    pub fn is_allowed(&self) -> bool {
        matches!(self, AuthzDecision::Allow { .. })
    }

    pub fn allow() -> Self {
        AuthzDecision::Allow { reason: None }
    }

    pub fn allow_with_reason(reason: impl Into<String>) -> Self {
        AuthzDecision::Allow {
            reason: Some(reason.into()),
        }
    }

    pub fn deny(reason: impl Into<String>) -> Self {
        AuthzDecision::Deny {
            reason: reason.into(),
            is_tenant_violation: false,
        }
    }

    pub fn tenant_violation(principal_tenant: &str, resource_tenant: &str) -> Self {
        AuthzDecision::Deny {
            reason: format!(
                "Tenant isolation violation: principal tenant '{}' cannot access resource tenant '{}'",
                principal_tenant, resource_tenant
            ),
            is_tenant_violation: true,
        }
    }
}

/// Errors that can occur during authorization.
#[derive(Debug, thiserror::Error)]
pub enum AuthzError {
    #[error("access denied: {0}")]
    AccessDenied(String),
    #[error("insufficient permissions for operation: {0}")]
    InsufficientPermissions(String),
    #[error("tenant isolation violation: {0}")]
    TenantViolation(String),
    #[error("authorization error: {0}")]
    Other(String),
}

/// An operation claim required for a specific action (e.g., "orders.create"),
/// carrying an optional resource tenant for cross-tenant isolation enforcement.
#[derive(Debug, Clone)]
pub struct OperationClaim {
    pub resource: String,
    pub action: String,
    pub resource_tenant_id: Option<String>,
}

impl OperationClaim {
    pub fn new(resource: impl Into<String>, action: impl Into<String>) -> Self {
        Self {
            resource: resource.into(),
            action: action.into(),
            resource_tenant_id: None,
        }
    }

    pub fn with_tenant(
        resource: impl Into<String>,
        action: impl Into<String>,
        tenant_id: impl Into<String>,
    ) -> Self {
        Self {
            resource: resource.into(),
            action: action.into(),
            resource_tenant_id: Some(tenant_id.into()),
        }
    }
}

/// Cedar-compatible policy rule representation.
#[derive(Debug, Clone)]
pub struct CedarPolicyRule {
    pub effect_permit: bool,
    pub principal_role: Option<String>,
    pub action: Option<String>,
    pub resource_type: Option<String>,
    pub require_tenant_match: bool,
}

/// Authorization service trait for evaluating access policies.
pub trait AuthorizationService: Send + Sync + 'static {
    /// Check whether the given claims satisfy the required operation claims
    /// under default multi-tenant isolation rules.
    fn authorize(
        &self,
        claims: &Claims,
        required: &[OperationClaim],
    ) -> Pin<Box<dyn Future<Output = AuthzResult> + Send + '_>>;
}

/// Default Multi-Tenant Cedar Policy Engine.
/// Guarantees:
/// 1. Tenant Isolation by default: Cross-tenant access is strictly denied (Forbid),
///    unless the principal has the PlatformAdmin role.
/// 2. Role-based permit checks aligned with Cedar policies.
/// 3. Cedar Forbid rules always override permit rules.
pub struct MultiTenantCedarAuthorizer {
    platform_admin_role: String,
    rules: Vec<CedarPolicyRule>,
}

impl MultiTenantCedarAuthorizer {
    pub fn new(platform_admin_role: impl Into<String>) -> Self {
        Self {
            platform_admin_role: platform_admin_role.into(),
            rules: Vec::new(),
        }
    }

    pub fn with_rule(mut self, rule: CedarPolicyRule) -> Self {
        self.rules.push(rule);
        self
    }
}

impl Default for MultiTenantCedarAuthorizer {
    fn default() -> Self {
        Self::new("PlatformAdmin")
    }
}

impl AuthorizationService for MultiTenantCedarAuthorizer {
    fn authorize(
        &self,
        claims: &Claims,
        required: &[OperationClaim],
    ) -> Pin<Box<dyn Future<Output = AuthzResult> + Send + '_>> {
        let claims = claims.clone();
        let required = required.to_vec();
        let platform_admin = self.platform_admin_role.clone();
        let rules = self.rules.clone();

        Box::pin(async move {
            let is_platform_admin = claims.roles.iter().any(|r| r == &platform_admin);

            for op in &required {
                // 1. Multi-Tenant İzolasyon Kontrolü (Default Tenant Guard)
                if let Some(ref res_tenant) = op.resource_tenant_id {
                    if res_tenant != &claims.tenant_id && !is_platform_admin {
                        return Ok(AuthzDecision::tenant_violation(&claims.tenant_id, res_tenant));
                    }
                }

                // PlatformAdmin tüm izin kurallarını bypass eder
                if is_platform_admin {
                    continue;
                }

                // 2. Cedar Forbid Kuralları Kontrolü (Forbid her zaman Permit'i ezer)
                for forbid_rule in rules.iter().filter(|r| !r.effect_permit) {
                    let role_match = forbid_rule
                        .principal_role
                        .as_ref()
                        .map_or(true, |r| claims.roles.iter().any(|cr| cr == r));
                    let action_match = forbid_rule
                        .action
                        .as_ref()
                        .map_or(true, |a| a == &op.action || a == "*");
                    let res_match = forbid_rule
                        .resource_type
                        .as_ref()
                        .map_or(true, |rt| rt == &op.resource || rt == "*");

                    if role_match && action_match && res_match {
                        return Ok(AuthzDecision::deny(format!(
                            "Explicit Cedar FORBID triggered for {}:{}",
                            op.resource, op.action
                        )));
                    }
                }

                // 3. İzin Kontrolü (TenantAdmin, spesifik rol veya Claim)
                let is_tenant_admin = claims.roles.iter().any(|r| r == "TenantAdmin" || r == "Admin");
                let expected_permission = format!("{}.{}", op.resource, op.action);
                let has_role_permission = claims
                    .roles
                    .iter()
                    .any(|r| r == &expected_permission || r == &op.resource);

                if !is_tenant_admin && !has_role_permission {
                    // Ek tanımlı Cedar permit kuralları kontrolü
                    let mut permitted = false;
                    for permit_rule in rules.iter().filter(|r| r.effect_permit) {
                        let role_match = permit_rule
                            .principal_role
                            .as_ref()
                            .map_or(false, |r| claims.roles.iter().any(|cr| cr == r));
                        let action_match = permit_rule
                            .action
                            .as_ref()
                            .map_or(true, |a| a == &op.action || a == "*");
                        let res_match = permit_rule
                            .resource_type
                            .as_ref()
                            .map_or(true, |rt| rt == &op.resource || rt == "*");

                        if role_match && action_match && res_match {
                            permitted = true;
                            break;
                        }
                    }

                    if !permitted {
                        return Ok(AuthzDecision::deny(format!(
                            "Insufficient permissions for operation: {}",
                            expected_permission
                        )));
                    }
                }
            }

            Ok(AuthzDecision::allow())
        })
    }
}
