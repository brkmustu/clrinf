import type { CloudEventEnvelope } from "./core";
export type { CloudEventEnvelope } from "./core";

export interface InspectorEvent extends CloudEventEnvelope {
  received_at: string;
  latency_ms?: number;
  is_error?: boolean;
}

export interface WebSocketMessage {
  type: "event" | "init" | "clear" | "ping";
  payload?: InspectorEvent | InspectorEvent[];
}
