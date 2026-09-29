export type JsonValue = null | boolean | number | string | JsonValue[] | JsonObject;
export interface JsonObject { [key: string]: JsonValue }

export interface Context {
  tenant_id: string;
  correlation_id: string;
  causation_id: string;
}

export interface ErrorEnvelope {
  error_code: string;
  message: string;
  correlation_id: string;
  tenant_id: string;
  retryable: boolean;
  details?: JsonObject;
}

export interface CloudEventEnvelope<T extends JsonObject = JsonObject> {
  specversion: "1.0";
  id: string;
  source: string;
  type: string;
  subject?: string;
  time: string;
  datacontenttype: "application/json";
  tenantid: string;
  correlationid: string;
  causationid: string;
  data: T;
  is_error?: boolean;
  is_compensation?: boolean;
}

export class ValidationError extends Error {
  constructor(message: string) {
    super(message);
    this.name = "ValidationError";
  }
}

export function object(value: unknown): Record<string, unknown> {
  if (value === null || typeof value !== "object" || Array.isArray(value) ||
      (Object.getPrototypeOf(value) !== Object.prototype && Object.getPrototypeOf(value) !== null)) {
    throw new ValidationError("Expected a JSON object");
  }
  return value as Record<string, unknown>;
}

function text(value: unknown, field: string): string {
  if (typeof value !== "string" || value.trim().length === 0) {
    throw new ValidationError(`${field} must be a non-empty string`);
  }
  return value;
}

function flag(value: unknown, field: string): boolean {
  if (typeof value !== "boolean") throw new ValidationError(`${field} must be a boolean`);
  return value;
}

function json(value: unknown, depth = 0): JsonValue {
  if (depth > 64) throw new ValidationError("JSON nesting exceeds 64 levels");
  if (value === null || typeof value === "string" || typeof value === "boolean") return value;
  if (typeof value === "number" && Number.isFinite(value)) return value;
  if (Array.isArray(value)) return Array.from(value, item => json(item, depth + 1));
  const input = object(value);
  return Object.fromEntries(Object.entries(input).map(([key, item]) => [key, json(item, depth + 1)]));
}

export function parseJsonObject(value: unknown): JsonObject {
  object(value);
  return json(value) as JsonObject;
}

export function parseContext(value: unknown): Context {
  const input = object(value);
  return {
    tenant_id: text(input.tenant_id, "tenant_id"),
    correlation_id: text(input.correlation_id, "correlation_id"),
    causation_id: text(input.causation_id, "causation_id"),
  };
}

export function parseError(value: unknown): ErrorEnvelope {
  const input = object(value);
  return {
    error_code: text(input.error_code, "error_code"),
    message: text(input.message, "message"),
    correlation_id: text(input.correlation_id, "correlation_id"),
    tenant_id: text(input.tenant_id, "tenant_id"),
    retryable: flag(input.retryable, "retryable"),
    ...(input.details !== undefined ? { details: parseJsonObject(input.details) } : {}),
  };
}

function timestamp(value: unknown): string {
  const time = text(value, "time");
  const match = /^(\d{4})-(\d{2})-(\d{2})T(\d{2}):(\d{2}):(\d{2})(?:\.\d+)?(?:Z|[+-](\d{2}):(\d{2}))$/.exec(time);
  if (!match || !Number.isFinite(Date.parse(time))) throw new ValidationError("time must be RFC3339");
  const year = Number(match[1]), month = Number(match[2]), day = Number(match[3]);
  const leap = year % 4 === 0 && (year % 100 !== 0 || year % 400 === 0);
  const days = [31, leap ? 29 : 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31][month - 1] ?? 0;
  if (month < 1 || month > 12 || day < 1 || day > days || Number(match[4]) > 23 ||
      Number(match[5]) > 59 || Number(match[6]) > 59 || Number(match[7] ?? 0) > 23 ||
      Number(match[8] ?? 0) > 59) throw new ValidationError("time must be a valid RFC3339 timestamp");
  return time;
}

export function parseEvent(value: unknown): CloudEventEnvelope {
  const input = object(value);
  if (input.specversion !== "1.0") throw new ValidationError('specversion must be "1.0"');
  if (input.datacontenttype !== "application/json") throw new ValidationError("datacontenttype must be application/json");
  return {
    specversion: "1.0",
    id: text(input.id, "id"),
    source: text(input.source, "source"),
    type: text(input.type, "type"),
    time: timestamp(input.time),
    datacontenttype: "application/json",
    tenantid: text(input.tenantid, "tenantid"),
    correlationid: text(input.correlationid, "correlationid"),
    causationid: text(input.causationid, "causationid"),
    data: parseJsonObject(input.data),
    ...(input.subject !== undefined ? { subject: text(input.subject, "subject") } : {}),
    ...(input.is_error !== undefined ? { is_error: flag(input.is_error, "is_error") } : {}),
    ...(input.is_compensation !== undefined ? { is_compensation: flag(input.is_compensation, "is_compensation") } : {}),
  };
}

export function createEvent(context: Context, input: {
  source: string; type: string; data: JsonObject; id?: string; time?: string;
  is_error?: boolean; is_compensation?: boolean;
}): CloudEventEnvelope {
  const ctx = parseContext(context);
  return parseEvent({
    ...input, specversion: "1.0", datacontenttype: "application/json",
    id: input.id ?? crypto.randomUUID(), time: input.time ?? new Date().toISOString(),
    tenantid: ctx.tenant_id, correlationid: ctx.correlation_id, causationid: ctx.causation_id,
  });
}

export function contextFromEvent(event: CloudEventEnvelope): Context {
  const parsed = parseEvent(event);
  return { tenant_id: parsed.tenantid, correlation_id: parsed.correlationid, causation_id: parsed.id };
}

export function createError(context: Context, input: {
  error_code: string; message: string; retryable?: boolean; details?: JsonObject;
}): ErrorEnvelope {
  const ctx = parseContext(context);
  return parseError({
    ...input,
    tenant_id: ctx.tenant_id,
    correlation_id: ctx.correlation_id,
    retryable: input.retryable ?? false,
  });
}
