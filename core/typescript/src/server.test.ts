import { describe, it, expect } from "bun:test";
import { createEvent } from "./core";
import { startInspector, InspectorEngine, waterfall } from "./inspector";
import { Script } from "node:vm";

const event = (id = "event-1") => createEvent({
  tenant_id: "tenant-1", correlation_id: "__proto__", causation_id: "request-1",
}, { id, source: "clrinf/documents", type: "DocumentReleased", data: { document_id: "doc-1" } });

describe("Inspector engine", () => {
  it("uses explicit independent error and compensation flags, not naming heuristics", () => {
    const engine = new InspectorEngine();
    expect(engine.recordEvent(event()).is_error).toBe(false);
    expect(engine.recordEvent({ ...event(), type: "ErrorFailed", is_compensation: true }).is_error).toBe(false);
    expect(engine.recordEvent({ ...event(), is_error: true }).is_error).toBe(true);
    expect(engine.recordEvent({ ...event(), is_compensation: true }).is_compensation).toBe(true);
  });
  it("validates, bounds history and isolates returned objects", () => {
    const engine = new InspectorEngine(2);
    expect(() => engine.recordEvent({ id: "bad" })).toThrow();
    engine.recordEvent(event("one"));
    engine.recordEvent(event("two"));
    const recorded = engine.recordEvent(event("three"));
    recorded.data.document_id = "mutated";
    expect(engine.list().map(e => e.id)).toEqual(["two", "three"]);
    expect(engine.list()[1]?.data.document_id).toBe("doc-1");
    engine.list().pop();
    expect(engine.size).toBe(2);
  });
  it("preserves replay causation and keeps tenant traces separate", () => {
    const engine = new InspectorEngine();
    const original = engine.recordEvent(event());
    const replayed = engine.replay(original);
    expect(replayed.causationid).toBe(original.id);
    expect(replayed.correlationid).toBe(original.correlationid);
    expect(replayed.id).not.toBe(original.id);
    engine.recordEvent({ ...event(), tenantid: "tenant-2" });
    expect(waterfall(engine.list())).toHaveLength(2);
  });
  it("bounds serialized history and rejects oversized records", () => {
    const engine = new InspectorEngine();
    for (let i = 0; i < 10; i++) engine.recordEvent({ ...event(String(i)), data: { text: "x".repeat(200 * 1024) } });
    expect(engine.size).toBe(5);
    expect(() => engine.recordEvent({ ...event(), data: { text: "x".repeat(256 * 1024) } })).toThrow("exceeds");
    engine.clear();
    engine.recordEvent(event());
    expect(engine.size).toBe(1);
  });
});

describe("Inspector HTTP and WebSocket", () => {
  it("validates ingress, preserves API compatibility, replays locally and clears history", async () => {
    const app = await startInspector({ port: 0 });
    const base = app.server.url;
    const request = (path: string, body: unknown) => fetch(new URL(path, base), {
      method: "POST", headers: { "Content-Type": "application/json" }, body: JSON.stringify(body),
    });
    try {
      expect((await (await fetch(new URL("/health", base))).json()).nats).toBe("disabled");
      expect((await request("/api/events", { id: "invalid" })).status).toBe(400);
      expect((await request("/api/events", null)).status).toBe(400);
      expect((await fetch(new URL("/api/events", base), { method: "POST", body: "{" })).status).toBe(400);
      expect((await request("/api/events", event())).status).toBe(201);
      const replay = await request("/api/replay", { event_id: "event-1" });
      const replayed = await replay.json();
      expect(replayed.destination).toBe("inspector-only");
      expect(replayed.event.causationid).toBe("event-1");
      expect((await request("/api/replay", { event: event(), publish: true })).status).toBe(400);
      expect((await request("/api/replay", { event: { id: "bad" } })).status).toBe(400);
      expect((await request("/api/replay", { event_id: "absent" })).status).toBe(404);
      expect((await request("/api/replay", null)).status).toBe(400);
      expect((await request("/api/events", { ...event(), data: { text: "x".repeat(300 * 1024) } })).status).toBe(413);
      await request("/api/events", { ...event(), tenantid: "tenant-other" });
      expect((await request("/api/replay", { event_id: "event-1" })).status).toBe(409);
      expect((await request("/api/replay", { event_id: "event-1", tenant_id: "tenant-other" })).status).toBe(200);
      const simulation = await (await request("/api/simulate", {})).json();
      const simulated = await (await fetch(new URL(`/api/events?correlation_id=${simulation.correlation_id}`, base))).json();
      expect(simulated.map((e: { type: string }) => e.type)).toContain("com.clrinf.document.DocumentApproved.v1");
      expect((await fetch(new URL("/", base))).status).toBe(200);
      expect((await fetch(new URL("/api/events", base), { method: "DELETE" })).status).toBe(204);
      expect(app.engine.size).toBe(0);
    } finally { await app.stop(); }
  });
  it("does not start HTTP when configured NATS cannot connect", async () => {
    await expect(startInspector({ port: 0, natsUrl: "nats://127.0.0.1:1" })).rejects.toThrow();
  });
  it("authenticates data endpoints and rejects cross-origin writes", async () => {
    await expect(startInspector({ hostname: "0.0.0.0", port: 0 })).rejects.toThrow("Remote Inspector");
    const app = await startInspector({ port: 0, token: "test-secret" });
    try {
      expect((await fetch(new URL("/api/events", app.server.url))).status).toBe(401);
      expect((await fetch(new URL("/api/events", app.server.url), { headers: { Authorization: "Bearer test-secret" } })).status).toBe(200);
      expect((await fetch(new URL("/api/events", app.server.url), {
        headers: { Authorization: `Basic ${Buffer.from("inspector:test-secret").toString("base64")}` },
      })).status).toBe(200);
      expect((await fetch(new URL("/api/events", app.server.url), {
        method: "DELETE", headers: { Authorization: "Bearer test-secret", Origin: "https://untrusted.example" },
      })).status).toBe(403);
    } finally { await app.stop(); }
  });
  it("streams a bounded snapshot and new events over a real WebSocket", async () => {
    const app = await startInspector({ port: 0, capacity: 1 });
    const socket = new WebSocket(new URL("/ws", app.server.url).href.replace("http:", "ws:"));
    const messages: { type: string; payload?: unknown }[] = [];
    try {
      await new Promise<void>((resolve, reject) => {
        const timeout = setTimeout(() => reject(new Error("WebSocket init timeout")), 2000);
        socket.onmessage = message => {
          messages.push(JSON.parse(String(message.data)));
          clearTimeout(timeout);
          resolve();
        };
        socket.onerror = () => { clearTimeout(timeout); reject(new Error("WebSocket failed")); };
      });
      expect(messages[0]).toEqual({ type: "init", payload: [] });
      const received = new Promise<unknown>((resolve, reject) => {
        const timeout = setTimeout(() => reject(new Error("WebSocket event timeout")), 2000);
        socket.onmessage = message => { clearTimeout(timeout); resolve(JSON.parse(String(message.data))); };
      });
      await fetch(new URL("/api/events", app.server.url), { method: "POST", body: JSON.stringify(event()) });
      expect(await received).toMatchObject({ type: "event", payload: { id: "event-1" } });
    } finally { socket.close(); await app.stop(); }
  });
});

it("keeps the browser script syntactically valid", async () => {
  const html = await Bun.file(new URL("../public/index.html", import.meta.url)).text();
  const source = /<script>([\s\S]*?)<\/script>/.exec(html)?.[1];
  if (!source) throw new Error("Inspector browser script missing");
  expect(() => new Script(source)).not.toThrow();
});
