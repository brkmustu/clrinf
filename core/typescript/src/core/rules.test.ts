import { expect, it } from "bun:test";
import {
  parseContext,
  pipeRules,
  Result,
  ruleFailed,
  rulePassed,
  violationToErrorEnvelope,
  type Rule,
} from "./index";

interface OrderItem {
  id: string;
  name: string;
  price: number;
}

const nameMustNotBeEmpty: Rule<OrderItem> = (item) => {
  if (!item.name.trim()) {
    return ruleFailed("EMPTY_NAME", "Order item name cannot be empty");
  }
  return rulePassed();
};

const priceMustBePositive: Rule<OrderItem> = (item) => {
  if (item.price <= 0) {
    return ruleFailed("INVALID_PRICE", "Order item price must be positive");
  }
  return rulePassed();
};

const enterpriseTenantMaxLimit: Rule<OrderItem> = async (item, context) => {
  if (context?.tenant_id === "tenant-enterprise" && item.price > 10_000) {
    return ruleFailed("PRICE_LIMIT_EXCEEDED", "Enterprise limit is 10,000");
  }
  return rulePassed();
};

it("Result monad works with ok, err, and match", () => {
  const ok = Result.ok(42);
  expect(ok.isOk).toBe(true);
  expect(ok.isErr).toBe(false);
  expect(ok.map((x) => x * 2).unwrapOr(0)).toBe(84);

  const err = Result.err("something went wrong");
  expect(err.isOk).toBe(false);
  expect(err.isErr).toBe(true);
  expect(err.unwrapOr(100)).toBe(100);

  const matched = err.match({
    ok: (val) => `val: ${val}`,
    err: (msg) => `err: ${msg}`,
  });
  expect(matched).toBe("err: something went wrong");
});

it("pipeRules returns ok when all rules pass", async () => {
  const context = parseContext({
    tenant_id: "tenant-standard",
    correlation_id: "corr-1",
    causation_id: "cause-1",
  });

  const pipeline = pipeRules(nameMustNotBeEmpty, priceMustBePositive, enterpriseTenantMaxLimit);

  const validItem: OrderItem = { id: "1", name: "Laptop", price: 1500 };
  const result = await pipeline(validItem, context);

  expect(result.isOk).toBe(true);
  expect(result.isErr).toBe(false);
});

it("pipeRules short-circuits on first failure and prevents raw throw", async () => {
  let thirdRuleRan = false;
  const thirdRule: Rule<OrderItem> = () => {
    thirdRuleRan = true;
    return rulePassed();
  };

  const pipeline = pipeRules(nameMustNotBeEmpty, priceMustBePositive, thirdRule);

  const invalidItem: OrderItem = { id: "1", name: "", price: -10 };
  const result = await pipeline(invalidItem);

  expect(result.isOk).toBe(false);
  expect(result.isErr).toBe(true);
  if (result.isErr) {
    expect(result.error.errorCode).toBe("EMPTY_NAME");
    expect(result.error.message).toBe("Order item name cannot be empty");
  }

  // Ensure short-circuit prevented subsequent rules from running
  expect(thirdRuleRan).toBe(false);
});

it("pipeRules handles tenant-aware rules and produces ErrorEnvelope", async () => {
  const enterpriseContext = parseContext({
    tenant_id: "tenant-enterprise",
    correlation_id: "corr-1",
    causation_id: "cause-1",
  });

  const standardContext = parseContext({
    tenant_id: "tenant-standard",
    correlation_id: "corr-2",
    causation_id: "cause-2",
  });

  const pipeline = pipeRules(nameMustNotBeEmpty, priceMustBePositive, enterpriseTenantMaxLimit);

  const expensiveItem: OrderItem = { id: "1", name: "SuperServer", price: 25000 };

  // Fails for enterprise tenant
  const enterpriseResult = await pipeline(expensiveItem, enterpriseContext);
  expect(enterpriseResult.isErr).toBe(true);
  if (enterpriseResult.isErr) {
    expect(enterpriseResult.error.errorCode).toBe("PRICE_LIMIT_EXCEEDED");
    const envelope = violationToErrorEnvelope(enterpriseResult.error, enterpriseContext);
    expect(envelope.error_code).toBe("PRICE_LIMIT_EXCEEDED");
    expect(envelope.tenant_id).toBe("tenant-enterprise");
    expect(envelope.correlation_id).toBe("corr-1");
  }

  // Passes for standard tenant
  const standardResult = await pipeline(expensiveItem, standardContext);
  expect(standardResult.isOk).toBe(true);
});
