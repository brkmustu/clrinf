// Multi-tenant Ecommerce Cedar Security Policies

// StoreAdmin has full access to Catalog, Inventory, and Orders within tenant
permit (
    principal in Role::"StoreAdmin",
    action,
    resource
)
when {
    principal.tenant_id == resource.tenant_id
};

// CatalogManager has access to Catalog and inventory read within tenant
permit (
    principal in Role::"CatalogManager",
    action,
    resource in [Resource::"Catalog"]
)
when {
    principal.tenant_id == resource.tenant_id
};

permit (
    principal in Role::"CatalogManager",
    action == Action::"inventory.read",
    resource in [Resource::"Inventory"]
)
when {
    principal.tenant_id == resource.tenant_id
};

// Customer can read catalog, reserve/release stock, and manage orders within tenant
permit (
    principal in Role::"Customer",
    action == Action::"catalog.read",
    resource in [Resource::"Catalog"]
)
when {
    principal.tenant_id == resource.tenant_id
};

permit (
    principal in Role::"Customer",
    action in [Action::"inventory.reserve", Action::"inventory.release", Action::"inventory.commit"],
    resource in [Resource::"Inventory"]
)
when {
    principal.tenant_id == resource.tenant_id
};

permit (
    principal in Role::"Customer",
    action in [Action::"orders.create", Action::"orders.read", Action::"orders.cancel"],
    resource in [Resource::"Orders"]
)
when {
    principal.tenant_id == resource.tenant_id
};
