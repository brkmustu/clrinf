import {
  createEvent, dispatchOutbox, parseContext, parseEvent, ValidationError,
  type Context, type IdempotencyStore, type OutboxStore,
} from "../src/core";
import { MemoryIdempotencyStore, MemoryOutboxStore } from "../src/adapters";

export class BusyError extends Error {}

/** An in-process workflow, not a transactional database implementation. */
export function documentWorkflow(idempotency: IdempotencyStore, outbox: OutboxStore) {
  return async function approve(context: Context, documentId: string, requestKey: string) {
    const ctx = parseContext(context);
    if (typeof documentId !== "string" || !documentId.trim()) throw new ValidationError("document_id is required");
    const key = { tenant_id: ctx.tenant_id, operation: "approve-document", key: requestKey };
    const claim = await idempotency.claim(key);
    if (claim.status === "busy") throw new BusyError("Request already in progress");
    if (claim.status === "completed") return parseEvent(claim.result);
    const event = createEvent(ctx, {
      source: "clrinf/documents", type: "com.clrinf.documents.DocumentApproved.v1",
      data: { document_id: documentId },
    });
    try {
      await outbox.enqueue(event);
    } catch (error) {
      await idempotency.release(key, claim.token);
      throw error;
    }
    // Separate operations, not an atomic commit; durable deployments must use a DB transaction.
    await idempotency.complete(key, claim.token, { ...event });
    return event;
  };
}

if (import.meta.main) {
  const outbox = new MemoryOutboxStore();
  const approve = documentWorkflow(new MemoryIdempotencyStore(), outbox);
  await approve({ tenant_id: "tenant-demo", correlation_id: "workflow-demo", causation_id: "request-demo" }, "document-1", "request-1");
  await dispatchOutbox(outbox, { publish: async event => { console.log(JSON.stringify(event)); } });
}
