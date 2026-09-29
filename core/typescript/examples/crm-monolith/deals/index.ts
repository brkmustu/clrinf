import { pipeRules, ruleFailed, rulePassed, type Rule } from "../../../src/core/rules.js";
import type { MultiTenantCedarAuthorizer, OperationClaim } from "../../../src/core/authz.js";
import type { Claims } from "../../../src/core/auth.js";

export interface Deal {
  id: string;
  tenant_id: string;
  title: string;
  contact_id: string;
  amount: number;
  stage: string;
  probability: number;
}

export interface CreateDealInput {
  tenant_id: string;
  title: string;
  contact_id: string;
  amount: number;
}

export interface ChangeDealStageInput {
  tenant_id: string;
  deal_id: string;
  new_stage: string;
}

export class InMemoryDealRepository {
  private storage = new Map<string, Deal>();

  async getById(tenantId: string, id: string): Promise<Deal | null> {
    return this.storage.get(`${tenantId}:${id}`) ?? null;
  }

  async save(deal: Deal): Promise<Deal> {
    this.storage.set(`${deal.tenant_id}:${deal.id}`, deal);
    return deal;
  }
}

// ─── Functional Business Rules ─────────────────────────────────────────────

export const ensureDealAmountPositiveRule: Rule<CreateDealInput> = (input) => {
  if (input.amount <= 0) {
    return ruleFailed("INVALID_DEAL_AMOUNT", "Deal amount must be greater than zero.");
  }
  return rulePassed();
};

export function createStageProgressionRule(repo: InMemoryDealRepository): Rule<ChangeDealStageInput> {
  const validStages = new Set(["Prospect", "Qualified", "Proposal", "Negotiation", "ClosedWon", "ClosedLost"]);

  return async (input) => {
    if (!validStages.has(input.new_stage)) {
      return ruleFailed("UNKNOWN_DEAL_STAGE", `Stage '${input.new_stage}' is not a recognized CRM deal stage.`);
    }

    const deal = await repo.getById(input.tenant_id, input.deal_id);
    if (!deal) {
      return ruleFailed("DEAL_NOT_FOUND", `Deal '${input.deal_id}' not found in tenant '${input.tenant_id}'.`);
    }

    if (deal.stage === "ClosedWon" || deal.stage === "ClosedLost") {
      return ruleFailed("DEAL_ALREADY_CLOSED", `Deal is already finalized as '${deal.stage}' and cannot transition directly.`);
    }

    if (deal.stage === "Prospect" && input.new_stage === "ClosedWon") {
      return ruleFailed("STAGE_PROGRESSION_VIOLATION", "Cannot transition deal directly from 'Prospect' to 'ClosedWon' without a Proposal stage.");
    }

    return rulePassed();
  };
}

// ─── Handlers ──────────────────────────────────────────────────────────────

export async function createDeal(
  input: CreateDealInput,
  repo: InMemoryDealRepository,
  authorizer: MultiTenantCedarAuthorizer,
  userClaims: Claims
): Promise<{ ok: true; value: Deal } | { ok: false; error_code: string; message: string }> {
  const claim: OperationClaim = {
    resource: "Deals",
    action: "deals.create",
    resource_tenant_id: input.tenant_id,
  };
  const decision = await authorizer.authorize(userClaims, [claim]);
  if (!decision.allowed) {
    return { ok: false, error_code: "ACCESS_DENIED", message: decision.reason };
  }

  const pipeline = pipeRules(ensureDealAmountPositiveRule);
  const ruleResult = await pipeline(input);
  if (ruleResult.isErr) {
    return { ok: false, error_code: ruleResult.error.errorCode, message: ruleResult.error.message };
  }

  const deal: Deal = {
    id: Math.random().toString(36).substring(2, 10),
    tenant_id: input.tenant_id,
    title: input.title,
    contact_id: input.contact_id,
    amount: input.amount,
    stage: "Prospect",
    probability: 10,
  };

  const saved = await repo.save(deal);
  return { ok: true, value: saved };
}

export async function changeDealStage(
  input: ChangeDealStageInput,
  repo: InMemoryDealRepository,
  authorizer: MultiTenantCedarAuthorizer,
  userClaims: Claims
): Promise<{ ok: true; value: Deal } | { ok: false; error_code: string; message: string }> {
  const claim: OperationClaim = {
    resource: "Deals",
    action: "deals.stage_change",
    resource_tenant_id: input.tenant_id,
  };
  const decision = await authorizer.authorize(userClaims, [claim]);
  if (!decision.allowed) {
    return { ok: false, error_code: "ACCESS_DENIED", message: decision.reason };
  }

  const pipeline = pipeRules(createStageProgressionRule(repo));
  const ruleResult = await pipeline(input);
  if (ruleResult.isErr) {
    return { ok: false, error_code: ruleResult.error.errorCode, message: ruleResult.error.message };
  }

  const deal = (await repo.getById(input.tenant_id, input.deal_id))!;
  const probabilityMap: Record<string, number> = {
    Prospect: 10,
    Qualified: 30,
    Proposal: 60,
    Negotiation: 80,
    ClosedWon: 100,
    ClosedLost: 0,
  };

  const updated: Deal = {
    ...deal,
    stage: input.new_stage,
    probability: probabilityMap[input.new_stage] ?? deal.probability,
  };

  const saved = await repo.save(updated);
  return { ok: true, value: saved };
}

export async function getDeal(
  tenantId: string,
  dealId: string,
  repo: InMemoryDealRepository,
  authorizer: MultiTenantCedarAuthorizer,
  userClaims: Claims
): Promise<{ ok: true; value: Deal | null } | { ok: false; error_code: string; message: string }> {
  const claim: OperationClaim = {
    resource: "Deals",
    action: "deals.read",
    resource_tenant_id: tenantId,
  };
  const decision = await authorizer.authorize(userClaims, [claim]);
  if (!decision.allowed) {
    return { ok: false, error_code: "ACCESS_DENIED", message: decision.reason };
  }

  const deal = await repo.getById(tenantId, dealId);
  return { ok: true, value: deal };
}
