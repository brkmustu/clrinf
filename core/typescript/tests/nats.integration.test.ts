import { expect, it } from "bun:test";
import { connect, StringCodec } from "nats";
import { createEvent, dispatchOutbox, object, parseEvent } from "../src/core";
import { MemoryOutboxStore } from "../src/adapters";
import { startInspector } from "../src/inspector";

const natsUrl = process.env.CLRINF_NATS_TEST_URL;

it.skipIf(!natsUrl)("dispatches through real NATS to Inspector, rejecting invalid envelopes and keeping replay local", async () => {
  if (!natsUrl) throw new Error("CLRINF_NATS_TEST_URL must point to a test NATS server");
  const inspector = await startInspector({ port: 0, natsUrl });
  const nc = await connect({ servers: natsUrl, timeout: 3000 });
  const codec = StringCodec();
  const event = createEvent({
    tenant_id: "tenant-integration", correlation_id: crypto.randomUUID(), causation_id: "request",
  }, {
    source: "clrinf/documents", type: `com.clrinf.tests.${crypto.randomUUID()}`,
    data: { document_id: "document-integration" }, is_error: false, is_compensation: true,
  });
  let observed = 0;
  const observer = nc.subscribe(event.type, { callback: error => {
    if (error) throw error;
    observed++;
  } });
  try {
    await nc.flush();
    const health = object(await (await fetch(new URL("/health", inspector.server.url))).json());
    expect(health.nats).toBe("connected");
    const outbox = new MemoryOutboxStore();
    await outbox.enqueue(event);
    expect(await dispatchOutbox(outbox, {
      publish: async envelope => { nc.publish(envelope.type, codec.encode(JSON.stringify(envelope))); await nc.flush(); },
    })).toBe(1);
    await waitUntil(() => inspector.engine.size === 1);
    const recorded = inspector.engine.list()[0];
    expect(parseEvent(recorded)).toEqual(event);
    nc.publish("com.clrinf.tests.invalid", codec.encode(JSON.stringify({ ...event, data: [] })));
    await nc.flush();
    const response = await fetch(new URL("/api/replay", inspector.server.url), {
      method: "POST", body: JSON.stringify({ event_id: event.id }),
    });
    expect(response.status).toBe(200);
    const replay = object(await response.json());
    expect(replay.destination).toBe("inspector-only");
    expect(parseEvent(replay.event)).toMatchObject({
      causationid: event.id, tenantid: event.tenantid, correlationid: event.correlationid, is_compensation: true,
    });
    await Bun.sleep(100);
    expect(inspector.engine.size).toBe(2);
    expect(observed).toBe(1);
  } finally {
    observer.unsubscribe();
    await nc.close();
    await inspector.stop();
  }
}, 10_000);

async function waitUntil(condition: () => boolean): Promise<void> {
  const deadline = Date.now() + 2000;
  while (!condition()) {
    if (Date.now() >= deadline) throw new Error("Timed out waiting for real NATS delivery");
    await Bun.sleep(10);
  }
}
