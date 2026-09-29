use clrinf_core::auth::Claims;
use clrinf_core::authz::{
    AuthorizationService, AuthzDecision, CedarPolicyRule, MultiTenantCedarAuthorizer,
    OperationClaim,
};

#[tokio::test]
async fn test_multi_tenant_cedar_permits_matching_role_and_tenant() {
    let authorizer = MultiTenantCedarAuthorizer::default().with_rule(CedarPolicyRule {
        effect_permit: true,
        principal_role: Some("PricingOfficer".into()),
        action: Some("update_price".into()),
        resource_type: Some("product".into()),
        require_tenant_match: true,
    });

    let claims = Claims {
        sub: "user-1".into(),
        tenant_id: "tenant-a".into(),
        roles: vec!["PricingOfficer".into()],
        exp: 9999999999,
        iat: 1000000000,
    };

    let op = OperationClaim::with_tenant("product", "update_price", "tenant-a");
    let result = authorizer.authorize(&claims, &[op]).await.unwrap();

    assert!(result.is_allowed());
}

#[tokio::test]
async fn test_multi_tenant_isolation_violation_denied_by_default() {
    let authorizer = MultiTenantCedarAuthorizer::default().with_rule(CedarPolicyRule {
        effect_permit: true,
        principal_role: Some("PricingOfficer".into()),
        action: Some("update_price".into()),
        resource_type: Some("product".into()),
        require_tenant_match: true,
    });

    let claims = Claims {
        sub: "user-1".into(),
        tenant_id: "tenant-a".into(),
        roles: vec!["PricingOfficer".into()],
        exp: 9999999999,
        iat: 1000000000,
    };

    // User is in tenant-a, but resource is in tenant-b!
    let op = OperationClaim::with_tenant("product", "update_price", "tenant-b");
    let result = authorizer.authorize(&claims, &[op]).await.unwrap();

    match result {
        AuthzDecision::Deny {
            is_tenant_violation,
            reason,
        } => {
            assert!(is_tenant_violation);
            assert!(reason.contains("Tenant isolation violation"));
        }
        _ => panic!("Expected tenant isolation violation deny"),
    }
}

#[tokio::test]
async fn test_platform_admin_bypasses_tenant_isolation() {
    let authorizer = MultiTenantCedarAuthorizer::default();

    let claims = Claims {
        sub: "super-admin".into(),
        tenant_id: "system".into(),
        roles: vec!["PlatformAdmin".into()],
        exp: 9999999999,
        iat: 1000000000,
    };

    let op = OperationClaim::with_tenant("product", "update_price", "tenant-b");
    let result = authorizer.authorize(&claims, &[op]).await.unwrap();

    assert!(result.is_allowed());
}

#[tokio::test]
async fn test_cedar_explicit_forbid_overrides_permit() {
    let authorizer = MultiTenantCedarAuthorizer::default()
        .with_rule(CedarPolicyRule {
            effect_permit: true,
            principal_role: Some("PricingOfficer".into()),
            action: Some("update_price".into()),
            resource_type: Some("product".into()),
            require_tenant_match: true,
        })
        .with_rule(CedarPolicyRule {
            effect_permit: false, // Forbid
            principal_role: Some("Suspended".into()),
            action: Some("*".into()),
            resource_type: Some("*".into()),
            require_tenant_match: false,
        });

    let claims = Claims {
        sub: "user-suspended".into(),
        tenant_id: "tenant-a".into(),
        roles: vec!["PricingOfficer".into(), "Suspended".into()],
        exp: 9999999999,
        iat: 1000000000,
    };

    let op = OperationClaim::with_tenant("product", "update_price", "tenant-a");
    let result = authorizer.authorize(&claims, &[op]).await.unwrap();

    match result {
        AuthzDecision::Deny { reason, .. } => {
            assert!(reason.contains("Explicit Cedar FORBID triggered"));
        }
        _ => panic!("Expected explicit forbid deny"),
    }
}
