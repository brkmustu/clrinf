// @clrinf:generated — Rust Module Scaffolding Generator
// Generates module domain structures, handlers, rules directory,
// and multi-tenant Cedar policy file.

use std::fs;
use std::io::Result;
use std::path::Path;

pub struct ModuleGenerator;

impl ModuleGenerator {
    pub fn generate(project_root: &Path, module_name: &str) -> Result<Vec<String>> {
        let slug = module_name.to_lowercase();
        let pascal = to_pascal_case(module_name);
        let mut created_files = Vec::new();

        // 1. Determine modules directory based on project architecture (Flat, Layered, CleanCQRS)
        let modules_root = if project_root.join("src").exists() {
            let dir = project_root.join("src").join("modules");
            fs::create_dir_all(&dir)?;
            dir
        } else if project_root.join("crates").exists() {
            let mut target_dir = None;
            if let Ok(entries) = fs::read_dir(project_root.join("crates")) {
                for entry in entries.flatten() {
                    let p = entry.path();
                    if p.is_dir() {
                        let name = p.file_name().and_then(|n| n.to_str()).unwrap_or("");
                        if name.ends_with("-core") || name.ends_with("-app") {
                            let src_mod = p.join("src").join("modules");
                            let _ = fs::create_dir_all(&src_mod);
                            target_dir = Some(src_mod);
                            break;
                        }
                    }
                }
            }
            target_dir.unwrap_or_else(|| {
                let dir = project_root.join("modules");
                let _ = fs::create_dir_all(&dir);
                dir
            })
        } else {
            let dir = project_root.join("modules");
            fs::create_dir_all(&dir)?;
            dir
        };

        let module_dir = modules_root.join(&slug);
        let rules_dir = module_dir.join("rules");
        fs::create_dir_all(&rules_dir)?;

        // 2. Create models.rs
        let models_path = module_dir.join("models.rs");
        if !models_path.exists() {
            let content = format!(
                r#"// @clrinf:generated — {pascal} domain models
use serde::{{Deserialize, Serialize}};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct {pascal} {{
    pub id: String,
    pub tenant_id: String,
    pub created_at: i64,
}}
"#
            );
            fs::write(&models_path, content)?;
            created_files.push(models_path.display().to_string());
        }

        // 3. Create handlers.rs
        let handlers_path = module_dir.join("handlers.rs");
        if !handlers_path.exists() {
            let content = format!(
                r#"// @clrinf:generated — {pascal} command and query handlers
use super::models::{pascal};

/// Operation Claim representation for multi-tenant authorization
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OperationClaim {{
    pub resource: &'static str,
    pub action: &'static str,
    pub resource_tenant_id: Option<String>,
}}

impl OperationClaim {{
    pub fn new(resource: &'static str, action: &'static str) -> Self {{
        Self {{
            resource,
            action,
            resource_tenant_id: None,
        }}
    }}

    pub fn with_tenant(resource: &'static str, action: &'static str, tenant_id: impl Into<String>) -> Self {{
        Self {{
            resource,
            action,
            resource_tenant_id: Some(tenant_id.into()),
        }}
    }}
}}

pub struct {pascal}Handlers;

impl {pascal}Handlers {{
    pub fn create_claim() -> OperationClaim {{
        OperationClaim::new("{slug}", "create")
    }}

    pub fn read_claim() -> OperationClaim {{
        OperationClaim::new("{slug}", "read")
    }}

    pub fn update_claim() -> OperationClaim {{
        OperationClaim::new("{slug}", "update")
    }}

    pub fn delete_claim() -> OperationClaim {{
        OperationClaim::new("{slug}", "delete")
    }}
}}
"#
            );
            fs::write(&handlers_path, content)?;
            created_files.push(handlers_path.display().to_string());
        }

        // 4. Create rules/mod.rs
        let rules_mod_path = rules_dir.join("mod.rs");
        if !rules_mod_path.exists() {
            let content = format!(
                r#"// @clrinf:generated — {pascal} business rules
// Export rule structs and validation pipes here.
"#
            );
            fs::write(&rules_mod_path, content)?;
            created_files.push(rules_mod_path.display().to_string());
        }

        // 5. Create module mod.rs
        let mod_rs_path = module_dir.join("mod.rs");
        if !mod_rs_path.exists() {
            let content = format!(
                r#"// @clrinf:generated — {pascal} module entry point
pub mod handlers;
pub mod models;
pub mod rules;

pub use handlers::{pascal}Handlers;
pub use models::{pascal};
"#
            );
            fs::write(&mod_rs_path, content)?;
            created_files.push(mod_rs_path.display().to_string());
        }

        // 6. Update or create modules_root/mod.rs
        let parent_mod = modules_root.join("mod.rs");
        let export_line = format!("pub mod {slug};\n");
        if parent_mod.exists() {
            let existing = fs::read_to_string(&parent_mod)?;
            if !existing.contains(&export_line) {
                fs::write(&parent_mod, format!("{existing}{export_line}"))?;
            }
        } else {
            fs::write(&parent_mod, format!("// @clrinf:generated — Application modules\n{export_line}"))?;
            created_files.push(parent_mod.display().to_string());
        }

        // 7. Create Multi-Tenant Cedar Policy: policies/<slug>.cedar
        let policies_dir = project_root.join("policies");
        fs::create_dir_all(&policies_dir)?;
        let policy_path = policies_dir.join(format!("{slug}.cedar"));
        if !policy_path.exists() {
            let cedar_content = format!(
                r#"// @clrinf:generated — Multi-Tenant Cedar Policy: {pascal}
// Generated by clrinfrs scaffold

// 1. PlatformAdmin: Kiracılar arası tam denetim yetkisi
permit (
    principal in Role::"PlatformAdmin",
    action,
    resource
);

// 2. TenantAdmin: Kendi kiracısındaki kaynaklar üzerinde tam yetki
permit (
    principal in Role::"TenantAdmin",
    action,
    resource in ResourceType::"{pascal}"
) when {{
    resource.tenant_id == principal.tenant_id
}};

// 3. Modül Yöneticisi: Kendi kiracısında tam CRUD yetkisi
permit (
    principal in Role::"{pascal}Manager",
    action in [
        Action::"{slug}.create",
        Action::"{slug}.read",
        Action::"{slug}.update",
        Action::"{slug}.delete"
    ],
    resource in ResourceType::"{pascal}"
) when {{
    context.tenant_id == principal.tenant_id &&
    resource.tenant_id == principal.tenant_id
}};

// 4. Okuma Rolü: Kendi kiracısında okuma yetkisi
permit (
    principal in Role::"{pascal}Reader",
    action == Action::"{slug}.read",
    resource in ResourceType::"{pascal}"
) when {{
    context.tenant_id == principal.tenant_id &&
    resource.tenant_id == principal.tenant_id
}};

// 5. Kesin Multi-Tenant İzolasyon Muhafızı (Forbid Guard):
// Kiracı uyuşmazlığı durumunda tüm izin kurallarını ezer ve erişimi engeller
forbid (
    principal,
    action,
    resource in ResourceType::"{pascal}"
) when {{
    resource.tenant_id != principal.tenant_id
}} unless {{
    principal in Role::"PlatformAdmin"
}};
"#
            );
            fs::write(&policy_path, cedar_content)?;
            created_files.push(policy_path.display().to_string());
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
