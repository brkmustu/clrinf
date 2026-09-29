import {
  parseEvent, parseJsonObject, ValidationError,
  type CloudEventEnvelope, type IdempotencyClaim, type IdempotencyKey,
  type IdempotencyStore, type JsonValue, type OutboxDelivery, type OutboxStore,
} from "../core";

export class CapacityError extends Error {
  constructor() { super("Memory store capacity exhausted"); this.name = "CapacityError"; }
}

function positive(value: number, field: string): number {
  if (!Number.isSafeInteger(value) || value <= 0) throw new ValidationError(`${field} must be a positive integer`);
  return value;
}

function keyOf(...parts: string[]): string {
  if (parts.some(part => typeof part !== "string" || !part.trim())) throw new ValidationError("Key components must be non-empty strings");
  return JSON.stringify(parts);
}

interface Pending { token: string; status: "pending" }
interface Completed { token: string; status: "completed"; result: JsonValue; expires: number }

/** Process-local only. Pending claims do not expire: release explicitly on failed work. */
export class MemoryIdempotencyStore implements IdempotencyStore {
  private readonly entries = new Map<string, Pending | Completed>();
  private readonly capacity: number;
  private readonly ttlMs: number;
  constructor(options: { capacity?: number; ttlMs?: number; now?: () => number } = {}) {
    this.capacity = positive(options.capacity ?? 1_000, "capacity");
    this.ttlMs = positive(options.ttlMs ?? 300_000, "ttlMs");
    this.now = options.now ?? Date.now;
  }
  private readonly now: () => number;
  private key(key: IdempotencyKey): string { return keyOf(key.tenant_id, key.operation, key.key); }
  async claim(key: IdempotencyKey): Promise<IdempotencyClaim> {
    const id = this.key(key), now = this.now();
    for (const [id, entry] of this.entries) {
      if (entry.status === "completed" && entry.expires <= now) this.entries.delete(id);
    }
    const existing = this.entries.get(id);
    if (existing?.status === "completed") return { status: "completed", result: structuredClone(existing.result) };
    if (existing) return { status: "busy" };
    if (this.entries.size >= this.capacity) throw new CapacityError();
    const token = crypto.randomUUID();
    this.entries.set(id, { status: "pending", token });
    return { status: "acquired", token };
  }
  async complete(key: IdempotencyKey, token: string, result: JsonValue): Promise<void> {
    const id = this.key(key);
    this.assertOwner(id, token);
    const cloned = parseJsonObject({ result }).result;
    if (cloned === undefined) throw new ValidationError("result must be JSON");
    this.entries.set(id, { status: "completed", token, result: cloned, expires: this.now() + this.ttlMs });
  }
  async release(key: IdempotencyKey, token: string): Promise<void> {
    const id = this.key(key);
    this.assertOwner(id, token);
    this.entries.delete(id);
  }
  private assertOwner(id: string, token: string): void {
    const entry = this.entries.get(id);
    if (!entry || entry.token !== token || entry.status !== "pending") throw new Error("Idempotency claim is not owned");
  }
}

interface Entry { event: CloudEventEnvelope; attempts: number; available: number; token?: string; leaseUntil?: number }

/** Process-local queue; not atomic with database changes and never a durable outbox. */
export class MemoryOutboxStore implements OutboxStore {
  private readonly entries = new Map<string, Entry>();
  private readonly capacity: number;
  private readonly now: () => number;
  constructor(options: { capacity?: number; now?: () => number } = {}) {
    this.capacity = positive(options.capacity ?? 1_000, "capacity");
    this.now = options.now ?? Date.now;
  }
  async enqueue(value: CloudEventEnvelope): Promise<void> {
    const event = parseEvent(value), id = keyOf(event.tenantid, event.id);
    if (this.entries.has(id)) throw new Error("Event already queued for tenant");
    if (this.entries.size >= this.capacity) throw new CapacityError();
    this.entries.set(id, { event, attempts: 0, available: this.now() });
  }
  async claim(limit: number, leaseMs: number): Promise<OutboxDelivery[]> {
    positive(limit, "limit"); positive(leaseMs, "leaseMs");
    const now = this.now(), deliveries: OutboxDelivery[] = [];
    for (const entry of this.entries.values()) {
      if (deliveries.length >= limit) break;
      if (entry.available > now || (entry.leaseUntil !== undefined && entry.leaseUntil > now)) continue;
      entry.token = crypto.randomUUID();
      entry.leaseUntil = now + leaseMs;
      entry.attempts++;
      deliveries.push({ event: structuredClone(entry.event), token: entry.token, attempts: entry.attempts });
    }
    return deliveries;
  }
  async acknowledge(tenantId: string, eventId: string, token: string): Promise<void> {
    const id = keyOf(tenantId, eventId);
    this.assertOwner(id, token);
    this.entries.delete(id);
  }
  async retry(tenantId: string, eventId: string, token: string, delayMs: number): Promise<void> {
    if (!Number.isSafeInteger(delayMs) || delayMs < 0) throw new ValidationError("delayMs must be nonnegative");
    const entry = this.assertOwner(keyOf(tenantId, eventId), token);
    entry.available = this.now() + delayMs;
    delete entry.token;
    delete entry.leaseUntil;
  }
  private assertOwner(id: string, token: string): Entry {
    const entry = this.entries.get(id);
    if (!entry || entry.token !== token || entry.leaseUntil === undefined || entry.leaseUntil <= this.now()) {
      throw new Error("Outbox lease expired or is not owned");
    }
    return entry;
  }
}
