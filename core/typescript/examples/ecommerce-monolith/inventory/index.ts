import { pipeRules, ruleFailed, rulePassed, type Rule } from "../../../src/core/rules.js";
import type { MultiTenantCedarAuthorizer, OperationClaim } from "../../../src/core/authz.js";
import type { Claims } from "../../../src/core/auth.js";

export interface StockItem {
  product_id: string;
  tenant_id: string;
  available_quantity: number;
  reserved_quantity: number;
}

export interface ReserveStockInput {
  tenant_id: string;
  product_id: string;
  quantity: number;
}

export class InMemoryInventoryRepository {
  private storage = new Map<string, StockItem>();

  async getByProductId(tenantId: string, productId: string): Promise<StockItem | null> {
    return this.storage.get(`${tenantId}:${productId}`) ?? null;
  }

  async save(item: StockItem): Promise<StockItem> {
    this.storage.set(`${item.tenant_id}:${item.product_id}`, item);
    return item;
  }
}

// ─── Business Rules ────────────────────────────────────────────────────────

export const ensurePositiveQuantityRule: Rule<ReserveStockInput> = (input) => {
  if (input.quantity <= 0) {
    return ruleFailed("INVALID_QUANTITY", `Requested quantity must be positive. Received: ${input.quantity}`);
  }
  return rulePassed();
};

export function createEnsureStockAvailabilityRule(repo: InMemoryInventoryRepository): Rule<ReserveStockInput> {
  return async (input) => {
    const item = await repo.getByProductId(input.tenant_id, input.product_id);
    if (!item || item.available_quantity < input.quantity) {
      const avail = item?.available_quantity ?? 0;
      return ruleFailed("STOCK_INSUFFICIENT", `Insufficient stock for product '${input.product_id}'. Available: ${avail}, Requested: ${input.quantity}`);
    }
    return rulePassed();
  };
}

// ─── Handlers ──────────────────────────────────────────────────────────────

export async function setStock(
  tenantId: string,
  productId: string,
  initialQuantity: number,
  repo: InMemoryInventoryRepository,
  authorizer: MultiTenantCedarAuthorizer,
  userClaims: Claims
): Promise<{ ok: true; value: StockItem } | { ok: false; error_code: string; message: string }> {
  const claim: OperationClaim = {
    resource: "Inventory",
    action: "inventory.reserve",
    resource_tenant_id: tenantId,
  };
  const decision = await authorizer.authorize(userClaims, [claim]);
  if (!decision.allowed) {
    return { ok: false, error_code: "ACCESS_DENIED", message: decision.reason };
  }

  const item: StockItem = {
    tenant_id: tenantId,
    product_id: productId,
    available_quantity: initialQuantity,
    reserved_quantity: 0,
  };
  const saved = await repo.save(item);
  return { ok: true, value: saved };
}

export async function reserveStock(
  input: ReserveStockInput,
  repo: InMemoryInventoryRepository,
  authorizer: MultiTenantCedarAuthorizer,
  userClaims: Claims
): Promise<{ ok: true; value: StockItem } | { ok: false; error_code: string; message: string }> {
  const claim: OperationClaim = {
    resource: "Inventory",
    action: "inventory.reserve",
    resource_tenant_id: input.tenant_id,
  };
  const decision = await authorizer.authorize(userClaims, [claim]);
  if (!decision.allowed) {
    return { ok: false, error_code: "ACCESS_DENIED", message: decision.reason };
  }

  const pipeline = pipeRules(
    ensurePositiveQuantityRule,
    createEnsureStockAvailabilityRule(repo)
  );
  const ruleResult = await pipeline(input);
  if (ruleResult.isErr) {
    return { ok: false, error_code: ruleResult.error.errorCode, message: ruleResult.error.message };
  }

  const current = (await repo.getByProductId(input.tenant_id, input.product_id))!;
  current.available_quantity -= input.quantity;
  current.reserved_quantity += input.quantity;
  const saved = await repo.save(current);
  return { ok: true, value: saved };
}

export async function releaseStock(
  input: ReserveStockInput,
  repo: InMemoryInventoryRepository,
  authorizer: MultiTenantCedarAuthorizer,
  userClaims: Claims
): Promise<{ ok: true; value: StockItem } | { ok: false; error_code: string; message: string }> {
  const claim: OperationClaim = {
    resource: "Inventory",
    action: "inventory.release",
    resource_tenant_id: input.tenant_id,
  };
  const decision = await authorizer.authorize(userClaims, [claim]);
  if (!decision.allowed) {
    return { ok: false, error_code: "ACCESS_DENIED", message: decision.reason };
  }

  const current = await repo.getByProductId(input.tenant_id, input.product_id);
  if (!current) {
    return { ok: false, error_code: "STOCK_NOT_FOUND", message: "Stock item not found" };
  }

  const releaseQty = Math.min(current.reserved_quantity, input.quantity);
  current.available_quantity += releaseQty;
  current.reserved_quantity -= releaseQty;
  const saved = await repo.save(current);
  return { ok: true, value: saved };
}

export async function commitStock(
  input: ReserveStockInput,
  repo: InMemoryInventoryRepository,
  authorizer: MultiTenantCedarAuthorizer,
  userClaims: Claims
): Promise<{ ok: true; value: StockItem } | { ok: false; error_code: string; message: string }> {
  const claim: OperationClaim = {
    resource: "Inventory",
    action: "inventory.commit",
    resource_tenant_id: input.tenant_id,
  };
  const decision = await authorizer.authorize(userClaims, [claim]);
  if (!decision.allowed) {
    return { ok: false, error_code: "ACCESS_DENIED", message: decision.reason };
  }

  const current = await repo.getByProductId(input.tenant_id, input.product_id);
  if (!current) {
    return { ok: false, error_code: "STOCK_NOT_FOUND", message: "Stock item not found" };
  }

  const commitQty = Math.min(current.reserved_quantity, input.quantity);
  current.reserved_quantity -= commitQty;
  const saved = await repo.save(current);
  return { ok: true, value: saved };
}
