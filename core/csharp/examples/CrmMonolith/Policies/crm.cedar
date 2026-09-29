// Multi-tenant CRM Cedar Security Policies

// Admin has full access to all CRM resources within tenant
permit (
    principal in Role::"Admin",
    action,
    resource
)
when {
    principal.tenant_id == resource.tenant_id
};

// Users can read Deals, Contacts, and Activities within their tenant
permit (
    principal in Role::"User",
    action in [Action::"GetById", Action::"GetList"],
    resource in [Resource::"Deal", Resource::"Contact", Resource::"Activity"]
)
when {
    principal.tenant_id == resource.tenant_id
};

// Users can create Deals and Activities within their tenant
permit (
    principal in Role::"User",
    action in [Action::"Create", Action::"Update"],
    resource in [Resource::"Deal", Resource::"Activity"]
)
when {
    principal.tenant_id == resource.tenant_id
};
