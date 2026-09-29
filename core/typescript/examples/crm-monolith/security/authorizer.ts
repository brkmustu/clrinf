import { MultiTenantCedarAuthorizer } from "../../../src/core/authz.js";

export function createCrmAuthorizer(): MultiTenantCedarAuthorizer {
  const authorizer = new MultiTenantCedarAuthorizer("PlatformAdmin");

  // 1. Contacts
  authorizer.addRule({ principal_role: "TenantAdmin", resource_type: "Contacts", effect_permit: true });
  authorizer.addRule({ principal_role: "SalesManager", resource_type: "Contacts", effect_permit: true });
  authorizer.addRule({ principal_role: "SalesRep", action: "contacts.create", resource_type: "Contacts", effect_permit: true });
  authorizer.addRule({ principal_role: "SalesRep", action: "contacts.read", resource_type: "Contacts", effect_permit: true });

  // 2. Deals
  authorizer.addRule({ principal_role: "TenantAdmin", resource_type: "Deals", effect_permit: true });
  authorizer.addRule({ principal_role: "SalesManager", resource_type: "Deals", effect_permit: true });
  authorizer.addRule({ principal_role: "SalesRep", action: "deals.create", resource_type: "Deals", effect_permit: true });
  authorizer.addRule({ principal_role: "SalesRep", action: "deals.read", resource_type: "Deals", effect_permit: true });
  authorizer.addRule({ principal_role: "SalesRep", action: "deals.update", resource_type: "Deals", effect_permit: true });
  authorizer.addRule({ principal_role: "SalesRep", action: "deals.stage_change", resource_type: "Deals", effect_permit: true });

  // 3. Activities
  authorizer.addRule({ principal_role: "TenantAdmin", resource_type: "Activities", effect_permit: true });
  authorizer.addRule({ principal_role: "SalesManager", resource_type: "Activities", effect_permit: true });
  authorizer.addRule({ principal_role: "SalesRep", action: "activities.create", resource_type: "Activities", effect_permit: true });
  authorizer.addRule({ principal_role: "SalesRep", action: "activities.read", resource_type: "Activities", effect_permit: true });
  authorizer.addRule({ principal_role: "SalesRep", action: "activities.complete", resource_type: "Activities", effect_permit: true });

  return authorizer;
}
