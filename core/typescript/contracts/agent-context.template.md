# Agent Context: {{service_name}}

## 1. Constitution reference

The design invariants in `docs/governance/constitution.md` apply. Resolve this
repository-relative path from the umbrella root; adapt it when copying this
template into another repository.

- `NO-DISTRIBUTED-TX`: local ACID is allowed; no distributed 2PC/XA across service
  boundaries. Define retries, timeouts and compensation/recovery where needed.
- `TENANT-ISOLATION`: validate and propagate tenant context at every boundary.
- `RESULT-PATTERN`: expected failures are explicit results; preserve the standard
  error wire contract.
- `CONTRACT-FIRST`: define contracts before implementation; generate core and
  application models separately.
- `OBSERVABLE-BY-DEFAULT`: preserve tenant/correlation/causation lineage.
  OpenTelemetry is a recommended target, not guaranteed implemented support.

## 2. Service identity and stable wire contracts

- **Name:** {{service_name}}
- **Domain / namespace:** {{domain}}
- **Language / runtime:** {{language}}
- **Tenant isolation strategy:** {{tenant_isolation_strategy}}
- **Context:** `_context.schema.json` requires string `tenant_id`,
  `correlation_id`, `causation_id`.
- **Events:** `_envelope.schema.json` retains its required fields and maps context
  to `tenantid`, `correlationid`, `causationid`. `is_error` and `is_compensation`
  are optional booleans.
- **Errors:** unchanged `StandardErrorEnvelope` (`_error.schema.json`) requires
  `error_code`, `message`, `correlation_id`, `tenant_id`, `retryable`.
- **Namespace metadata:** `x-domain` is codegen metadata, not a domain-specific
  framework dependency; absent metadata defaults to `common`.

## 3. Capabilities and deployment choices

- **Selected adapters:** {{selected_adapters}}
- **Durability / transaction boundary:** {{durability_and_transaction_boundary}}
- **Implemented telemetry and known gaps:** {{telemetry_coverage}}

NATS, PostgreSQL (including RLS), Cedar and Nomad are optional capability adapters,
not universal core requirements. Record actual tested capabilities, not assumptions
based on adapter names.

## 4. Owned and consumed events

**Published:** {{owned_events_list}}

**Subscribed:** {{consumed_events_list}}

## 5. Application requirements (EARS)

{{ears_requirements}}

Application examples live in `examples/contracts`, including credential/JWT/Cedar
specific auth contracts. They are not core dependencies.

## 6. Reliability and multi-step processes

- **Dedupe key:** {{dedupe_key}} — scope by tenant + operation +
  message/idempotency key; never universally dedupe by `correlation_id`.
- **Dedupe retention / atomicity:** {{dedupe_retention_and_atomicity}}
- **Retry / timeout / recovery policy:** {{retry_timeout_recovery_policy}}
- **Saga role, if used:** {{saga_role}}
- **Compensating actions, if applicable:** {{compensating_actions}}

## 7. Code generation

Use the root canonical CLI from the umbrella repository root:

```sh
clrinf-codegen generate --schema-dir tools/clrinf-codegen/schemas --output tests/generated/core
clrinf-codegen generate --schema-dir examples/contracts/schemas --output tests/generated/all
```

The second output contains examples only. Keep application/core inputs and outputs
separate. Outside the checkout, supply explicit absolute schema/template paths.
Generated DTOs do not replace boundary schema validation.

## 8. SLO targets (service-specific, not framework guarantees)

- **Availability:** {{availability_target}}
- **P99 latency:** {{p99_latency_target}}
- **Event processing P99:** {{event_processing_p99_target}}
