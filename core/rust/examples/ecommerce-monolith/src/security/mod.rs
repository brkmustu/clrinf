use clrinf_core::authz::{CedarPolicyRule, MultiTenantCedarAuthorizer};

pub fn create_ecommerce_authorizer() -> MultiTenantCedarAuthorizer {
    MultiTenantCedarAuthorizer::new("PlatformAdmin")
        // 1. StoreAdmin - Full access
        .with_rule(CedarPolicyRule {
            effect_permit: true,
            principal_role: Some("StoreAdmin".into()),
            action: None,
            resource_type: Some("Catalog".into()),
            require_tenant_match: true,
        })
        .with_rule(CedarPolicyRule {
            effect_permit: true,
            principal_role: Some("StoreAdmin".into()),
            action: None,
            resource_type: Some("Inventory".into()),
            require_tenant_match: true,
        })
        .with_rule(CedarPolicyRule {
            effect_permit: true,
            principal_role: Some("StoreAdmin".into()),
            action: None,
            resource_type: Some("Orders".into()),
            require_tenant_match: true,
        })
        // 2. CatalogManager
        .with_rule(CedarPolicyRule {
            effect_permit: true,
            principal_role: Some("CatalogManager".into()),
            action: None,
            resource_type: Some("Catalog".into()),
            require_tenant_match: true,
        })
        .with_rule(CedarPolicyRule {
            effect_permit: true,
            principal_role: Some("CatalogManager".into()),
            action: Some("inventory.read".into()),
            resource_type: Some("Inventory".into()),
            require_tenant_match: true,
        })
        // 3. Customer
        .with_rule(CedarPolicyRule {
            effect_permit: true,
            principal_role: Some("Customer".into()),
            action: Some("catalog.read".into()),
            resource_type: Some("Catalog".into()),
            require_tenant_match: true,
        })
        .with_rule(CedarPolicyRule {
            effect_permit: true,
            principal_role: Some("Customer".into()),
            action: Some("inventory.reserve".into()),
            resource_type: Some("Inventory".into()),
            require_tenant_match: true,
        })
        .with_rule(CedarPolicyRule {
            effect_permit: true,
            principal_role: Some("Customer".into()),
            action: Some("inventory.release".into()),
            resource_type: Some("Inventory".into()),
            require_tenant_match: true,
        })
        .with_rule(CedarPolicyRule {
            effect_permit: true,
            principal_role: Some("Customer".into()),
            action: Some("inventory.commit".into()),
            resource_type: Some("Inventory".into()),
            require_tenant_match: true,
        })
        .with_rule(CedarPolicyRule {
            effect_permit: true,
            principal_role: Some("Customer".into()),
            action: Some("orders.create".into()),
            resource_type: Some("Orders".into()),
            require_tenant_match: true,
        })
        .with_rule(CedarPolicyRule {
            effect_permit: true,
            principal_role: Some("Customer".into()),
            action: Some("orders.read".into()),
            resource_type: Some("Orders".into()),
            require_tenant_match: true,
        })
        .with_rule(CedarPolicyRule {
            effect_permit: true,
            principal_role: Some("Customer".into()),
            action: Some("orders.cancel".into()),
            resource_type: Some("Orders".into()),
            require_tenant_match: true,
        })
}
