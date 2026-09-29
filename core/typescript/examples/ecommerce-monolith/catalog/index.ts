import { pipeRules, ruleFailed, rulePassed, type Rule } from "../../../src/core/rules.js";
import type { MultiTenantCedarAuthorizer, OperationClaim } from "../../../src/core/authz.js";
import type { Claims } from "../../../src/core/auth.js";

export interface Product {
  id: string;
  tenant_id: string;
  sku: string;
  name: string;
  price: number;
  status: string;
}

export interface CreateProductInput {
  tenant_id: string;
  sku: string;
  name: string;
  price: number;
}

export class InMemoryCatalogRepository {
  private storage = new Map<string, Product>();

  async getById(tenantId: string, id: string): Promise<Product | null> {
    return this.storage.get(`${tenantId}:${id}`) ?? null;
  }

  async getBySku(tenantId: string, sku: string): Promise<Product | null> {
    for (const product of this.storage.values()) {
      if (product.tenant_id === tenantId && product.sku.toLowerCase() === sku.toLowerCase()) {
        return product;
      }
    }
    return null;
  }

  async save(product: Product): Promise<Product> {
    this.storage.set(`${product.tenant_id}:${product.id}`, product);
    return product;
  }
}

// ─── Functional Business Rules (ARCH_TS_001 Compliant: No raw throws) ──────

export const ensureProductPricePositiveRule: Rule<CreateProductInput> = (input) => {
  if (input.price <= 0) {
    return ruleFailed("INVALID_PRODUCT_PRICE", `Product price must be greater than zero. Received: ${input.price}`);
  }
  return rulePassed();
};

export function createEnsureSkuUniqueRule(repo: InMemoryCatalogRepository): Rule<CreateProductInput> {
  return async (input) => {
    const existing = await repo.getBySku(input.tenant_id, input.sku);
    if (existing) {
      return ruleFailed("DUPLICATE_SKU", `Product with SKU '${input.sku}' already exists in tenant '${input.tenant_id}'.`);
    }
    return rulePassed();
  };
}

// ─── Functional Pipeline & Handlers ────────────────────────────────────────

export async function createProduct(
  input: CreateProductInput,
  repo: InMemoryCatalogRepository,
  authorizer: MultiTenantCedarAuthorizer,
  userClaims: Claims
): Promise<{ ok: true; value: Product } | { ok: false; error_code: string; message: string }> {
  // 1. Cedar Authorization Check
  const claim: OperationClaim = {
    resource: "Catalog",
    action: "catalog.create",
    resource_tenant_id: input.tenant_id,
  };
  const decision = await authorizer.authorize(userClaims, [claim]);
  if (!decision.allowed) {
    return { ok: false, error_code: "ACCESS_DENIED", message: decision.reason };
  }

  // 2. Functional Business Rule Pipeline
  const pipeline = pipeRules(
    ensureProductPricePositiveRule,
    createEnsureSkuUniqueRule(repo)
  );
  const ruleResult = await pipeline(input);
  if (ruleResult.isErr) {
    return { ok: false, error_code: ruleResult.error.errorCode, message: ruleResult.error.message };
  }

  // 3. Persistence
  const product: Product = {
    id: Math.random().toString(36).substring(2, 10),
    tenant_id: input.tenant_id,
    sku: input.sku,
    name: input.name,
    price: input.price,
    status: "Active",
  };

  const saved = await repo.save(product);
  return { ok: true, value: saved };
}

export async function getProduct(
  tenantId: string,
  productId: string,
  repo: InMemoryCatalogRepository,
  authorizer: MultiTenantCedarAuthorizer,
  userClaims: Claims
): Promise<{ ok: true; value: Product | null } | { ok: false; error_code: string; message: string }> {
  const claim: OperationClaim = {
    resource: "Catalog",
    action: "catalog.read",
    resource_tenant_id: tenantId,
  };
  const decision = await authorizer.authorize(userClaims, [claim]);
  if (!decision.allowed) {
    return { ok: false, error_code: "ACCESS_DENIED", message: decision.reason };
  }

  const product = await repo.getById(tenantId, productId);
  return { ok: true, value: product };
}
