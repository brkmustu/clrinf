import { pipeRules, ruleFailed, rulePassed, type Rule } from "../../../src/core/rules.js";
import type { MultiTenantCedarAuthorizer, OperationClaim } from "../../../src/core/authz.js";
import type { Claims } from "../../../src/core/auth.js";

export interface OrderItem {
  product_id: string;
  quantity: number;
  unit_price: number;
}

export interface Order {
  id: string;
  tenant_id: string;
  customer_id: string;
  items: OrderItem[];
  total_amount: number;
  status: "Pending" | "Completed" | "Cancelled";
}

export interface CreateOrderInput {
  tenant_id: string;
  customer_id: string;
  items: OrderItem[];
}

export class InMemoryOrderRepository {
  private storage = new Map<string, Order>();

  async getById(tenantId: string, id: string): Promise<Order | null> {
    return this.storage.get(`${tenantId}:${id}`) ?? null;
  }

  async save(order: Order): Promise<Order> {
    this.storage.set(`${order.tenant_id}:${order.id}`, order);
    return order;
  }
}

// ─── Business Rules ────────────────────────────────────────────────────────

export const ensureOrderItemsNotEmptyRule: Rule<CreateOrderInput> = (input) => {
  if (!input.items || input.items.length === 0) {
    return ruleFailed("EMPTY_ORDER_ITEMS", "Order must contain at least one item.");
  }
  return rulePassed();
};

export const minimumOrderAmountRule: Rule<CreateOrderInput> = (input) => {
  const total = input.items.reduce((sum, item) => sum + item.quantity * item.unit_price, 0);
  if (total < 25.0) {
    return ruleFailed("MINIMUM_ORDER_AMOUNT_NOT_MET", `Order total $${total} is below minimum required threshold $25.00.`);
  }
  return rulePassed();
};

export function createCannotCancelCompletedOrderRule(repo: InMemoryOrderRepository): Rule<{ tenant_id: string; order_id: string }> {
  return async (input) => {
    const order = await repo.getById(input.tenant_id, input.order_id);
    if (order && order.status === "Completed") {
      return ruleFailed("INVALID_ORDER_STATE", `Cannot cancel completed order '${input.order_id}'.`);
    }
    return rulePassed();
  };
}

// ─── Handlers ──────────────────────────────────────────────────────────────

export async function createOrder(
  input: CreateOrderInput,
  repo: InMemoryOrderRepository,
  authorizer: MultiTenantCedarAuthorizer,
  userClaims: Claims
): Promise<{ ok: true; value: Order } | { ok: false; error_code: string; message: string }> {
  const claim: OperationClaim = {
    resource: "Orders",
    action: "orders.create",
    resource_tenant_id: input.tenant_id,
  };
  const decision = await authorizer.authorize(userClaims, [claim]);
  if (!decision.allowed) {
    return { ok: false, error_code: "ACCESS_DENIED", message: decision.reason };
  }

  const pipeline = pipeRules(
    ensureOrderItemsNotEmptyRule,
    minimumOrderAmountRule
  );
  const ruleResult = await pipeline(input);
  if (ruleResult.isErr) {
    return { ok: false, error_code: ruleResult.error.errorCode, message: ruleResult.error.message };
  }

  const total = input.items.reduce((sum, item) => sum + item.quantity * item.unit_price, 0);
  const order: Order = {
    id: Math.random().toString(36).substring(2, 10),
    tenant_id: input.tenant_id,
    customer_id: input.customer_id,
    items: input.items,
    total_amount: total,
    status: "Pending",
  };

  const saved = await repo.save(order);
  return { ok: true, value: saved };
}

export async function cancelOrder(
  tenantId: string,
  orderId: string,
  repo: InMemoryOrderRepository,
  authorizer: MultiTenantCedarAuthorizer,
  userClaims: Claims
): Promise<{ ok: true; value: Order } | { ok: false; error_code: string; message: string }> {
  const claim: OperationClaim = {
    resource: "Orders",
    action: "orders.cancel",
    resource_tenant_id: tenantId,
  };
  const decision = await authorizer.authorize(userClaims, [claim]);
  if (!decision.allowed) {
    return { ok: false, error_code: "ACCESS_DENIED", message: decision.reason };
  }

  const pipeline = pipeRules(createCannotCancelCompletedOrderRule(repo));
  const ruleResult = await pipeline({ tenant_id: tenantId, order_id: orderId });
  if (ruleResult.isErr) {
    return { ok: false, error_code: ruleResult.error.errorCode, message: ruleResult.error.message };
  }

  const current = await repo.getById(tenantId, orderId);
  if (!current) {
    return { ok: false, error_code: "ORDER_NOT_FOUND", message: "Order not found" };
  }

  current.status = "Cancelled";
  const saved = await repo.save(current);
  return { ok: true, value: saved };
}

export async function completeOrder(
  tenantId: string,
  orderId: string,
  repo: InMemoryOrderRepository,
  authorizer: MultiTenantCedarAuthorizer,
  userClaims: Claims
): Promise<{ ok: true; value: Order } | { ok: false; error_code: string; message: string }> {
  const claim: OperationClaim = {
    resource: "Orders",
    action: "orders.create",
    resource_tenant_id: tenantId,
  };
  const decision = await authorizer.authorize(userClaims, [claim]);
  if (!decision.allowed) {
    return { ok: false, error_code: "ACCESS_DENIED", message: decision.reason };
  }

  const current = await repo.getById(tenantId, orderId);
  if (!current) {
    return { ok: false, error_code: "ORDER_NOT_FOUND", message: "Order not found" };
  }

  current.status = "Completed";
  const saved = await repo.save(current);
  return { ok: true, value: saved };
}

export async function getOrder(
  tenantId: string,
  orderId: string,
  repo: InMemoryOrderRepository,
  authorizer: MultiTenantCedarAuthorizer,
  userClaims: Claims
): Promise<{ ok: true; value: Order | null } | { ok: false; error_code: string; message: string }> {
  const claim: OperationClaim = {
    resource: "Orders",
    action: "orders.read",
    resource_tenant_id: tenantId,
  };
  const decision = await authorizer.authorize(userClaims, [claim]);
  if (!decision.allowed) {
    return { ok: false, error_code: "ACCESS_DENIED", message: decision.reason };
  }

  const order = await repo.getById(tenantId, orderId);
  return { ok: true, value: order };
}
