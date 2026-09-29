# C# SDK and generator context

- Target supported .NET 10. The reusable runtime is `src/ClrinfCS.Core`; database
  details belong in `src/ClrinfCS.Adapters.Sqlite`, not the core.
- Local `contracts/schemas` defines wire contracts. Use `ContractJson` for ingress
  and egress; never create competing context, error or CloudEvent DTOs.
- Tenant context must originate at a trusted authentication boundary, not an
  unverified header or event. Storage scopes every key to that context.
- Commit state, inbox identity and outgoing messages in one short local
  transaction. Never perform network calls inside `Execute`. Delivery is
  at-least-once; remote effects and broker acknowledgement are not atomic.
- `examples/Documents` contains demo domain logic, not infrastructure defaults.
- Preserve the specialized CQRS generator and existing public interfaces.
  Its internal FP errors are not the cross-language error envelope.
- Validate with `dotnet test tests/ClrinfCS.Tests/ClrinfCS.Tests.csproj`.
  Set `CLRINF_CONFORMANCE_DIR` to shared fixtures for standalone checkouts.
- Do not stage, commit or push unless the user explicitly authorizes it.
