import { mkdir, writeFile } from "node:fs/promises";
import { join } from "node:path";

export interface RuleGeneratorOptions {
  /** The name of the rule, e.g. 'CheckMaxDiscount' or 'maxDiscountRule' */
  ruleName: string;
  /** Directory where rule and test files will be generated */
  targetDir: string;
  /** The TypeScript interface/type name for context, e.g. 'CartContext' */
  contextType: string;
  /** Canonical error code for violation, e.g. 'MAX_DISCOUNT_EXCEEDED' */
  errorCode: string;
  /** Default error message */
  defaultMessage: string;
  /** Import path for @clrinf/core (defaults to '@clrinf/core') */
  coreImportPath?: string;
}

export interface GeneratedRuleResult {
  rulePath: string;
  testPath: string;
  ruleCode: string;
  testCode: string;
}

function toKebabCase(str: string): string {
  return str
    .replace(/([a-z0-9]|(?=[A-Z]))([A-Z])/g, "$1-$2")
    .toLowerCase()
    .replace(/^-+|-+$/g, "");
}

function toCamelCase(str: string): string {
  const s = str.replace(/[-_]([a-z])/g, (_, c) => c.toUpperCase());
  return s.charAt(0).toLowerCase() + s.slice(1);
}

export class RuleGenerator {
  generateRuleTemplate(options: RuleGeneratorOptions): { ruleCode: string; testCode: string; fileBase: string } {
    const corePkg = options.coreImportPath ?? "@clrinf/core";
    const funcName = toCamelCase(options.ruleName);
    const fileBase = toKebabCase(options.ruleName);

    const ruleCode = `import { type Rule, type Context, rulePassed, ruleFailed } from "${corePkg}";

export interface ${options.contextType} {
  readonly id: string;
  readonly [key: string]: unknown;
}

/**
 * Business Rule: ${options.ruleName}
 * Code: ${options.errorCode}
 *
 * Enforces business boundaries without throwing raw exceptions.
 */
export const ${funcName}: Rule<${options.contextType}> = (
  entity: ${options.contextType},
  requestContext?: Context
) => {
  // TODO: Add entity validation condition here
  const isValid = Boolean(entity.id);

  if (!isValid) {
    return ruleFailed(
      "${options.errorCode}",
      "${options.defaultMessage}",
      { entityId: entity.id }
    );
  }

  return rulePassed();
};
`;

    const testCode = `import { describe, expect, it } from "bun:test";
import { ${funcName}, type ${options.contextType} } from "./${fileBase}.rule";

describe("${options.ruleName}", () => {
  it("passes when condition is met", () => {
    const entity: ${options.contextType} = { id: "valid-id" };
    const result = ${funcName}(entity);

    expect(result.isOk).toBe(true);
    expect(result.isErr).toBe(false);
  });

  it("fails with ${options.errorCode} when condition is violated", () => {
    const entity: ${options.contextType} = { id: "" };
    const result = ${funcName}(entity);

    expect(result.isOk).toBe(false);
    expect(result.isErr).toBe(true);
    if (result.isErr) {
      expect(result.error.errorCode).toBe("${options.errorCode}");
      expect(result.error.message).toBe("${options.defaultMessage}");
    }
  });
});
`;

    return { ruleCode, testCode, fileBase };
  }

  async scaffoldRule(options: RuleGeneratorOptions): Promise<GeneratedRuleResult> {
    await mkdir(options.targetDir, { recursive: true });
    const { ruleCode, testCode, fileBase } = this.generateRuleTemplate(options);

    const rulePath = join(options.targetDir, `${fileBase}.rule.ts`);
    const testPath = join(options.targetDir, `${fileBase}.rule.test.ts`);

    await writeFile(rulePath, ruleCode, "utf-8");
    await writeFile(testPath, testCode, "utf-8");

    return {
      rulePath,
      testPath,
      ruleCode,
      testCode,
    };
  }
}

if (import.meta.main) {
  const args = process.argv.slice(2);
  const ruleName = args[0] || "SampleRule";
  const targetDir = args[1] || "src/rules";
  const contextType = args[2] || "EntityContext";
  const errorCode = args[3] || `${ruleName.toUpperCase()}_FAILED`;
  const defaultMessage = args[4] || `Validation failed for ${ruleName}`;

  const generator = new RuleGenerator();
  const res = await generator.scaffoldRule({
    ruleName,
    targetDir,
    contextType,
    errorCode,
    defaultMessage,
  });
  console.log(JSON.stringify(res));
}
