import { expect, it, describe } from "bun:test";
import { ArchitecturalLinter } from "./arch-linter";

describe("ArchitecturalLinter", () => {
  const linter = new ArchitecturalLinter();

  it("flags raw throw in rule files (ARCH_TS_001)", () => {
    const badCode = `
      import { Rule } from "@clrinf/core";
      export const validateUser: Rule<User> = (user) => {
        if (!user.name) {
          throw new Error("Name is required");
        }
        return rulePassed();
      };
    `;
    const violations = linter.lintSource(badCode, "user.rules.ts");
    expect(violations.length).toBe(1);
    expect(violations[0]!.code).toBe("ARCH_TS_001");
    expect(violations[0]!.message).toContain("Raw 'throw' statement detected");
  });

  it("passes clean rules returning Result / ruleFailed (no ARCH_TS_001)", () => {
    const goodCode = `
      import { Rule, rulePassed, ruleFailed } from "@clrinf/core";
      export const validateUser: Rule<User> = (user) => {
        if (!user.name) {
          return ruleFailed("NAME_REQUIRED", "User name is required");
        }
        return rulePassed();
      };
    `;
    const violations = linter.lintSource(goodCode, "user.rules.ts");
    expect(violations.length).toBe(0);
  });

  it("flags class-based Mediator/Dispatcher (ARCH_TS_003)", () => {
    const mediatorCode = `
      export class CommandDispatcher {
        send(cmd: any) { /* reflection / locator */ }
      }
    `;
    const violations = linter.lintSource(mediatorCode, "dispatcher.ts");
    expect(violations.length).toBe(1);
    expect(violations[0]!.code).toBe("ARCH_TS_003");
    expect(violations[0]!.message).toContain("CommandDispatcher");
  });

  it("ignores non-rule/non-service files for throw statements", () => {
    const randomCode = `
      export function parseRawJson(str: string) {
        if (!str) throw new Error("Empty");
        return JSON.parse(str);
      }
    `;
    const violations = linter.lintSource(randomCode, "json-parser.ts");
    expect(violations.length).toBe(0);
  });
});
