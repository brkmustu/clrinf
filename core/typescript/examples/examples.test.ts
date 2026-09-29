import { expect, it } from "bun:test";
import { documentHandler } from "./http-service";

it("uses the same monolith workflow over HTTP with tenant-scoped idempotency", async () => {
  const handler = documentHandler();
  const request = (tenant = "tenant") => new Request("http://localhost/documents/approve", {
    method: "POST",
    headers: {
      "x-tenant-id": tenant, "x-correlation-id": "workflow", "x-causation-id": "request", "idempotency-key": "request-1",
    },
    body: JSON.stringify({ document_id: "document-1" }),
  });
  const first = await handler.fetch(request());
  expect(first.status).toBe(202);
  const event = await first.json();
  expect(await (await handler.fetch(request())).json()).toEqual(event);
  const other = await (await handler.fetch(request("other"))).json();
  expect(other.tenantid).toBe("other");
  expect(other.id).not.toBe(event.id);
  expect(await handler.outbox.claim(10, 1000)).toHaveLength(2);
  expect((await handler.fetch(new Request("http://localhost/documents/approve", { method: "POST", body: "{}" }))).status).toBe(400);
});
