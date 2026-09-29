import { createEvent, parseEvent, ValidationError, type CloudEventEnvelope } from "../core";
import type { InspectorEvent } from "../types";

export class InspectorEngine {
  private readonly events: InspectorEvent[] = [];
  private readonly sizes: number[] = [];
  private bytes = 0;
  constructor(private readonly capacity = 500) {
    if (!Number.isSafeInteger(capacity) || capacity < 1) throw new ValidationError("capacity must be a positive integer");
  }
  recordEvent(input: unknown): InspectorEvent {
    const event = parseEvent(input);
    const recorded = { ...event, received_at: new Date().toISOString(), is_error: event.is_error ?? false };
    const size = new TextEncoder().encode(JSON.stringify(recorded)).byteLength;
    if (size > 256 * 1024) throw new ValidationError("Recorded event exceeds 256 KiB");
    this.events.push(recorded);
    this.sizes.push(size);
    this.bytes += size;
    while (this.events.length > this.capacity || this.bytes > 1024 * 1024) {
      this.events.shift();
      this.bytes -= this.sizes.shift() ?? 0;
    }
    return structuredClone(recorded);
  }
  list(correlationId?: string | null, tenantId?: string | null): InspectorEvent[] {
    return structuredClone(this.events.filter(event =>
      (!correlationId || event.correlationid === correlationId) && (!tenantId || event.tenantid === tenantId)));
  }
  replay(input: unknown, mutateCorrelation = false): InspectorEvent {
    const event = parseEvent(input);
    return this.recordEvent({
      ...event, id: crypto.randomUUID(), time: new Date().toISOString(),
      causationid: event.id, correlationid: mutateCorrelation ? crypto.randomUUID() : event.correlationid,
    });
  }
  clear(): void { this.events.length = 0; this.sizes.length = 0; this.bytes = 0; }
  get size(): number { return this.events.length; }
}

export function documentSimulation(): CloudEventEnvelope[] {
  const correlation_id = crypto.randomUUID(), document_id = crypto.randomUUID();
  let causation_id: string = correlation_id;
  return ["DocumentSubmitted", "DocumentReviewed", "DocumentApproved", "DocumentArchived"].map((name, index) => {
    const event = createEvent({ tenant_id: "tenant-demo", correlation_id, causation_id }, {
      source: "clrinf/document-workflow", type: `com.clrinf.document.${name}.v1`,
      time: new Date(Date.now() + index * 45).toISOString(), data: { document_id },
    });
    causation_id = event.id;
    return event;
  });
}

export function waterfall(events: InspectorEvent[]) {
  const groups = new Map<string, InspectorEvent[]>();
  for (const event of events) {
    const key = JSON.stringify([event.tenantid, event.correlationid]);
    const group = groups.get(key) ?? [];
    group.push(event);
    groups.set(key, group);
  }
  return [...groups.values()].map(group => {
    const sorted = [...group].sort((a, b) => Date.parse(a.time) - Date.parse(b.time));
    const first = sorted[0];
    if (!first) throw new Error("Empty waterfall group");
    const base = Date.parse(first.time);
    const spans = sorted.map((event, index) => ({
      id: event.id, name: event.type, type: event.type, source: event.source,
      tenant_id: event.tenantid, offset_ms: Date.parse(event.time) - base,
      duration_ms: Math.max(15, Date.parse(sorted[index + 1]?.time ?? event.time) - Date.parse(event.time)),
      is_error: event.is_error, is_compensation: event.is_compensation ?? false, time: event.time,
    }));
    const last = spans.at(-1);
    return {
      correlation_id: first.correlationid, tenant_id: first.tenantid,
      total_duration_ms: last ? last.offset_ms + last.duration_ms : 0,
      spans_count: spans.length, spans,
    };
  });
}
