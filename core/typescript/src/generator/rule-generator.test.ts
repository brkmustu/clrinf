import { describe, expect, it } from "bun:test";
import { RuleGenerator } from "./rule-generator";
import { mkdtemp, rm } from "node:fs/promises";
import { tmpdir } from "node:os";
import { join } from "node:path";

describe("RuleGenerator", () => {
  const generator = new RuleGenerator();

  it("generates valid rule and test code templates", () => {
    const { ruleCode, testCode, fileBase } = generator.generateRuleTemplate({
      ruleName: "MaxDiscountRule",
      contextType: "DiscountContext",
      errorCode: "MAX_DISCOUNT_EXCEEDED",
      defaultMessage: "Discount cannot exceed 50%",
      targetDir: "/fake/dir",
    });

    expect(fileBase).toBe("max-discount-rule");
    expect(ruleCode).toContain("export const maxDiscountRule: Rule<DiscountContext>");
    expect(ruleCode).toContain("ruleFailed(");
    expect(ruleCode).toContain('"MAX_DISCOUNT_EXCEEDED"');
    expect(testCode).toContain("describe(\"MaxDiscountRule\"");
    expect(testCode).toContain("maxDiscountRule(entity)");
  });

  it("scaffolds rule files on disk", async () => {
    const tempDir = await mkdtemp(join(tmpdir(), "clrinf-rule-test-"));
    try {
      const result = await generator.scaffoldRule({
        ruleName: "CustomerCreditCheck",
        contextType: "CreditContext",
        errorCode: "INSUFFICIENT_CREDIT",
        defaultMessage: "Customer has insufficient credit balance",
        targetDir: tempDir,
      });

      expect(result.rulePath).toContain("customer-credit-check.rule.ts");
      expect(result.testPath).toContain("customer-credit-check.rule.test.ts");

      const ruleExists = await Bun.file(result.rulePath).exists();
      const testExists = await Bun.file(result.testPath).exists();

      expect(ruleExists).toBe(true);
      expect(testExists).toBe(true);
    } finally {
      await rm(tempDir, { recursive: true, force: true });
    }
  });
});
