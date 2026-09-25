import { afterAll, beforeAll, describe, expect, test } from "bun:test";
import { join } from "node:path";

const inspector = process.env.INSPECTOR_URL ?? "http://127.0.0.1:4200";
const streaming = process.env.STREAMING_URL ?? "http://127.0.0.1:4000";
const fixtures = process.env.CLRINF_CONFORMANCE_DIR ?? join(import.meta.dir, "../conformance/fixtures");
const enabled = process.env.CLRINF_E2E_STRICT !== "0";
const sockets: WebSocket[] = [];

async function ready(base: string, natsKey: string): Promise<void> {
  const deadline = Date.now() + 60_000;
  let failure = "not ready";
  while (Date.now() < deadline) {
    try {
      const response = await fetch(`${base}/health`, { signal: AbortSignal.timeout(2_000) });
      const health = await response.json();
      if (response.ok && health[natsKey] === "connected") return;
      failure = `HTTP ${response.status}: ${JSON.stringify(health)}`;
    } catch (error) {
      failure = String(error);
    }
    await Bun.sleep(250);
  }
  throw new Error(`${base} did not become ready: ${failure}`);
}

function connectInspector(): Promise<{ socket: WebSocket; events: unknown[] }> {
  return new Promise((resolve, reject) => {
    const socket = new WebSocket(`${inspector.replace(/^http/, "ws")}/ws`);
    sockets.push(socket);
    const events: unknown[] = [];
    const timer = setTimeout(() => {
      socket.close();
      reject(new Error("Inspector did not send an init message"));
    }, 5_000);

    socket.onerror = () => { clearTimeout(timer); reject(new Error("Inspector WebSocket failed")); };
    socket.onmessage = (message) => {
      const body = JSON.parse(String(message.data));
      if (body.type === "init") {
        clearTimeout(timer);
        resolve({ socket, events });
      } else if (body.type === "event") {
        events.push(body.payload);
      }
    };
  });
}

// Explicitly disabled E2E is reported as skipped, never as a successful offline simulation.
describe.skipIf(!enabled)("generic Elixir -> NATS -> TypeScript transport", () => {
  beforeAll(async () => {
    await Promise.all([ready(inspector, "nats"), ready(streaming, "nats_mode")]);
  });


  afterAll(() => { for (const socket of sockets) socket.close(); });

  test("publishes a canonical event and observes the same event in Inspector", async () => {
    const fixture = await Bun.file(join(fixtures, "valid.json")).json();
    const event = { ...fixture.event, id: crypto.randomUUID(), correlationid: crypto.randomUUID() };
    const { events } = await connectInspector();

    const response = await fetch(`${streaming}/api/broadcast`, {
      method: "POST",
      headers: {
        "Content-Type": "application/json",
        "x-tenant-id": event.tenantid,
        "x-correlation-id": event.correlationid,
        "x-causation-id": event.causationid,
      },
      body: JSON.stringify(event),
      signal: AbortSignal.timeout(5_000),
    });
    expect(response.status).toBe(202);

    const deadline = Date.now() + 5_000;
    while (events.length === 0 && Date.now() < deadline) await Bun.sleep(20);
    const received = events.find(value =>
      typeof value === "object" && value !== null && "id" in value && value.id === event.id);
    expect(received).toBeDefined();
    const forwarded: Record<string, unknown> = { ...received };
    delete forwarded.received_at;
    expect(forwarded).toEqual(event);
  }, 10_000);

  test("malformed external events are rejected rather than reported as delivered", async () => {
    const response = await fetch(`${streaming}/api/broadcast`, {
      method: "POST",
      headers: {
        "Content-Type": "application/json",
        "x-tenant-id": "tenant-example",
        "x-correlation-id": "workflow-example",
        "x-causation-id": "request-example",
      },
      body: JSON.stringify({ type: "com.clrinf.documents.Changed.v1", data: {} }),
      signal: AbortSignal.timeout(5_000),
    });
    expect(response.status).toBe(400);
    const error = await response.json();
    expect(error.error_code).toBeString();
    expect(error.retryable).toBe(false);
  });
});
