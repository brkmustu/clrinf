import { MemoryIdempotencyStore, MemoryOutboxStore, CapacityError } from "../src/adapters";
import { object, parseContext, ValidationError } from "../src/core";
import { BusyError, documentWorkflow } from "./monolith";

export function documentHandler() {
  const outbox = new MemoryOutboxStore();
  const approve = documentWorkflow(new MemoryIdempotencyStore(), outbox);
  return {
    outbox,
    async fetch(request: Request): Promise<Response> {
      const url = new URL(request.url);
      if (request.method !== "POST" || url.pathname !== "/documents/approve") return new Response("Not Found", { status: 404 });
      try {
        const context = parseContext({
          tenant_id: request.headers.get("x-tenant-id"),
          correlation_id: request.headers.get("x-correlation-id"),
          causation_id: request.headers.get("x-causation-id"),
        });
        const input = object(await request.json());
        if (typeof input.document_id !== "string") throw new ValidationError("document_id must be a string");
        const key = request.headers.get("idempotency-key");
        if (!key?.trim()) throw new ValidationError("Idempotency-Key is required");
        return Response.json(await approve(context, input.document_id, key), { status: 202 });
      } catch (error) {
        if (error instanceof ValidationError || error instanceof SyntaxError) {
          return Response.json({ error: error.message }, { status: 400 });
        }
        if (error instanceof BusyError) return Response.json({ error: error.message }, { status: 409 });
        if (error instanceof CapacityError) return Response.json({ error: error.message }, { status: 503 });
        throw error;
      }
    },
  };
}

if (import.meta.main) {
  const handler = documentHandler();
  Bun.serve({ hostname: "127.0.0.1", port: 4300, maxRequestBodySize: 64 * 1024, fetch: handler.fetch });
  console.warn("Development-only HTTP example on http://127.0.0.1:4300. Headers are NOT authentication; pending outbox is volatile.");
}
