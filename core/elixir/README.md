# clrinfex

An Elixir core for both modular monoliths and distributed applications, with an
optional live streaming adapter. The core contains no commerce, database,
authentication provider, policy engine or saga-engine requirement.

## Core API

`Clrinfex.Core.Context`, `Error`, and `CloudEvent` validate string-keyed JSON maps
and return `{:ok, value}` or `{:error, standard_error_envelope}`. Native `with`
expressions work directly; `Result.map/2` and `Result.bind/2` short-circuit errors.

```elixir
alias Clrinfex.Core.{Context, CloudEvent}

with {:ok, context} <- Context.new("tenant-1", "workflow-1", "request-1"),
     {:ok, event} <- CloudEvent.new(
       context, "clrinf/documents", "com.clrinf.documents.Changed.v1",
       %{"document_id" => "doc-1"}
     ) do
  CloudEvent.encode(event)
end
```

Context requires `tenant_id`, `correlation_id`, and `causation_id`. Events require
CloudEvents `specversion: "1.0"`, `id`, `source`, `type`, an ISO-8601 timestamp with
an offset, `datacontenttype: "application/json"`, `tenantid`, `correlationid`,
`causationid`, and object-valued `data`. Optional `is_error` / `is_compensation`
flags must be booleans. Validation preserves extensions and flags. Error values
contain `error_code`, `message`, `correlation_id`, `tenant_id`, `retryable`, and
optional JSON `details`. For malformed inputs without usable metadata, validation
errors explicitly use `"unknown"` identifiers; these are not business context.

Metadata is not authentication or authorization. Applications must bind tenants
to trusted identity and enforce tenant access in storage and transports.

## In-process example (no NATS or HTTP)

Requires Elixir 1.16+ and installed dependencies (`mix deps.get`).

```sh
CLRINF_MODE=core mix run examples/monolith.exs
```

This calls `Clrinfex.Examples.Documents.approve/2` directly and prints a canonical
event. It does not persist a document. `CLRINF_MODE=core` starts an empty application
supervisor, without the streaming HTTP listener, presence process or NATS bridge.
Transport packages remain dependencies but no network service is started.

## HTTP and live streaming example

```sh
NATS_URL='' mix run --no-halt
curl -sS http://localhost:4000/api/example/documents/approve \
  -H 'content-type: application/json' \
  -H 'x-tenant-id: tenant-1' \
  -H 'x-correlation-id: workflow-1' \
  -H 'x-causation-id: request-1' \
  -d '{"document_id":"doc-1"}'
```

The HTTP example calls the same pure operation and returns its CloudEvent, without
persisting or publishing it. Missing headers or invalid commands return standard
400 error envelopes. `Clrinfex.Adapters.HTTP` maps headers and Result tuples to
Plug responses. This example is not an authenticated production API.

`POST /api/broadcast` accepts a **complete canonical event**, using metadata from
the envelope; no additional headers are required. It returns 202 with
`{"status":"broadcasted","event":...}`, 400 for invalid events/subjects, or a
retryable `BUS_UNAVAILABLE` error with 503 during a configured NATS outage.
Malformed JSON and unsupported media types also return standard errors.
Legacy external `{"type":...}`-only payloads are intentionally rejected.

HTTP JSON bodies and encoded canonical domain events are capped at **262,144
bytes (256 KiB)**, including in standalone mode. NATS ingress uses the same event
cap. For publishing, the broker's `max_payload` is read on every connection and
reconnection: encoded JSON **plus encoded HPUB headers and the 12-byte NATS
header framing** must also fit that limit. Oversized events return a standard
413 `PAYLOAD_TOO_LARGE` before publication or local fan-out. HTTP body limits apply
to raw request bytes (including whitespace); event limits apply to encoded JSON.

`NATS_URL=''` selects standalone mode; a configured but unavailable NATS server
is **disconnected**, never silently standalone. `/health` returns 503 while
disconnected. The bridge reconnects and retains local subscriptions. Core NATS
is live, non-durable fan-out: publication is not a JetStream persistence
acknowledgement, missed events are not replayed, and 202 is not proof of consumer
processing. `CLRINF_NATS_SUBJECT` controls the subscription (default
`com.clrinf.>`); publish types must be valid concrete NATS subjects.

Presence and WebRTC signaling are process-local messages delivered through
`NatsBridge.broadcast_local/1`, not canonical external events. For compatibility,
`broadcast_event/1` still accepts legacy `presence_update`/`signal` maps locally.
Canonical maps, even with those type names, still undergo full validation.
Incoming NATS messages must be canonical and their `type` must match the subject.
SSE and WebSocket subscribers currently receive all local bridge events: the
demo does not implement tenant-filtered fan-out or authenticated presence.

## Storage ports and reliability limits

`Clrinfex.Core.Outbox` defines append, tenant-scoped pending reads, and
acknowledgement. `Clrinfex.Core.Idempotency` defines tenant/consumer/key claims,
token-fenced completion and release. `Clrinfex.Adapters.Memory` implements both
using one GenServer. Concurrent claims have one owner; duplicate completed keys
return their cached result. Pending event IDs cannot be overwritten with a
different payload.

**Memory is not durable**: each call is atomic only inside one BEAM process;
there is no transaction spanning calls or application business state. Restart
loses events/results/claims. Claims do not expire, so worker failure requires
explicit release; completed results have no eviction policy. These are reference
ports, not production delivery guarantees or exactly-once processing.

A production storage adapter must transactionally commit business state, inbox
and outbox in the same database; a dispatcher must acknowledge only after the
chosen transport confirms acceptance. Crashes between publish and acknowledge
can duplicate delivery. Durable JetStream producer/consumer adapters, database
storage, policy/identity integrations and saga coordination are optional and are
not implemented here.

## Development

```sh
mix compile --warnings-as-errors
mix test --no-start
NATS_TEST_URL=nats://127.0.0.1:4222 mix test --no-start
```

Run the broker with `test/fixtures/nats-small-payload.conf` to exercise the
negotiated 1,024-byte HPUB limit below the application cap, including the exact
header-inclusive boundary and continued connection usability after rejection.

The test helper starts dependencies; tests own their service processes. Shared
fixtures default to `../tests/conformance/fixtures`, or
`CLRINF_CONFORMANCE_DIR=/absolute/fixture/directory`. Missing fixtures **fail**
the conformance suite. Standalone clones and Docker test runners must supply
both `valid.json` and `invalid.json`. NATS-tagged tests require an isolated broker
owned by the runner and cover ingress, echo suppression, local-only messages,
disconnection and reconnect.

## Dependency status

As of 2026-09-09, the locked `cowlib 2.20.0` is the latest stable version
[published on Hex](https://hex.pm/api/packages/cowlib), but `mix hex.audit` reports:

- [EEF-CVE-2026-43971](https://osv.dev/vulnerability/EEF-CVE-2026-43971):
  Link header directive smuggling.
- [EEF-CVE-2026-43966](https://osv.dev/vulnerability/EEF-CVE-2026-43966):
  HTTP response splitting in structured-header encoding.
- [EEF-CVE-2026-43969](https://osv.dev/vulnerability/EEF-CVE-2026-43969):
  cookie request-header injection.

Upstream source fixes or mitigations do not establish a patched stable package
for all three. No unsafe transitive override or unsupported fork is installed.
**The dependency advisory gate remains unresolved**; do not describe this
lockfile or HTTP adapter as security-cleared. Recheck upstream and update the
compatible stable release when one is available.
