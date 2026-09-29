import { type Rule, type Context, pipeRules, rulePassed, ruleFailed } from "../../src/core";

export interface CartItem {
  readonly productId: string;
  readonly quantity: number;
  readonly unitPrice: number;
}

export interface Cart {
  readonly id: string;
  readonly items: readonly CartItem[];
  readonly customerCredit: number;
}

/**
 * Rule: Cart must not be empty.
 */
export const checkCartNotEmpty: Rule<Cart> = (cart: Cart) => {
  if (!cart.items || cart.items.length === 0) {
    return ruleFailed("EMPTY_CART", "Cart must contain at least one item");
  }
  return rulePassed();
};

/**
 * Rule: Cart items quantity cannot exceed maximum limit (e.g. 100).
 */
export const checkMaxItemsPerCart: Rule<Cart> = (cart: Cart) => {
  const totalQuantity = cart.items.reduce((sum, item) => sum + item.quantity, 0);
  if (totalQuantity > 100) {
    return ruleFailed(
      "MAX_ITEMS_EXCEEDED",
      "Total cart items cannot exceed 100 units",
      { totalQuantity, maxAllowed: 100 }
    );
  }
  return rulePassed();
};

/**
 * Rule: Cart total price cannot exceed available customer credit.
 */
export const checkCustomerCredit: Rule<Cart> = (cart: Cart, requestContext?: Context) => {
  const total = cart.items.reduce(
    (sum, item) => sum + item.quantity * item.unitPrice,
    0
  );
  if (total > cart.customerCredit) {
    return ruleFailed(
      "INSUFFICIENT_CREDIT",
      "Cart total exceeds customer available credit",
      {
        cartId: cart.id,
        total,
        availableCredit: cart.customerCredit,
        ...(requestContext?.tenant_id ? { tenantId: requestContext.tenant_id } : {}),
      }
    );
  }
  return rulePassed();
};

/**
 * Composed sequential rule pipeline for Cart validation.
 * Short-circuits on first failure without throwing raw exceptions.
 */
export const validateCart = pipeRules<Cart>(
  checkCartNotEmpty,
  checkMaxItemsPerCart,
  checkCustomerCredit
);
