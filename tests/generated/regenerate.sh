#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/../.."

cargo build --quiet --manifest-path tools/clrinf-codegen/Cargo.toml
codegen="$PWD/tools/clrinf-codegen/target/debug/clrinf-codegen"
"$codegen" generate --schema-dir clrinf-contracts/schemas --output tests/generated/core
"$codegen" generate --schema-dir examples/contracts/schemas --output tests/generated/all
"$codegen" generate --schema-dir examples/contracts/schemas --lang rust --output tests/generated/rust/src
"$codegen" generate --schema-dir examples/contracts/schemas --lang csharp --output tests/generated/csharp/Generated
"$codegen" generate --schema-dir examples/contracts/schemas --lang typescript --output tests/generated/typescript/src
"$codegen" generate --schema-dir examples/contracts/schemas --lang elixir --output tests/generated/elixir
"$codegen" generate-upcasters --schema-dir examples/contracts/schemas --output tests/generated/upcasters
