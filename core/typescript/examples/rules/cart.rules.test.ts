import { describe, expect, it } from "bun:test";
import {
  checkCartNotEmpty,
  checkMaxItemsPerCart,
  checkCustomerCredit,
  validateCart,
  type Cart,
} from "./cart.rules";
import { violationToErrorEnvelope, type Context } from "../../src/core";

describe("Cart Business Rules", () => {
  const sampleContext: Context = {
    tenant_id: "tenant-ecommerce",
    correlation_id: "order-flow-1",
    causation_id: "user-req-1",
  };

  it("passes all rules for a valid cart within limits", async () => {
    const validCart: Cart = {
      id: "cart-123",
      customerCredit: 500,
      items: [
        { productId: "prod-1", quantity: 2, unitPrice: 50 },
        { productId: "prod-2", quantity: 1, unitPrice: 100 },
      ],
    };

    const result = await validateCart(validCart, sampleContext);
    expect(result.isOk).toBe(true);
    expect(result.isErr).toBe(false);
  });

  it("fails checkCartNotEmpty when cart has no items", async () => {
    const emptyCart: Cart = {
      id: "cart-empty",
      customerCredit: 500,
      items: [],
    };

    const result = await validateCart(emptyCart, sampleContext);
    expect(result.isOk).toBe(false);
    expect(result.isErr).toBe(true);
    if (result.isErr) {
      expect(result.error.errorCode).toBe("EMPTY_CART");
      const envelope = violationToErrorEnvelope(result.error, sampleContext);
      expect(envelope.error_code).toBe("EMPTY_CART");
      expect(envelope.tenant_id).toBe("tenant-ecommerce");
    }
  });

  it("fails checkMaxItemsPerCart when item count exceeds 100", async () => {
    const hugeCart: Cart = {
      id: "cart-huge",
      customerCredit: 100000,
      items: [{ productId: "prod-bulk", quantity: 150, unitPrice: 10 }],
    };

    const result = await validateCart(hugeCart, sampleContext);
    expect(result.isErr).toBe(true);
    if (result.isErr) {
      expect(result.error.errorCode).toBe("MAX_ITEMS_EXCEEDED");
      expect(result.error.details?.totalQuantity).toBe(150);
    }
  });

  it("fails checkCustomerCredit when total exceeds customer credit and attaches details", async () => {
    const expensiveCart: Cart = {
      id: "cart-expensive",
      customerCredit: 100,
      items: [{ productId: "laptop", quantity: 1, unitPrice: 1200 }],
    };

    const result = await validateCart(expensiveCart, sampleContext);
    expect(result.isErr).toBe(true);
    if (result.isErr) {
      expect(result.error.errorCode).toBe("INSUFFICIENT_CREDIT");
      expect(result.error.details?.total).toBe(1200);
      expect(result.error.details?.availableCredit).toBe(100);
      expect(result.error.details?.tenantId).toBe("tenant-ecommerce");
    }
  });
});
