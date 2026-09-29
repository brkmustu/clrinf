import { expect, it } from "bun:test";
import { CapacityError, MemoryIdempotencyStore, MemoryOutboxStore } from "./index";
import { createEvent, dispatchOutbox } from "../core";

const key = { tenant_id: "tenant", operation: "approve-document", key: "request" };
const event = (id = "event") => createEvent({
  tenant_id: "tenant", correlation_id: "correlation", causation_id: "request",
}, { id, source: "clrinf/documents", type: "DocumentApproved", data: { revision: 1 } });

it("claims idempotency atomically and scopes it by tenant and operation", async () => {
  const store = new MemoryIdempotencyStore();
  const claims = await Promise.all(Array.from({ length: 20 }, () => store.claim(key)));
  expect(claims.filter(c => c.status === "acquired")).toHaveLength(1);
  expect(claims.filter(c => c.status === "busy")).toHaveLength(19);
  const owner = claims.find(c => c.status === "acquired");
  if (!owner || owner.status !== "acquired") throw new Error("No claim acquired");
  await expect(store.complete(key, "wrong", null)).rejects.toThrow("not owned");
  await expect(store.release(key, "wrong")).rejects.toThrow("not owned");
  const result = { document_id: "doc" };
  await store.complete(key, owner.token, result);
  result.document_id = "mutated";
  expect(await store.claim(key)).toEqual({ status: "completed", result: { document_id: "doc" } });
  expect((await store.claim({ ...key, tenant_id: "another" })).status).toBe("acquired");
  expect((await store.claim({ ...key, operation: "archive" })).status).toBe("acquired");
});

it("expires completed entries only and rejects capacity rather than evicting active work", async () => {
  let now = 0;
  const store = new MemoryIdempotencyStore({ capacity: 1, ttlMs: 10, now: () => now });
  const claim = await store.claim(key);
  if (claim.status !== "acquired") throw new Error("Expected owner");
  now = 100;
  expect((await store.claim(key)).status).toBe("busy");
  await expect(store.claim({ ...key, key: "another" })).rejects.toBeInstanceOf(CapacityError);
  await store.complete(key, claim.token, { ok: true });
  now = 111;
  const renewed = await store.claim(key);
  if (renewed.status !== "acquired") throw new Error("Expected renewed owner");
  await expect(store.release(key, claim.token)).rejects.toThrow("not owned");
  await store.release(key, renewed.token);
  expect((await store.claim(key)).status).toBe("acquired");
});

it("leases outbox entries atomically, rejects stale owners and retries after expiry", async () => {
  let now = 0;
  const store = new MemoryOutboxStore({ capacity: 1, now: () => now });
  const original = event();
  await store.enqueue(original);
  original.data.revision = 99;
  await expect(store.enqueue(event("other"))).rejects.toBeInstanceOf(CapacityError);
  await expect(store.enqueue(event())).rejects.toThrow("already queued");
  const first = (await store.claim(1, 10))[0];
  if (!first) throw new Error("Expected delivery");
  expect(first.event.data.revision).toBe(1);
  expect(await store.claim(1, 10)).toEqual([]);
  now = 11;
  const second = (await store.claim(1, 10))[0];
  if (!second) throw new Error("Expected redelivery");
  expect(second.attempts).toBe(2);
  await expect(store.acknowledge("tenant", "event", first.token)).rejects.toThrow("not owned");
  await expect(store.acknowledge("different-tenant", "event", second.token)).rejects.toThrow("not owned");
  await store.retry("tenant", "event", second.token, 5);
  expect(await store.claim(1, 10)).toEqual([]);
  now = 16;
  const third = (await store.claim(1, 10))[0];
  if (!third) throw new Error("Expected delayed delivery");
  expect(third.attempts).toBe(3);
  await store.acknowledge("tenant", "event", third.token);
  expect(await store.claim(1, 10)).toEqual([]);
  await store.enqueue(event("new"));
});

it("dispatches only acknowledged successes and surfaces publisher failures", async () => {
  const store = new MemoryOutboxStore();
  await store.enqueue(event());
  await expect(dispatchOutbox(store, { publish: async () => { throw new Error("publisher unavailable"); } }, { retryDelayMs: 0 }))
    .rejects.toThrow("publisher unavailable");
  const published: string[] = [];
  expect(await dispatchOutbox(store, { publish: async e => { published.push(e.id); } })).toBe(1);
  expect(published).toEqual(["event"]);
  expect(await dispatchOutbox(store, { publish: async () => {} })).toBe(0);
});

it("acknowledgement loss can redeliver and durable guarantees are not simulated", async () => {
  let now = 0;
  const store = new MemoryOutboxStore({ now: () => now });
  await store.enqueue(event());
  const delivery = (await store.claim(1, 10))[0];
  if (!delivery) throw new Error("Expected delivery");
  // A published event remains pending when its worker dies before acknowledgement.
  now = 11;
  const redelivery = (await store.claim(1, 10))[0];
  expect(redelivery?.event.id).toBe(delivery.event.id);
  expect(redelivery?.token).not.toBe(delivery.token);
  expect(await new MemoryOutboxStore().claim(1, 10)).toEqual([]);
});
