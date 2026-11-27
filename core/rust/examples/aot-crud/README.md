# Rust Compile-Time CRUD Example (`clrinf-aot-crud`)

Demonstrates zero-boilerplate CRUD endpoints in Rust using `#[derive(ClrinfCrud)]` (`clrinf-macros`), `clrinf_core::crud::{BeforeDeleteRule, CrudRuleRunner}`, and `clrinf_adapters::crud::{crud_router, InMemoryCrudStore}`.

Only domain structs (`src/domain/*.rs`) and lifecycle rules (`src/rules/*.rs`) live on disk. Run `clrinfrs crud-map .` to inspect generated vs ejected operations, or `clrinfrs eject Category --operation Update` to eject a single handler into `src/features/category/update.rs`.
