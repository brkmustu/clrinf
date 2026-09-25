#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/../.."

cargo test --quiet --manifest-path tests/generated/rust/Cargo.toml
dotnet build tests/generated/csharp/GeneratedTests.csproj --nologo --verbosity quiet
bun run --cwd tests/generated/typescript typecheck
beam_output="$(mktemp -d)"
trap 'find "$beam_output" -maxdepth 1 -type f -name "*.beam" -delete; rmdir "$beam_output"' EXIT
elixirc --warnings-as-errors -o "$beam_output" \
  tests/generated/core/elixir/*.ex \
  tests/generated/all/elixir/*.ex \
  tools/clrinf-codegen/tests/golden/elixir/*.ex
elixir -pa "$beam_output" tests/generated/elixir_wire.exs
