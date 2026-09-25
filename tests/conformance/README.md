# Shared wire-format conformance

All four language cores consume the same files:

- `fixtures/valid.json`: one context, standard error and CloudEvent.
- `fixtures/invalid.json`: context/error/event arrays of `{name, value}` rejection cases.

Use `CLRINF_CONFORMANCE_DIR` to point standalone submodule tests at this directory's
`fixtures` folder. A missing fixture must fail; tests must not silently fall back
to unrelated local values.

The code generator's schema tests validate these samples against the canonical
JSON Schema, and language tests validate/round-trip the same samples. A pass
covers these cases, not every possible JSON Schema constraint or identical
runtime semantics. Property order and equivalent timestamp formatting need not
be byte-identical. Extension fields must not change meaning.

Storage durability, dispatcher behavior and NATS integration have separate tests.
Sharing the same event format does not turn volatile adapters into durable stores.

From the umbrella root, CI exports:

```bash
export CLRINF_CONFORMANCE_DIR="$PWD/tests/conformance/fixtures"
```

Then each submodule runs its native suite as documented in its own README.
