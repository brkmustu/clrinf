# Incremental Adoption in Existing Projects

You do not need to rewrite your application or adopt a new runtime framework to use `clrinf`.

## 1. 5-Minute Guardrails (Zero Runtime Lock-In)

Start with the single-binary CLI (`clrinf`) to enforce layer rules, module boundaries, generated-file protection, and event contracts in your existing repository:

```bash
# Auto-detect domain/service/schema directories and bootstrap clrinf.rules.toml,
# synced AGENTS/CLAUDE/Cursor rules, post-edit hooks, and optional GitHub Actions CI:
clrinf init --guardrails --ci
```

Preview or run full verification and diagnostics at any time:
```bash
clrinf doctor            # verify rules, agent instruction sync, hooks, schemas & baseline health
clrinf verify            # topology + code<->contract drift + [[forbid]] + [[boundaries]]
clrinf verify --changed  # incremental check (plus generated-file edit protection)
clrinf verify --format sarif --output clrinf.sarif  # SARIF 2.1.0 for GitHub Code Scanning
```

If your codebase publishes or handles events without using `clrinf`-generated symbol names, annotate those call sites with a comment so drift and impact checks still track them:
```ts
// @clrinf:publishes billing.invoice.issued.v1
```
```python
# @clrinf:handles billing.invoice.issued.v1
```

## 2. Wire-Format Event Contracts First

Keep your existing domain models as-is. Map service boundaries to `_context`, `_error`, and `_envelope` schemas, and generate canonical DTOs and publisher/subscriber shells from your own schema folder:

```bash
clrinf generate \
  --schema-dir ./my-contracts \
  --templates-dir ./tools/clrinf-codegen/templates \
  --output ./generated-contracts
```

Review the supported JSON Schema subset before relying on generated types for schema constraints such as `enum`, `pattern`, or numeric bounds—always validate untrusted payloads at the ingress boundary.

## 3. Optional Language Core Libraries

When you want standardized `Result`, `RequestContext`, `Dispatcher`, or outbox/idempotency ports, reference the `core/` project in your chosen language (Rust path crate, C# `.csproj` reference, TypeScript package export, or Elixir path dependency).

Keep authentication in your host application and construct tenant context only from verified principals—never trust raw client tenant headers or unverified event payloads.

## 4. Persistence & Distribution Boundaries

Preserve your existing local ACID transaction boundary. Do not claim atomic outbox delivery unless business data and outbox writes share the same database transaction. When extracting a module into a separate service, explicitly design retry, timeout, deduplication (`tenant + operation + idempotency key`), and compensation policies.

## 5. Zero-Dependency Module Adoption (`clrinf module adopt`)

Copy self-contained domain or infrastructure modules (CRM Deals, Contacts, Activities, Authz/Cedar, Caching, Outbox, Idempotency) into any `.csproj`, `Cargo.toml`, or `package.json`:

```bash
# Raw mode: pure source files with zero @clrinf/* or clrinf_* package dependency
clrinf module adopt deals \
  --to-project ./apps/BillingService/BillingService.csproj \
  --target-dir src/Features/Sales \
  --as Sales \
  --mode raw

# Wired mode: registers into native DI / Rust pub mod / TypeScript barrel exports + Cedar policy
clrinf module adopt contacts \
  --to-project ./packages/storefront/package.json \
  --target-dir src/modules/customers \
  --as Customers \
  --mode wired
```

Every adopted module includes a multi-tenant-by-default Cedar policy template under `policies/<module>.cedar`.
