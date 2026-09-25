# Example application contracts

These are opt-in application contracts, not framework core dependencies.
Authentication here is a credential/JWT/Cedar-specific example; its presence does
not require every application to use that authentication or authorization design.

The move preserves every existing file byte-for-byte, including schema `$id`,
event identities, versions, EARS contents, cache rules and upcaster mappings.
Relative layout within each moved tree is unchanged. Historical references inside
these samples are retained as part of that preservation; resolve old repository
paths using the migration index below.

## Migration index

All paths are relative to the umbrella repository root. Directory rows move the
entire subtree, including commands, events, queries and any upcasters.

| Previous path | Current path |
| --- | --- |
| `clrinf-contracts/schemas/auth/` | `examples/contracts/schemas/auth/` |
| `clrinf-contracts/schemas/eticaret/` | `examples/contracts/schemas/eticaret/` |
| `clrinf-contracts/schemas/kampanya/` | `examples/contracts/schemas/kampanya/` |
| `clrinf-contracts/schemas/katalog/` | `examples/contracts/schemas/katalog/` |
| `clrinf-contracts/schemas/oms/` | `examples/contracts/schemas/oms/` |
| `clrinf-contracts/schemas/stok/` | `examples/contracts/schemas/stok/` |
| `clrinf-contracts/schemas/egitim/` | `examples/contracts/schemas/egitim/` |
| `clrinf-contracts/requirements/auth.ears.md` | `examples/contracts/requirements/auth.ears.md` |
| `clrinf-contracts/requirements/stok.ears.md` | `examples/contracts/requirements/stok.ears.md` |
| `clrinf-contracts/cache-rules/` | `examples/contracts/cache-rules/` (`stok`, `katalog`, `oms`, `eticaret`, `kampanya`, `auth`, `crm`) |

In particular, the existing upcaster is now
`examples/contracts/schemas/kampanya/upcasters/KampanyaOlustur.v1-to-v2.yaml`.
Core `_envelope.schema.json` and `_error.schema.json` are maintained under
`tools/clrinf-codegen/schemas` (and each language's `contracts/schemas`); the new `_context.schema.json` lives beside them.
No domain contracts remain in that core schema directory.

## Validate and generate separately

Run the root canonical CLI (`tools/clrinf-codegen`) from the umbrella root:

```sh
clrinf-codegen check --schema-dir tools/clrinf-codegen/schemas
clrinf-codegen generate --schema-dir tools/clrinf-codegen/schemas --output tests/generated/core

clrinf-codegen check --schema-dir examples/contracts/schemas
clrinf-codegen generate --schema-dir examples/contracts/schemas --output tests/generated/all
```

Omitting `--lang` generates Rust, C#, TypeScript and Elixir subdirectories.
`tests/generated/all` in these commands contains the example set only, not a
merge with core. Do not point application generation at the core output directory.
Update path-based consumers and remove stale generated copies as appropriate;
the generator does not delete unrelated existing files.

`x-domain` selects a generated namespace; it is not a domain-specific framework
dependency. Core metadata uses `common` (also the default when absent).
See the [core contracts](../../clrinf-contracts/README.md) and
[versioning policy](../../clrinf-contracts/VERSIONING.md) for wire stability.
