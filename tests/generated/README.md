# Generated contract fixtures

Core and examples are deliberately separate. `core/` contains only canonical
request context, error and event envelope models. `all/` contains the example
contract catalog. The `rust/src`, `csharp/Generated`, `typescript/src` and `elixir`
copies preserve the existing language fixture paths and now include the full
example catalog (including education).

From the repository root:

```sh
bash tests/generated/regenerate.sh
cargo test --manifest-path tools/clrinf-codegen/Cargo.toml
bash tests/generated/check.sh
```

The language harnesses include the core models and the codegen golden models.
Rust also checks JSON wire round trips, including camel-case names and recursive
array/object shapes. Compilation is not full JSON Schema validation; the native
codegen tests separately validate the shared conformance fixture corpus.

Prerequisites are Rust, .NET 10, TypeScript (`bun install` in `typescript/`),
and Elixir. No message broker or database is required. In CI, after regeneration,
check **both** tracked diffs and new/untracked files to detect drift:

```sh
test -z "$(git status --porcelain -- tests/generated)"
```

Do not use `git diff --exit-code` alone: it ignores newly generated files.
The generator does not delete unrelated files from existing output directories.
