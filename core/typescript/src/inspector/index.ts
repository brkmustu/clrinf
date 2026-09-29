import { connect, Events, StringCodec, type NatsConnection, type Subscription } from "nats";
import type { ServerWebSocket } from "bun";
import { object, ValidationError } from "../core";
import type { WebSocketMessage } from "../types";
import { InspectorEngine, documentSimulation, waterfall } from "./engine";
export { InspectorEngine, documentSimulation, waterfall } from "./engine";

export interface InspectorOptions {
  port?: number;
  hostname?: string;
  natsUrl?: string;
  token?: string;
  allowUnauthenticated?: boolean;
  capacity?: number;
  maxClients?: number;
}

export async function startInspector(options: InspectorOptions = {}) {
  const hostname = options.hostname ?? "127.0.0.1";
  const port = options.port ?? 4200, maxClients = options.maxClients ?? 32;
  if (!Number.isInteger(port) || port < 0 || port > 65535) throw new ValidationError("Invalid port");
  if (!Number.isSafeInteger(maxClients) || maxClients < 1) throw new ValidationError("Invalid maxClients");
  if (options.token !== undefined && !options.token.trim()) throw new ValidationError("Token must not be empty");
  if (!["127.0.0.1", "::1", "localhost"].includes(hostname) && !options.token && !options.allowUnauthenticated) {
    throw new Error("Remote Inspector requires INSPECTOR_TOKEN or explicit development-only INSPECTOR_ALLOW_UNAUTHENTICATED=true");
  }
  if (!options.token) console.warn("[Inspector] Development-only, unauthenticated, no tenant authorization. Do not expose publicly.");
  const engine = new InspectorEngine(options.capacity);
  const sockets = new Set<ServerWebSocket<undefined>>();
  function send(ws: ServerWebSocket<undefined>, serialized: string) {
    if (ws.send(serialized) <= 0) {
      console.warn("[Inspector] Closing slow WebSocket client");
      sockets.delete(ws);
      ws.close(1013, "Slow consumer; reconnect for bounded snapshot");
    }
  }
  function broadcast(message: WebSocketMessage) {
    const serialized = JSON.stringify(message);
    for (const ws of sockets) send(ws, serialized);
  }
  function recordEvent(value: unknown) {
    const event = engine.recordEvent(value);
    broadcast({ type: "event", payload: event });
    return event;
  }
  let nc: NatsConnection | undefined, subscription: Subscription | undefined;
  let subscriber: Promise<void> | undefined;
  let connectionStatus: Promise<void> | undefined;
  let stopStatus: (() => void) | undefined;
  let connected = false;
  let subscriberActive = false;
  const maxBytes = 256 * 1024;
  if (options.natsUrl) {
    nc = await connect({ servers: options.natsUrl, timeout: 3000, reconnect: true, maxReconnectAttempts: 10 });
    connected = true;
    const connection = nc;
    const statuses = connection.status();
    // NATS 2.x exposes a queued iterator that does not finish when the connection closes.
    if (!("stop" in statuses) || typeof statuses.stop !== "function") {
      await connection.close();
      throw new Error("NATS status iterator does not support lifecycle cleanup");
    }
    const stopIterator = statuses.stop.bind(statuses);
    stopStatus = () => { stopIterator(); };
    connectionStatus = (async () => {
      for await (const status of statuses) {
        if (status.type === Events.Disconnect) connected = false;
        if (status.type === Events.Reconnect) connected = true;
      }
      connected = false;
    })();
    void connection.closed().then(() => stopIterator());
    subscription = nc.subscribe("com.clrinf.>");
    const sub = subscription;
    const codec = StringCodec();
    subscriberActive = true;
    subscriber = (async () => {
      try {
        for await (const message of sub) {
          try {
            if (message.data.byteLength > maxBytes) throw new ValidationError("NATS event exceeds size limit");
            recordEvent(JSON.parse(codec.decode(message.data)));
          } catch (error) {
            if (!(error instanceof SyntaxError || error instanceof ValidationError)) throw error;
            console.warn("[Inspector] Rejected NATS event:", error.message);
          }
        }
      } finally { subscriberActive = false; }
    })();
    // Attach immediately so an unexpected subscriber failure is visible even before shutdown.
    void subscriber.catch(error => console.error("[Inspector] Subscriber failed:", error));
    try { await nc.flush(); } catch (error) {
      await nc.close();
      await connectionStatus;
      throw error;
    }
  }
  function authorized(req: Request): boolean {
    if (!options.token) return true;
    const header = req.headers.get("authorization");
    return header === `Bearer ${options.token}` ||
      header === `Basic ${Buffer.from(`inspector:${options.token}`, "utf8").toString("base64")}`;
  }
  async function body(req: Request): Promise<unknown> {
    try { return await req.json(); }
    catch (error) {
      if (error instanceof SyntaxError) throw new ValidationError("Malformed JSON request");
      throw error;
    }
  }
  const nats = nc;
  let server;
  try {
    server = Bun.serve<undefined>({
      hostname, port, maxRequestBodySize: maxBytes,
      async fetch(req, server) {
        const url = new URL(req.url);
        // Readiness contains no event data; authentication protects all tooling and data endpoints.
        if (url.pathname === "/health" && req.method === "GET") {
          const ready = !nats || (connected && subscriberActive && !nats.isClosed() && !nats.isDraining());
          return Response.json({
            status: ready ? "healthy" : "degraded",
            service: "clrinfjs-event-inspector", events_count: engine.size,
            active_ws_clients: sockets.size, nats: !nats ? "disabled" : ready ? "connected" : "disconnected",
          }, { status: ready ? 200 : 503 });
        }
        if (!authorized(req)) return new Response("Inspector authentication required", {
          status: 401, headers: { "WWW-Authenticate": 'Basic realm="Inspector", charset="UTF-8"' },
        });
        if ((req.method !== "GET" || url.pathname === "/ws") &&
            req.headers.has("origin") && req.headers.get("origin") !== url.origin) {
          return new Response("Cross-origin access denied", { status: 403 });
        }
        if (url.pathname === "/ws") {
          if (sockets.size >= maxClients) return new Response("Client capacity exhausted", { status: 503 });
          return server.upgrade(req, { data: undefined }) ? undefined : new Response("Upgrade failed", { status: 400 });
        }
        try {
          if (url.pathname === "/api/events" && req.method === "GET") {
            return Response.json(engine.list(url.searchParams.get("correlation_id"), url.searchParams.get("tenant_id")));
          }
          if (url.pathname === "/api/events" && req.method === "POST") {
            return Response.json(recordEvent(await body(req)), { status: 201 });
          }
          if (url.pathname === "/api/events" && req.method === "DELETE") {
            engine.clear();
            broadcast({ type: "clear" });
            return new Response(null, { status: 204 });
          }
          if (url.pathname === "/api/replay" && req.method === "POST") {
            const input = object(await body(req));
            if (input.publish !== undefined && input.publish !== false) throw new ValidationError("Inspector replay is local-only");
            if (input.mutate_correlation !== undefined && typeof input.mutate_correlation !== "boolean") {
              throw new ValidationError("mutate_correlation must be boolean");
            }
            if (input.event_id !== undefined && (typeof input.event_id !== "string" || !input.event_id.trim())) {
              throw new ValidationError("event_id must be a non-empty string");
            }
            if (input.tenant_id !== undefined && typeof input.tenant_id !== "string") throw new ValidationError("tenant_id must be string");
            const matches = input.event_id === undefined ? [] : engine.list(null, input.tenant_id).filter(e => e.id === input.event_id);
            if (matches.length > 1) return Response.json({ error: "Ambiguous event_id; provide tenant_id" }, { status: 409 });
            const event = input.event_id === undefined ? input.event : matches[0];
            if (event === undefined) return Response.json({ error: "Event not found for replay" }, { status: 404 });
            const recorded = engine.replay(event, input.mutate_correlation === true);
            broadcast({ type: "event", payload: recorded });
            return Response.json({ status: "replayed", destination: "inspector-only", event: recorded });
          }
          if (url.pathname === "/api/waterfall" && req.method === "GET") {
            return Response.json(waterfall(engine.list(url.searchParams.get("correlation_id"), url.searchParams.get("tenant_id"))));
          }
          if (url.pathname === "/api/simulate" && req.method === "POST") {
            const events = documentSimulation();
            // Synchronous bounded recording avoids an unbounded timer queue under repeated requests.
            for (const event of events) recordEvent(event);
            return Response.json({ status: "simulating", correlation_id: events[0]?.correlationid, events_queued: events.length });
          }
          if ((url.pathname === "/" || url.pathname === "/index.html") && req.method === "GET") {
            return new Response(Bun.file(new URL("../../public/index.html", import.meta.url)), {
              headers: { "Content-Type": "text/html; charset=utf-8" },
            });
          }
          return new Response("Not Found", { status: 404 });
        } catch (error) {
          if (error instanceof ValidationError) return Response.json({ error: error.message }, { status: 400 });
          throw error;
        }
      },
      websocket: {
        maxPayloadLength: 1024,
        backpressureLimit: 2 * 1024 * 1024,
        closeOnBackpressureLimit: true,
        open(ws) {
          if (sockets.size >= maxClients) { ws.close(1013, "Client capacity exhausted"); return; }
          sockets.add(ws);
          send(ws, JSON.stringify({ type: "init", payload: engine.list() }));
        },
        message() {},
        close(ws) { sockets.delete(ws); },
      },
    });
  } catch (error) {
    await nc?.close();
    stopStatus?.();
    throw error;
  }
  return {
    server, engine,
    async stop() {
      server.stop(true);
      subscription?.unsubscribe();
      await nc?.close();
      await subscriber;
      await connectionStatus;
    },
  };
}
