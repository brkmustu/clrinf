import { MultiTenantCedarAuthorizer } from "../../../src/core/authz.js";

export function createEcommerceAuthorizer(): MultiTenantCedarAuthorizer {
  const authorizer = new MultiTenantCedarAuthorizer("PlatformAdmin");

  // 1. StoreAdmin - Full access to all domains
  authorizer.addRule({ principal_role: "StoreAdmin", resource_type: "Catalog", effect_permit: true });
  authorizer.addRule({ principal_role: "StoreAdmin", resource_type: "Inventory", effect_permit: true });
  authorizer.addRule({ principal_role: "StoreAdmin", resource_type: "Orders", effect_permit: true });

  // 2. CatalogManager - Manage catalog & view inventory
  authorizer.addRule({ principal_role: "CatalogManager", resource_type: "Catalog", effect_permit: true });
  authorizer.addRule({ principal_role: "CatalogManager", action: "inventory.read", resource_type: "Inventory", effect_permit: true });

  // 3. Customer - Browse catalog, place orders, reserve/release inventory
  authorizer.addRule({ principal_role: "Customer", action: "catalog.read", resource_type: "Catalog", effect_permit: true });
  authorizer.addRule({ principal_role: "Customer", action: "inventory.reserve", resource_type: "Inventory", effect_permit: true });
  authorizer.addRule({ principal_role: "Customer", action: "inventory.release", resource_type: "Inventory", effect_permit: true });
  authorizer.addRule({ principal_role: "Customer", action: "inventory.commit", resource_type: "Inventory", effect_permit: true });
  authorizer.addRule({ principal_role: "Customer", action: "orders.create", resource_type: "Orders", effect_permit: true });
  authorizer.addRule({ principal_role: "Customer", action: "orders.read", resource_type: "Orders", effect_permit: true });
  authorizer.addRule({ principal_role: "Customer", action: "orders.cancel", resource_type: "Orders", effect_permit: true });

  return authorizer;
}
