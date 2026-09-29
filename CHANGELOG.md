# Changelog

All notable changes are documented here.
Format: [Keep a Changelog](https://keepachangelog.com/en/1.1.0/).

---

## [Unreleased]

### Added
- Contract-first multi-language framework (Rust, C#, TypeScript, Elixir)
- `clrinf-codegen` CLI: schema validation, multi-language codegen, topology validation, Meta MCP server
- Cross-service Pub/Sub topology validation (TOPOLOGY_DEAD_EVENT, TOPOLOGY_ORPHAN_SUBSCRIBER, TOPOLOGY_UPCASTER_MISSING)
- Idiomatic business rule engines per language
- Static architecture linters: Roslyn (ARCH001-ARCH004), Syn AST (RUST_ARCH001), TS Compiler API (ARCH_TS_001)
- Module catalog and adoption engine (`clrinf catalog`, `clrinf module adopt`)
- Cedar ABAC multi-tenant authorization
- Meta MCP Gateway (JSON-RPC 2.0 stdio) for AI coding agents
- Elixir live streaming bridge adapter
- Conformance test suite for cross-language wire format validation
