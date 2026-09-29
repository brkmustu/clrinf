import { pipeRules, ruleFailed, rulePassed, type Rule } from "../../../src/core/rules.js";
import type { MultiTenantCedarAuthorizer, OperationClaim } from "../../../src/core/authz.js";
import type { Claims } from "../../../src/core/auth.js";
import type { InMemoryDealRepository } from "../deals/index.js";

export interface Activity {
  id: string;
  tenant_id: string;
  deal_id?: string | undefined;
  contact_id?: string | undefined;
  type: string;
  subject: string;
  notes: string;
  is_completed: boolean;
}

export interface LogActivityInput {
  tenant_id: string;
  deal_id?: string | undefined;
  contact_id?: string | undefined;
  type: string;
  subject: string;
  notes: string;
}

export interface CompleteActivityInput {
  tenant_id: string;
  activity_id: string;
}

export class InMemoryActivityRepository {
  private storage = new Map<string, Activity>();

  async getById(tenantId: string, id: string): Promise<Activity | null> {
    return this.storage.get(`${tenantId}:${id}`) ?? null;
  }

  async save(activity: Activity): Promise<Activity> {
    this.storage.set(`${activity.tenant_id}:${activity.id}`, activity);
    return activity;
  }
}

// ─── Functional Business Rules ─────────────────────────────────────────────

export function createCannotLogActivityOnLostDealRule(dealRepo: InMemoryDealRepository): Rule<LogActivityInput> {
  return async (input) => {
    if (!input.deal_id) {
      return rulePassed();
    }

    const deal = await dealRepo.getById(input.tenant_id, input.deal_id);
    if (deal && deal.stage === "ClosedLost") {
      return ruleFailed("CANNOT_LOG_ON_LOST_DEAL", `Cannot log new activity on deal '${input.deal_id}' because it is marked as 'ClosedLost'.`);
    }

    return rulePassed();
  };
}

// ─── Handlers ──────────────────────────────────────────────────────────────

export async function logActivity(
  input: LogActivityInput,
  repo: InMemoryActivityRepository,
  dealRepo: InMemoryDealRepository,
  authorizer: MultiTenantCedarAuthorizer,
  userClaims: Claims
): Promise<{ ok: true; value: Activity } | { ok: false; error_code: string; message: string }> {
  const claim: OperationClaim = {
    resource: "Activities",
    action: "activities.create",
    resource_tenant_id: input.tenant_id,
  };
  const decision = await authorizer.authorize(userClaims, [claim]);
  if (!decision.allowed) {
    return { ok: false, error_code: "ACCESS_DENIED", message: decision.reason };
  }

  const pipeline = pipeRules(createCannotLogActivityOnLostDealRule(dealRepo));
  const ruleResult = await pipeline(input);
  if (ruleResult.isErr) {
    return { ok: false, error_code: ruleResult.error.errorCode, message: ruleResult.error.message };
  }

  const activity: Activity = {
    id: Math.random().toString(36).substring(2, 10),
    tenant_id: input.tenant_id,
    deal_id: input.deal_id,
    contact_id: input.contact_id,
    type: input.type,
    subject: input.subject,
    notes: input.notes,
    is_completed: false,
  };

  const saved = await repo.save(activity);
  return { ok: true, value: saved };
}

export async function completeActivity(
  input: CompleteActivityInput,
  repo: InMemoryActivityRepository,
  authorizer: MultiTenantCedarAuthorizer,
  userClaims: Claims
): Promise<{ ok: true; value: Activity } | { ok: false; error_code: string; message: string }> {
  const claim: OperationClaim = {
    resource: "Activities",
    action: "activities.complete",
    resource_tenant_id: input.tenant_id,
  };
  const decision = await authorizer.authorize(userClaims, [claim]);
  if (!decision.allowed) {
    return { ok: false, error_code: "ACCESS_DENIED", message: decision.reason };
  }

  const activity = await repo.getById(input.tenant_id, input.activity_id);
  if (!activity) {
    return { ok: false, error_code: "NOT_FOUND", message: "Activity not found" };
  }

  const updated: Activity = { ...activity, is_completed: true };
  const saved = await repo.save(updated);
  return { ok: true, value: saved };
}
