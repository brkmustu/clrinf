import { describe, expect, it } from "bun:test";
import { InMemoryContactRepository, createContact, getContact } from "./contacts/index.js";
import { InMemoryDealRepository, createDeal, changeDealStage, getDeal } from "./deals/index.js";
import { InMemoryActivityRepository, logActivity, completeActivity } from "./activities/index.js";
import { createCrmAuthorizer } from "./security/authorizer.js";
import type { Claims } from "../../src/core/auth.js";

describe("TypeScript CRM Monolith End-to-End Tests", () => {
  const contactRepo = new InMemoryContactRepository();
  const dealRepo = new InMemoryDealRepository();
  const activityRepo = new InMemoryActivityRepository();
  const authorizer = createCrmAuthorizer();

  const userClaims: Claims = {
    sub: "user_sales_1",
    roles: ["SalesManager"],
    tenant_id: "tenant_acme",
    exp: 9999999999,
    iat: 1000000000,
  };

  it("creates contact successfully with valid email", async () => {
    const res = await createContact(
      {
        tenant_id: "tenant_acme",
        first_name: "Bruce",
        last_name: "Wayne",
        email: "bruce@waynecorp.com",
        company: "Wayne Enterprises",
      },
      contactRepo,
      authorizer,
      userClaims
    );

    expect(res.ok).toBe(true);
    if (res.ok) {
      expect(res.value.first_name).toBe("Bruce");
      expect(res.value.tenant_id).toBe("tenant_acme");
      expect(res.value.status).toBe("Lead");
    }
  });

  it("fails to create contact when email format is invalid", async () => {
    const res = await createContact(
      {
        tenant_id: "tenant_acme",
        first_name: "Invalid",
        last_name: "User",
        email: "not-an-email",
        company: "Acme",
      },
      contactRepo,
      authorizer,
      userClaims
    );

    expect(res.ok).toBe(false);
    if (!res.ok) {
      expect(res.error_code).toBe("INVALID_EMAIL_FORMAT");
    }
  });

  it("fails to create contact when email is duplicate in same tenant", async () => {
    // Second insertion with identical email
    const res = await createContact(
      {
        tenant_id: "tenant_acme",
        first_name: "Bruce",
        last_name: "Duplicate",
        email: "bruce@waynecorp.com",
        company: "Wayne Enterprises",
      },
      contactRepo,
      authorizer,
      userClaims
    );

    expect(res.ok).toBe(false);
    if (!res.ok) {
      expect(res.error_code).toBe("DUPLICATE_CONTACT_EMAIL");
    }
  });

  it("enforces deal stage progression rule (Prospect cannot jump directly to ClosedWon)", async () => {
    // 1. Create deal
    const createRes = await createDeal(
      {
        tenant_id: "tenant_acme",
        title: "Batmobile Upgrade",
        contact_id: "c_1",
        amount: 250000,
      },
      dealRepo,
      authorizer,
      userClaims
    );

    expect(createRes.ok).toBe(true);
    if (!createRes.ok) return;

    const dealId = createRes.value.id;

    // 2. Direct jump to ClosedWon must fail
    const jumpRes = await changeDealStage(
      {
        tenant_id: "tenant_acme",
        deal_id: dealId,
        new_stage: "ClosedWon",
      },
      dealRepo,
      authorizer,
      userClaims
    );

    expect(jumpRes.ok).toBe(false);
    if (!jumpRes.ok) {
      expect(jumpRes.error_code).toBe("STAGE_PROGRESSION_VIOLATION");
    }

    // 3. Normal progression: Proposal -> ClosedWon succeeds
    const proposalRes = await changeDealStage(
      {
        tenant_id: "tenant_acme",
        deal_id: dealId,
        new_stage: "Proposal",
      },
      dealRepo,
      authorizer,
      userClaims
    );
    expect(proposalRes.ok).toBe(true);

    const wonRes = await changeDealStage(
      {
        tenant_id: "tenant_acme",
        deal_id: dealId,
        new_stage: "ClosedWon",
      },
      dealRepo,
      authorizer,
      userClaims
    );
    expect(wonRes.ok).toBe(true);
    if (wonRes.ok) {
      expect(wonRes.value.stage).toBe("ClosedWon");
      expect(wonRes.value.probability).toBe(100);
    }
  });

  it("prevents logging activity on ClosedLost deal", async () => {
    // 1. Create and close as lost
    const createRes = await createDeal(
      {
        tenant_id: "tenant_acme",
        title: "Lost Deal",
        contact_id: "c_2",
        amount: 10000,
      },
      dealRepo,
      authorizer,
      userClaims
    );
    expect(createRes.ok).toBe(true);
    if (!createRes.ok) return;

    await changeDealStage(
      {
        tenant_id: "tenant_acme",
        deal_id: createRes.value.id,
        new_stage: "ClosedLost",
      },
      dealRepo,
      authorizer,
      userClaims
    );

    // 2. Logging activity must be blocked
    const actRes = await logActivity(
      {
        tenant_id: "tenant_acme",
        deal_id: createRes.value.id,
        type: "Call",
        subject: "Post-mortem",
        notes: "Trying to understand lost reason",
      },
      activityRepo,
      dealRepo,
      authorizer,
      userClaims
    );

    expect(actRes.ok).toBe(false);
    if (!actRes.ok) {
      expect(actRes.error_code).toBe("CANNOT_LOG_ON_LOST_DEAL");
    }
  });

  it("strictly enforces Cedar multi-tenant forbid guard on cross-tenant access", async () => {
    // User claims tenant_acme, resource is tenant_stark
    const crossTenantRes = await createContact(
      {
        tenant_id: "tenant_stark",
        first_name: "Tony",
        last_name: "Stark",
        email: "tony@stark.com",
        company: "Stark Industries",
      },
      contactRepo,
      authorizer,
      userClaims
    );

    expect(crossTenantRes.ok).toBe(false);
    if (!crossTenantRes.ok) {
      expect(crossTenantRes.error_code).toBe("ACCESS_DENIED");
      expect(crossTenantRes.message).toContain("Tenant isolation violation");
    }
  });

  it("allows PlatformAdmin to access any tenant", async () => {
    const platformAdminClaims: Claims = {
      sub: "superadmin",
      roles: ["PlatformAdmin"],
      tenant_id: "system",
      exp: 9999999999,
      iat: 1000000000,
    };

    const res = await createContact(
      {
        tenant_id: "tenant_stark",
        first_name: "Pepper",
        last_name: "Potts",
        email: "pepper@stark.com",
        company: "Stark Industries",
      },
      contactRepo,
      authorizer,
      platformAdminClaims
    );

    expect(res.ok).toBe(true);
    if (res.ok) {
      expect(res.value.tenant_id).toBe("tenant_stark");
    }
  });

  it("benchmark: executes 10,000 functional pipeline operations rapidly", async () => {
    const createRes = await createDeal(
      {
        tenant_id: "tenant_acme",
        title: "Fast Deal",
        contact_id: "c_bench",
        amount: 500,
      },
      dealRepo,
      authorizer,
      userClaims
    );
    expect(createRes.ok).toBe(true);
    if (!createRes.ok) return;

    const dealId = createRes.value.id;

    const start = performance.now();
    for (let i = 0; i < 10000; i++) {
      const res = await getDeal("tenant_acme", dealId, dealRepo, authorizer, userClaims);
      expect(res.ok).toBe(true);
    }
    const elapsedMs = performance.now() - start;

    expect(elapsedMs).toBeLessThan(1000); // 10,000 pipeline executions in < 1000ms
  });
});
