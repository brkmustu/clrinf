import { describe, expect, it } from "bun:test";
import { MultiTenantCedarAuthorizer, type Claims, type OperationClaim } from "./index";

describe("MultiTenantCedarAuthorizer", () => {
  const authorizer = new MultiTenantCedarAuthorizer("PlatformAdmin")
    .addRule({
      effect_permit: true,
      principal_role: "PricingOfficer",
      action: "update_price",
      resource_type: "product",
    })
    .addRule({
      effect_permit: false, // Explicit Cedar Forbid
      principal_role: "Intern",
      action: "*",
      resource_type: "product",
    });

  const tenantAClaims: Claims = {
    sub: "user-1",
    tenant_id: "tenant-a",
    roles: ["PricingOfficer"],
    exp: 9999999999,
    iat: 1000000000,
  };

  it("permits operation when principal role and tenant match", async () => {
    const claim: OperationClaim = {
      resource: "product",
      action: "update_price",
      resource_tenant_id: "tenant-a",
    };

    const decision = await authorizer.authorize(tenantAClaims, [claim]);
    expect(decision.allowed).toBe(true);
  });

  it("strictly enforces tenant isolation by default (denies cross-tenant access)", async () => {
    const crossTenantClaim: OperationClaim = {
      resource: "product",
      action: "update_price",
      resource_tenant_id: "tenant-b", // Different tenant!
    };

    const decision = await authorizer.authorize(tenantAClaims, [crossTenantClaim]);
    expect(decision.allowed).toBe(false);
    if (!decision.allowed) {
      expect(decision.is_tenant_violation).toBe(true);
      expect(decision.reason).toContain("Tenant isolation violation");
    }
  });

  it("allows PlatformAdmin to bypass tenant isolation", async () => {
    const platformAdminClaims: Claims = {
      sub: "super-admin",
      tenant_id: "system",
      roles: ["PlatformAdmin"],
      exp: 9999999999,
      iat: 1000000000,
    };

    const crossTenantClaim: OperationClaim = {
      resource: "product",
      action: "update_price",
      resource_tenant_id: "tenant-b",
    };

    const decision = await authorizer.authorize(platformAdminClaims, [crossTenantClaim]);
    expect(decision.allowed).toBe(true);
  });

  it("enforces explicit Cedar Forbid override even if user has another permitting role", async () => {
    const internWithPricingClaims: Claims = {
      sub: "user-intern",
      tenant_id: "tenant-a",
      roles: ["PricingOfficer", "Intern"],
      exp: 9999999999,
      iat: 1000000000,
    };

    const claim: OperationClaim = {
      resource: "product",
      action: "update_price",
      resource_tenant_id: "tenant-a",
    };

    const decision = await authorizer.authorize(internWithPricingClaims, [claim]);
    expect(decision.allowed).toBe(false);
    if (!decision.allowed) {
      expect(decision.reason).toContain("Explicit Cedar FORBID triggered");
    }
  });

  it("allows TenantAdmin full access within their tenant", async () => {
    const tenantAdminClaims: Claims = {
      sub: "admin-1",
      tenant_id: "tenant-a",
      roles: ["TenantAdmin"],
      exp: 9999999999,
      iat: 1000000000,
    };

    const anyClaim: OperationClaim = {
      resource: "invoice",
      action: "delete",
      resource_tenant_id: "tenant-a",
    };

    const decision = await authorizer.authorize(tenantAdminClaims, [anyClaim]);
    expect(decision.allowed).toBe(true);
  });
});
