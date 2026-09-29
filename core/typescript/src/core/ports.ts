import type { CloudEventEnvelope, JsonValue } from "./contracts";

export interface IdempotencyKey { tenant_id: string; operation: string; key: string }
export type IdempotencyClaim =
  | { status: "acquired"; token: string }
  | { status: "busy" }
  | { status: "completed"; result: JsonValue };

/** Durable implementations must claim atomically across all workers. */
export interface IdempotencyStore {
  claim(key: IdempotencyKey): Promise<IdempotencyClaim>;
  complete(key: IdempotencyKey, token: string, result: JsonValue): Promise<void>;
  release(key: IdempotencyKey, token: string): Promise<void>;
}

export interface OutboxDelivery {
  event: CloudEventEnvelope;
  token: string;
  attempts: number;
}

/** Enqueue must share the business transaction in a durable implementation. */
export interface OutboxStore {
  enqueue(event: CloudEventEnvelope): Promise<void>;
  claim(limit: number, leaseMs: number): Promise<OutboxDelivery[]>;
  acknowledge(tenantId: string, eventId: string, token: string): Promise<void>;
  retry(tenantId: string, eventId: string, token: string, delayMs: number): Promise<void>;
}

export interface EventPublisher {
  publish(event: CloudEventEnvelope): Promise<void>;
}

/** At-least-once: a crash after publishing but before acknowledgement can duplicate delivery. */
export async function dispatchOutbox(store: OutboxStore, publisher: EventPublisher, options: {
  limit?: number; leaseMs?: number; retryDelayMs?: number;
} = {}): Promise<number> {
  const deliveries = await store.claim(options.limit ?? 10, options.leaseMs ?? 30_000);
  let sent = 0;
  for (const delivery of deliveries) {
    const { event, token } = delivery;
    try {
      await publisher.publish(event);
    } catch (error) {
      await store.retry(event.tenantid, event.id, token, options.retryDelayMs ?? 1_000);
      throw error;
    }
    await store.acknowledge(event.tenantid, event.id, token);
    sent++;
  }
  return sent;
}
