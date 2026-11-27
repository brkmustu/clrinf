use clrinf_macros::ClrinfCrud;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, ClrinfCrud)]
#[clrinf_crud(route = "/api/brands")]
pub struct Brand {
    pub id: String,
    pub name: String,
    #[crud_ignore]
    pub tenant_id: String,
}
