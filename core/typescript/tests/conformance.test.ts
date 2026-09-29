import { describe, expect, it } from "bun:test";
import { resolve } from "node:path";
import { fileURLToPath } from "node:url";
import { object, parseContext, parseError, parseEvent, ValidationError } from "../src/core";

const directory = process.env.CLRINF_CONFORMANCE_DIR ?? fileURLToPath(new URL("../../../tests/conformance/fixtures/", import.meta.url));
async function fixture(name: string) {
  const path = resolve(directory, name);
  if (!await Bun.file(path).exists()) throw new Error(`Required conformance fixture missing: ${path}. Set CLRINF_CONFORMANCE_DIR to the shared fixtures directory.`);
  return object(await Bun.file(path).json());
}
const valid = await fixture("valid.json"), invalid = await fixture("invalid.json");
const parsers = { context: parseContext, error: parseError, event: parseEvent };

describe("shared cross-language conformance fixtures", () => {
  for (const kind of ["context", "error", "event"] as const) {
    it(`round-trips canonical ${kind}`, () => {
      expect(JSON.parse(JSON.stringify(parsers[kind](valid[kind])))).toEqual(valid[kind]);
    });
    const cases: unknown = invalid[kind];
    if (!Array.isArray(cases) || cases.length === 0) throw new Error(`Missing invalid fixture cases for ${kind}`);
    for (const candidate of cases) {
      const entry = object(candidate);
      if (typeof entry.name !== "string" || !("value" in entry)) throw new Error(`Malformed ${kind} fixture case`);
      it(`rejects ${kind}: ${entry.name}`, () => expect(() => parsers[kind](entry.value)).toThrow(ValidationError));
    }
  }
});
