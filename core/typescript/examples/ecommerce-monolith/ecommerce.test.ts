import { describe, expect, it } from "bun:test";
import { InMemoryCatalogRepository, createProduct, getProduct } from "./catalog/index.js";
import { InMemoryInventoryRepository, setStock, reserveStock, releaseStock } from "./inventory/index.js";
import { InMemoryOrderRepository, createOrder, cancelOrder, getOrder } from "./orders/index.js";
import { createEcommerceAuthorizer } from "./security/authorizer.js";
import type { Claims } from "../../src/core/auth.js";

describe("TypeScript Ecommerce Monolith End-to-End Tests", () => {
  const catalogRepo = new InMemoryCatalogRepository();
  const inventoryRepo = new InMemoryInventoryRepository();
  const orderRepo = new InMemoryOrderRepository();
  const authorizer = createEcommerceAuthorizer();

  const adminClaims: Claims = {
    sub: "admin_1",
    roles: ["StoreAdmin"],
    tenant_id: "tenant_acme",
    exp: 9999999999,
    iat: 1000000000,
  };

  const customerClaims: Claims = {
    sub: "cust_1",
    roles: ["Customer"],
    tenant_id: "tenant_acme",
    exp: 9999999999,
    iat: 1000000000,
  };

  it("creates product successfully with positive price", async () => {
    const res = await createProduct(
      {
        tenant_id: "tenant_acme",
        sku: "SKU-KEYBOARD",
        name: "Mechanical Keyboard",
        price: 120.0,
      },
      catalogRepo,
      authorizer,
      adminClaims
    );

    expect(res.ok).toBe(true);
    if (res.ok) {
      expect(res.value.sku).toBe("SKU-KEYBOARD");
      expect(res.value.price).toBe(120.0);
      expect(res.value.status).toBe("Active");
    }
  });

  it("fails to create product when price is zero or negative", async () => {
    const res = await createProduct(
      {
        tenant_id: "tenant_acme",
        sku: "SKU-FREE",
        name: "Free Item",
        price: 0,
      },
      catalogRepo,
      authorizer,
      adminClaims
    );

    expect(res.ok).toBe(false);
    if (!res.ok) {
      expect(res.error_code).toBe("INVALID_PRODUCT_PRICE");
    }
  });

  it("fails to create product when SKU is duplicate in same tenant", async () => {
    const res1 = await createProduct(
      {
        tenant_id: "tenant_acme",
        sku: "SKU-MOUSE",
        name: "Gaming Mouse",
        price: 50.0,
      },
      catalogRepo,
      authorizer,
      adminClaims
    );
    expect(res1.ok).toBe(true);

    const res2 = await createProduct(
      {
        tenant_id: "tenant_acme",
        sku: "SKU-MOUSE",
        name: "Office Mouse",
        price: 30.0,
      },
      catalogRepo,
      authorizer,
      adminClaims
    );

    expect(res2.ok).toBe(false);
    if (!res2.ok) {
      expect(res2.error_code).toBe("DUPLICATE_SKU");
    }
  });

  it("reserves stock successfully when available", async () => {
    await setStock("tenant_acme", "prod_1", 10, inventoryRepo, authorizer, adminClaims);

    const reserveRes = await reserveStock(
      { tenant_id: "tenant_acme", product_id: "prod_1", quantity: 3 },
      inventoryRepo,
      authorizer,
      customerClaims
    );

    expect(reserveRes.ok).toBe(true);
    if (reserveRes.ok) {
      expect(reserveRes.value.available_quantity).toBe(7);
      expect(reserveRes.value.reserved_quantity).toBe(3);
    }
  });

  it("fails to reserve stock when requested quantity exceeds available stock", async () => {
    await setStock("tenant_acme", "prod_2", 2, inventoryRepo, authorizer, adminClaims);

    const reserveRes = await reserveStock(
      { tenant_id: "tenant_acme", product_id: "prod_2", quantity: 5 },
      inventoryRepo,
      authorizer,
      customerClaims
    );

    expect(reserveRes.ok).toBe(false);
    if (!reserveRes.ok) {
      expect(reserveRes.error_code).toBe("STOCK_INSUFFICIENT");
    }
  });

  it("creates order successfully with valid items and total", async () => {
    const res = await createOrder(
      {
        tenant_id: "tenant_acme",
        customer_id: "cust_1",
        items: [{ product_id: "prod_1", quantity: 1, unit_price: 120.0 }],
      },
      orderRepo,
      authorizer,
      customerClaims
    );

    expect(res.ok).toBe(true);
    if (res.ok) {
      expect(res.value.status).toBe("Pending");
      expect(res.value.total_amount).toBe(120.0);
    }
  });

  it("fails to create order when total amount is below minimum threshold", async () => {
    const res = await createOrder(
      {
        tenant_id: "tenant_acme",
        customer_id: "cust_1",
        items: [{ product_id: "prod_sticker", quantity: 1, unit_price: 5.0 }],
      },
      orderRepo,
      authorizer,
      customerClaims
    );

    expect(res.ok).toBe(false);
    if (!res.ok) {
      expect(res.error_code).toBe("MINIMUM_ORDER_AMOUNT_NOT_MET");
    }
  });

  it("fails to create order when items list is empty", async () => {
    const res = await createOrder(
      {
        tenant_id: "tenant_acme",
        customer_id: "cust_1",
        items: [],
      },
      orderRepo,
      authorizer,
      customerClaims
    );

    expect(res.ok).toBe(false);
    if (!res.ok) {
      expect(res.error_code).toBe("EMPTY_ORDER_ITEMS");
    }
  });

  it("executes order cancellation and stock release compensating lifecycle", async () => {
    // 1. Set stock
    await setStock("tenant_acme", "prod_headset", 10, inventoryRepo, authorizer, adminClaims);

    // 2. Reserve 2 items
    const reserveRes = await reserveStock(
      { tenant_id: "tenant_acme", product_id: "prod_headset", quantity: 2 },
      inventoryRepo,
      authorizer,
      customerClaims
    );
    expect(reserveRes.ok).toBe(true);

    // 3. Create order
    const orderRes = await createOrder(
      {
        tenant_id: "tenant_acme",
        customer_id: "cust_1",
        items: [{ product_id: "prod_headset", quantity: 2, unit_price: 50.0 }],
      },
      orderRepo,
      authorizer,
      customerClaims
    );
    expect(orderRes.ok).toBe(true);
    if (!orderRes.ok) return;

    // 4. Cancel order
    const cancelRes = await cancelOrder("tenant_acme", orderRes.value.id, orderRepo, authorizer, customerClaims);
    expect(cancelRes.ok).toBe(true);
    if (cancelRes.ok) {
      expect(cancelRes.value.status).toBe("Cancelled");
    }

    // 5. Compensating action: Release reserved stock
    const releaseRes = await releaseStock(
      { tenant_id: "tenant_acme", product_id: "prod_headset", quantity: 2 },
      inventoryRepo,
      authorizer,
      customerClaims
    );
    expect(releaseRes.ok).toBe(true);
    if (releaseRes.ok) {
      expect(releaseRes.value.available_quantity).toBe(10);
      expect(releaseRes.value.reserved_quantity).toBe(0);
    }
  });

  it("enforces multi-tenant isolation via Cedar policy", async () => {
    const tenantBetaClaims: Claims = {
      sub: "user_beta",
      roles: ["Customer"],
      tenant_id: "tenant_beta",
      exp: 9999999999,
      iat: 1000000000,
    };

    // User in tenant_beta attempts to access tenant_acme order
    const res = await createOrder(
      {
        tenant_id: "tenant_acme",
        customer_id: "user_beta",
        items: [{ product_id: "p1", quantity: 1, unit_price: 50.0 }],
      },
      orderRepo,
      authorizer,
      tenantBetaClaims
    );

    expect(res.ok).toBe(false);
    if (!res.ok) {
      expect(res.error_code).toBe("ACCESS_DENIED");
    }
  });
});
