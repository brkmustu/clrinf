# Agent Context: clrinfex

## 1. Constitution Reference

The local `contracts/constitution.md` defines contract-first design,
explicit Results, tenant isolation, observability and no distributed transaction.
Keep storage/transport/policy choices optional. Do not promote example domains
into core abstractions or claim guarantees this implementation does not provide.

## 2. Service Identity

- Name: clrinfex.
- Domain: reusable Elixir core with optional live streaming/presence adapters.
- Runtime: Elixir 1.16+; Docker uses Elixir 1.17.
- Tenancy: explicit context and tenant-scoped memory keys. The streaming demo
  lacks authentication and tenant-filtered subscriptions; not production isolation.
- Errors: string-keyed standard envelopes carried by native Result tuples.

## 3. Owned Events (Published)

Core owns no business events. `com.clrinf.documents.DocumentApproved.v1` belongs
only to the pure document example. The HTTP broadcast adapter validates and
relays supplied canonical domain events unchanged.

## 4. Consumed Events (Subscribed)

NATS Core subscription defaults to `com.clrinf.>` and is configurable using
`CLRINF_NATS_SUBJECT`. Incoming envelopes and subject/type agreement are checked.
Presence/signaling remain explicitly local and must not enter the domain bus.

## 5. Domain Requirements (EARS)

- WHEN metadata/envelopes are invalid, validators SHALL return standard errors.
- WHEN a configured NATS server is unavailable, health/publish SHALL return 503
  and the bridge SHALL retry without silently selecting standalone mode.
- WHEN running with `CLRINF_MODE=core`, the application SHALL NOT start a network
  listener, presence process or NATS connection.
- WHEN fixtures are absent, conformance tests SHALL fail rather than skip.
- WHEN using memory storage, documentation SHALL disclose process-local atomicity,
  data loss on restart, no business-state transaction and no claim expiry.

## 6. Saga Participation

No saga engine. Optional event flags are preserved; orchestration and compensation
remain application choices. Outbox and idempotency ports are implemented only by
a volatile reference; there is no durable JetStream adapter or database transaction.

## 7. SLO Targets

No measured production SLO is asserted. Core NATS fan-out is not durable delivery.

## Current State

Phase-0 reconnect, no-echo, standalone/disconnected modes and 503 behavior are
retained. Core contracts, polyglot fixture tests, ports/memory and core-only/HTTP
examples are implemented. Preserve the existing uncommitted phase-0 work.
The stable cowlib dependency still has unresolved upstream advisories documented
in README; never claim the dependency gate passes or silently suppress it.
