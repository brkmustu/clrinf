// @clrinf:generated — Rust Entity Scaffolding Generator
// Generates entity domain model, repository port, CRUD handlers,
// and tenant-isolated operations.

use std::fs;
use std::io::Result;
use std::path::Path;

pub struct EntityGenerator;

impl EntityGenerator {
    pub fn generate(
        project_root: &Path,
        module_name: &str,
        entity_name: &str,
        properties: &[(String, String)],
    ) -> Result<Vec<String>> {
        let mod_slug = module_name.to_lowercase();
        let entity_slug = entity_name.to_lowercase();
        let entity_pascal = to_pascal_case(entity_name);
        let mut created_files = Vec::new();

        // 1. Locate or create module directory
        let module_dir = find_module_dir(project_root, &mod_slug).unwrap_or_else(|| {
            let _ = super::module::ModuleGenerator::generate(project_root, module_name);
            find_module_dir(project_root, &mod_slug)
                .unwrap_or_else(|| project_root.join("modules").join(&mod_slug))
        });

        // 2. Build struct fields from properties
        let mut struct_fields = String::new();
        for (prop_name, prop_type) in properties {
            let field_name = prop_name.to_lowercase();
            let lower = prop_type.to_lowercase();
            let rust_type: &str = match lower.as_str() {
                "string" | "str" | "text" => "String",
                "int" | "i32" | "integer" => "i32",
                "i64" | "long" => "i64",
                "f64" | "float" | "double" | "decimal" => "f64",
                "bool" | "boolean" => "bool",
                _ => prop_type.as_str(),
            };
            struct_fields.push_str(&format!("    pub {field_name}: {rust_type},\n"));
        }

        // 3. Create or update entity file: src/modules/<mod_slug>/<entity_slug>.rs
        let entity_file = module_dir.join(format!("{entity_slug}.rs"));
        let content = format!(
            r#"// @clrinf:generated — {entity_pascal} Entity, Repository Port, and CRUD Operations
use async_trait::async_trait;
use super::handlers::OperationClaim;
use serde::{{Deserialize, Serialize}};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// {entity_pascal} Domain Entity
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct {entity_pascal} {{
    pub id: String,
    pub tenant_id: String,
{struct_fields}    pub created_at: i64,
    pub updated_at: i64,
}}

/// {entity_pascal} Repository Port (Data Access Abstraction)
#[async_trait]
pub trait {entity_pascal}Repository: Send + Sync + 'static {{
    async fn get_by_id(&self, tenant_id: &str, id: &str) -> Result<Option<{entity_pascal}>, String>;
    async fn list_by_tenant(&self, tenant_id: &str) -> Result<Vec<{entity_pascal}>, String>;
    async fn insert(&self, entity: {entity_pascal}) -> Result<{entity_pascal}, String>;
    async fn update(&self, entity: {entity_pascal}) -> Result<{entity_pascal}, String>;
    async fn delete(&self, tenant_id: &str, id: &str) -> Result<bool, String>;
}}

/// In-Memory {entity_pascal} Repository Implementation
pub struct InMemory{entity_pascal}Repository {{
    storage: Arc<RwLock<HashMap<String, {entity_pascal}>>>,
}}

impl InMemory{entity_pascal}Repository {{
    pub fn new() -> Self {{
        Self {{
            storage: Arc::new(RwLock::new(HashMap::new())),
        }}
    }}

    fn storage_key(tenant_id: &str, id: &str) -> String {{
        format!("{{tenant_id}}:{{id}}")
    }}
}}

impl Default for InMemory{entity_pascal}Repository {{
    fn default() -> Self {{
        Self::new()
    }}
}}

#[async_trait]
impl {entity_pascal}Repository for InMemory{entity_pascal}Repository {{
    async fn get_by_id(&self, tenant_id: &str, id: &str) -> Result<Option<{entity_pascal}>, String> {{
        let key = Self::storage_key(tenant_id, id);
        let store = self.storage.read().await;
        Ok(store.get(&key).cloned())
    }}

    async fn list_by_tenant(&self, tenant_id: &str) -> Result<Vec<{entity_pascal}>, String> {{
        let store = self.storage.read().await;
        let items: Vec<{entity_pascal}> = store
            .values()
            .filter(|e| e.tenant_id == tenant_id)
            .cloned()
            .collect();
        Ok(items)
    }}

    async fn insert(&self, entity: {entity_pascal}) -> Result<{entity_pascal}, String> {{
        let key = Self::storage_key(&entity.tenant_id, &entity.id);
        let mut store = self.storage.write().await;
        store.insert(key, entity.clone());
        Ok(entity)
    }}

    async fn update(&self, entity: {entity_pascal}) -> Result<{entity_pascal}, String> {{
        let key = Self::storage_key(&entity.tenant_id, &entity.id);
        let mut store = self.storage.write().await;
        store.insert(key, entity.clone());
        Ok(entity)
    }}

    async fn delete(&self, tenant_id: &str, id: &str) -> Result<bool, String> {{
        let key = Self::storage_key(tenant_id, id);
        let mut store = self.storage.write().await;
        Ok(store.remove(&key).is_some())
    }}
}}

/// Multi-Tenant Operation Claims for {entity_pascal}
pub struct {entity_pascal}Claims;

impl {entity_pascal}Claims {{
    pub fn create(tenant_id: &str) -> OperationClaim {{
        OperationClaim::with_tenant("{entity_slug}", "create", tenant_id)
    }}

    pub fn read(tenant_id: &str) -> OperationClaim {{
        OperationClaim::with_tenant("{entity_slug}", "read", tenant_id)
    }}

    pub fn update(tenant_id: &str) -> OperationClaim {{
        OperationClaim::with_tenant("{entity_slug}", "update", tenant_id)
    }}

    pub fn delete(tenant_id: &str) -> OperationClaim {{
        OperationClaim::with_tenant("{entity_slug}", "delete", tenant_id)
    }}
}}
"#
        );

        fs::write(&entity_file, content)?;
        created_files.push(entity_file.display().to_string());

        // 4. Register in module's mod.rs
        let mod_rs = module_dir.join("mod.rs");
        let export_line = format!("pub mod {entity_slug};\npub use {entity_slug}::*;\n");
        if mod_rs.exists() {
            let existing = fs::read_to_string(&mod_rs)?;
            if !existing.contains(&format!("pub mod {entity_slug};")) {
                fs::write(&mod_rs, format!("{existing}\n{export_line}"))?;
            }
        }

        Ok(created_files)
    }
}

fn to_pascal_case(s: &str) -> String {
    s.split(|c: char| c == '_' || c == '-' || c == ' ')
        .filter(|part| !part.is_empty())
        .map(|part| {
            let mut chars = part.chars();
            match chars.next() {
                Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
                None => String::new(),
            }
        })
        .collect()
}

fn find_module_dir(project_root: &Path, mod_slug: &str) -> Option<std::path::PathBuf> {
    let p = project_root.join("src").join("modules").join(mod_slug);
    if p.exists() {
        return Some(p);
    }
    if let Ok(entries) = fs::read_dir(project_root.join("crates")) {
        for entry in entries.flatten() {
            let p = entry.path().join("src").join("modules").join(mod_slug);
            if p.exists() {
                return Some(p);
            }
        }
    }
    let p = project_root.join("modules").join(mod_slug);
    if p.exists() {
        return Some(p);
    }
    None
}
