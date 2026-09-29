import { expect, it } from "bun:test";
import { contextFromEvent, createEvent, parseEvent, parseError, parseContext } from "./index";

const context = { tenant_id: "tenant", correlation_id: "workflow", causation_id: "request" };
const event = createEvent(context, { source: "clrinf/documents", type: "DocumentApproved", data: { revision: 1 } });

it("propagates context without changing tenant or correlation", () => {
  expect(contextFromEvent(event)).toEqual({ ...context, causation_id: event.id });
  expect(parseContext(context)).toEqual(context);
});
it("rejects malformed timestamps and every non-object/non-JSON payload", () => {
  for (const time of ["bad", "2026-02-30T00:00:00Z", "2026-09-09", "2026-01-01T25:00:00Z"]) {
    expect(() => parseEvent({ ...event, time })).toThrow();
  }
  for (const data of [null, [], "text", { invalid: undefined }, { number: Infinity }, { date: new Date() }]) {
    expect(() => parseEvent({ ...event, data })).toThrow();
  }
  const cycle: Record<string, unknown> = {};
  cycle.self = cycle;
  expect(() => parseEvent({ ...event, data: cycle })).toThrow();
  expect(() => parseEvent({ ...event, is_compensation: "true" })).toThrow();
});
it("preserves JSON error details and extension flags", () => {
  expect(parseError({ ...context, error_code: "DENIED", message: "Denied", retryable: true, details: { reason: "policy" } }).details).toEqual({ reason: "policy" });
  expect(parseEvent({ ...event, is_error: false, is_compensation: true })).toMatchObject({ is_error: false, is_compensation: true });
  const input = { ...event, data: { nested: { value: 1 } } };
  const parsed = parseEvent(input);
  input.data.nested.value = 2;
  expect(parsed.data).toEqual({ nested: { value: 1 } });
});
it("core and server imports do not start listeners or connect", async () => {
  const proc = Bun.spawn([process.execPath, "-e", "await import('@clrinf/core'); await import('@clrinf/core/core'); await import('@clrinf/core/adapters'); await import('@clrinf/core/inspector'); await import('./src/server.ts');"], {
    cwd: new URL("../..", import.meta.url).pathname.replace(/^\/([a-zA-Z]:)/, "$1"), env: { ...Bun.env, PORT: "invalid", NATS_URL: "nats://127.0.0.1:1" },
    stdout: "pipe", stderr: "pipe",
  });
  expect(await proc.exited).toBe(0);
  expect(await new Response(proc.stderr).text()).toBe("");
});
