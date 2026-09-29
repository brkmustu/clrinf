use clrinf_core::authz::{CedarPolicyRule, MultiTenantCedarAuthorizer};

pub fn create_crm_authorizer() -> MultiTenantCedarAuthorizer {
    MultiTenantCedarAuthorizer::new("PlatformAdmin")
        // 1. Contacts
        .with_rule(CedarPolicyRule {
            effect_permit: true,
            principal_role: Some("TenantAdmin".into()),
            action: None,
            resource_type: Some("Contacts".into()),
            require_tenant_match: true,
        })
        .with_rule(CedarPolicyRule {
            effect_permit: true,
            principal_role: Some("SalesManager".into()),
            action: None,
            resource_type: Some("Contacts".into()),
            require_tenant_match: true,
        })
        .with_rule(CedarPolicyRule {
            effect_permit: true,
            principal_role: Some("SalesRep".into()),
            action: Some("contacts.create".into()),
            resource_type: Some("Contacts".into()),
            require_tenant_match: true,
        })
        .with_rule(CedarPolicyRule {
            effect_permit: true,
            principal_role: Some("SalesRep".into()),
            action: Some("contacts.read".into()),
            resource_type: Some("Contacts".into()),
            require_tenant_match: true,
        })
        // 2. Deals
        .with_rule(CedarPolicyRule {
            effect_permit: true,
            principal_role: Some("TenantAdmin".into()),
            action: None,
            resource_type: Some("Deals".into()),
            require_tenant_match: true,
        })
        .with_rule(CedarPolicyRule {
            effect_permit: true,
            principal_role: Some("SalesManager".into()),
            action: None,
            resource_type: Some("Deals".into()),
            require_tenant_match: true,
        })
        .with_rule(CedarPolicyRule {
            effect_permit: true,
            principal_role: Some("SalesRep".into()),
            action: Some("deals.create".into()),
            resource_type: Some("Deals".into()),
            require_tenant_match: true,
        })
        .with_rule(CedarPolicyRule {
            effect_permit: true,
            principal_role: Some("SalesRep".into()),
            action: Some("deals.read".into()),
            resource_type: Some("Deals".into()),
            require_tenant_match: true,
        })
        .with_rule(CedarPolicyRule {
            effect_permit: true,
            principal_role: Some("SalesRep".into()),
            action: Some("deals.update".into()),
            resource_type: Some("Deals".into()),
            require_tenant_match: true,
        })
        .with_rule(CedarPolicyRule {
            effect_permit: true,
            principal_role: Some("SalesRep".into()),
            action: Some("deals.stage_change".into()),
            resource_type: Some("Deals".into()),
            require_tenant_match: true,
        })
        // 3. Activities
        .with_rule(CedarPolicyRule {
            effect_permit: true,
            principal_role: Some("TenantAdmin".into()),
            action: None,
            resource_type: Some("Activities".into()),
            require_tenant_match: true,
        })
        .with_rule(CedarPolicyRule {
            effect_permit: true,
            principal_role: Some("SalesManager".into()),
            action: None,
            resource_type: Some("Activities".into()),
            require_tenant_match: true,
        })
        .with_rule(CedarPolicyRule {
            effect_permit: true,
            principal_role: Some("SalesRep".into()),
            action: Some("activities.create".into()),
            resource_type: Some("Activities".into()),
            require_tenant_match: true,
        })
        .with_rule(CedarPolicyRule {
            effect_permit: true,
            principal_role: Some("SalesRep".into()),
            action: Some("activities.read".into()),
            resource_type: Some("Activities".into()),
            require_tenant_match: true,
        })
        .with_rule(CedarPolicyRule {
            effect_permit: true,
            principal_role: Some("SalesRep".into()),
            action: Some("activities.complete".into()),
            resource_type: Some("Activities".into()),
            require_tenant_match: true,
        })
}
