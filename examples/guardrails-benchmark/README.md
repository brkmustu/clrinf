# Guardrails & Context-Cost Benchmark Fixture

Self-contained polyglot workspace (`orders` in Rust, `billing` in TypeScript, `inventory` in C#) used by `scripts/bench-guardrails.ps1` and `scripts/bench-guardrails.sh` to verify:

1. Clean baseline (`clrinf verify` passes with 0 errors).
2. Deterministic context-cost reduction (`clrinf bench context orders.placed.v1`).
3. Deterministic failure detection on 3 canonical AI-agent mistakes:
   - Forbidden infrastructure import in a domain layer (`RULE_FORBIDDEN_DEPENDENCY`, `scope = "imports"`).
   - Direct outbox bypass (`DRIFT_RAW_OUTBOX_WRITE`).
   - Breaking v2 event schema without an upcaster (`TOPOLOGY_UPCASTER_MISSING`).
4. Post-edit Claude Code hook feedback (`clrinf hook run --format claude` returning exit code `2`).
