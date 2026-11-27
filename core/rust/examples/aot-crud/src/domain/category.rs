use clrinf_macros::ClrinfCrud;
use serde::{Deserialize, Serialize};

/// Standard parameter table. DTOs, store wiring, and Axum routes are expanded
/// at compile time via `#[derive(ClrinfCrud)]`; only this entity lives on disk.
#[derive(Debug, Clone, Serialize, Deserialize, ClrinfCrud)]
#[clrinf_crud(route = "/api/categories")]
pub struct Category {
    pub id: String,
    pub name: String,
    pub slug: String,
    /// Tenant comes from `RequestContext`, never from client DTOs.
    #[crud_ignore]
    pub tenant_id: String,
}
