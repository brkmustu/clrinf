// @clrinf:generated — C# Security Module code generation
//
// Ports AddSecurityCommand from ClrinfCS to native Rust clrinf-codegen.
// Generates JWT, Password Hashing, User/Role/Claim Entities, Repositories,
// Commands (Login, Register), and injects DbSets and Service Registrations.

use crate::csharp::injector;
use crate::manifest::{ArchStyle, ProjectManifest};
use anyhow::{Context, Result};
use std::fs;
use std::path::{Path, PathBuf};
use tera::{Context as TeraContext, Tera};

/// Security mode: Basic (JWT + Refresh Token) or Advanced (JWT + 2FA/OTP/Email)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SecurityMode {
    Basic,
    Advanced,
}

impl Default for SecurityMode {
    fn default() -> Self {
        SecurityMode::Basic
    }
}

impl SecurityMode {
    pub fn as_str(&self) -> &'static str {
        match self {
            SecurityMode::Basic => "basic",
            SecurityMode::Advanced => "advanced",
        }
    }

    #[allow(dead_code)]
    pub fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "advanced" => SecurityMode::Advanced,
            _ => SecurityMode::Basic,
        }
    }
}

pub struct AddSecurityOptions<'a> {
    pub project_path: &'a Path,
    pub manifest: &'a ProjectManifest,
    pub templates_dir: &'a Path,
    pub mode: SecurityMode,
}

pub struct AddSecurityResult {
    pub files_created: Vec<PathBuf>,
    pub files_modified: Vec<PathBuf>,
}

/// Generates Security module into the target C# project
pub fn add_security(opts: &AddSecurityOptions) -> Result<AddSecurityResult> {
    let mode_str = opts.mode.as_str();
    let security_tpl_dir = opts.templates_dir
        .join("csharp")
        .join("security")
        .join(mode_str);

    let sec_dir = if security_tpl_dir.is_dir() {
        security_tpl_dir
    } else {
        // Fallback to tools/clrinf-codegen/templates/csharp/security/<mode>
        opts.templates_dir
            .join("security")
            .join(mode_str)
    };

    if !sec_dir.is_dir() {
        anyhow::bail!(
            "Security templates directory not found at: {}",
            sec_dir.display()
        );
    }

    let glob_pattern = format!("{}/**/*.tera", sec_dir.display());
    let tera = Tera::new(&glob_pattern)
        .with_context(|| format!("Failed to load security templates from {}", sec_dir.display()))?;

    let project_name = &opts.manifest.project.name;
    let project_namespace = opts.manifest.project.namespace
        .as_deref()
        .unwrap_or(project_name);
    let db_context_name = opts.manifest.project.db_context
        .as_deref()
        .unwrap_or("BaseDbContext");

    let mut ctx = TeraContext::new();
    ctx.insert("project_name", project_name);
    ctx.insert("project_namespace", project_namespace);
    ctx.insert("db_context_name", db_context_name);

    let mut files_created = Vec::new();
    let mut files_modified = Vec::new();

    // Iterate through all files in sec_dir
    for entry in walkdir::WalkDir::new(&sec_dir) {
        let entry = entry?;
        if !entry.file_type().is_file() {
            continue;
        }

        let path = entry.path();
        if path.extension().and_then(|s| s.to_str()) != Some("tera") {
            continue;
        }

        let rel_path = path.strip_prefix(&sec_dir)?;
        let tpl_name = rel_path.to_str().unwrap();

        // Strip .tera extension to get target relative C# file path
        let target_rel_file = rel_path.with_extension("");

        // Map target path based on layer structure
        let target_dest_path = resolve_target_file_path(
            opts.project_path,
            opts.manifest,
            &target_rel_file,
        )?;

        if target_dest_path.exists() {
            continue;
        }

        if let Some(parent) = target_dest_path.parent() {
            fs::create_dir_all(parent)?;
        }

        let rendered = tera.render(tpl_name, &ctx)
            .with_context(|| format!("Failed to render security template {}", tpl_name))?;

        fs::write(&target_dest_path, &rendered)
            .with_context(|| format!("Failed to write {}", target_dest_path.display()))?;

        println!("  ✅ {}", target_dest_path.display());
        files_created.push(target_dest_path);
    }

    // Inject DbSets into DbContext
    let persistence_layer = match opts.manifest.project.arch {
        ArchStyle::Flat => project_name.clone(),
        ArchStyle::Layered => format!("{}.Core", project_name),
        ArchStyle::CleanCqrs => "Persistence".to_string(),
    };

    if let Some(ctx_path) = injector::find_dbcontext(opts.project_path, &persistence_layer, db_context_name) {
        let using_entities = format!("using {}.Domain.Entities;", project_namespace);
        let _ = injector::add_using(&ctx_path, &using_entities);

        let sec_entities = [
            ("User", "Users"),
            ("OperationClaim", "OperationClaims"),
            ("UserOperationClaim", "UserOperationClaims"),
            ("RefreshToken", "RefreshTokens"),
        ];

        let mut injected_any = false;
        for (entity, plural) in &sec_entities {
            if injector::add_dbset_property(&ctx_path, entity, plural)? {
                injected_any = true;
            }
        }

        if opts.mode == SecurityMode::Advanced {
            let _ = injector::add_dbset_property(&ctx_path, "OtpAuthenticator", "OtpAuthenticators");
            let _ = injector::add_dbset_property(&ctx_path, "EmailAuthenticator", "EmailAuthenticators");
            injected_any = true;
        }

        if injected_any {
            println!("  ✅ Security DbSets injected into {}", ctx_path.display());
            files_modified.push(ctx_path);
        }
    }

    Ok(AddSecurityResult {
        files_created,
        files_modified,
    })
}

/// Resolves target file path inside project structure
fn resolve_target_file_path(
    project_path: &Path,
    manifest: &ProjectManifest,
    rel_path: &Path,
) -> Result<PathBuf> {
    let src = project_path.join("src");
    let base = if src.is_dir() { src } else { project_path.to_path_buf() };

    match manifest.project.arch {
        ArchStyle::CleanCqrs => {
            // rel_path starts with Domain/, Application/, Infrastructure/, Persistence/, Api/
            Ok(base.join(rel_path))
        }
        ArchStyle::Layered => {
            let p_name = &manifest.project.name;
            let components: Vec<_> = rel_path.iter().collect();
            if components.is_empty() {
                return Ok(base.join(rel_path));
            }
            let first = components[0].to_str().unwrap_or("");
            let rest: PathBuf = components.iter().skip(1).collect();

            let target_sub = match first {
                "Domain" | "Application" => base.join(format!("{}.Core", p_name)).join(rel_path),
                "Infrastructure" => base.join(format!("{}.Infrastructure", p_name)).join(&rest),
                "Persistence" => base.join(format!("{}.Persistence", p_name)).join(&rest),
                "Api" => base.join(format!("{}.Api", p_name)).join(&rest),
                _ => base.join(rel_path),
            };
            Ok(target_sub)
        }
        ArchStyle::Flat => {
            let p_name = &manifest.project.name;
            let flat_dir = if base.join(p_name).is_dir() {
                base.join(p_name)
            } else {
                base
            };
            Ok(flat_dir.join(rel_path))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_add_security_basic() {
        let tmp = tempdir().unwrap();
        let project_dir = tmp.path().join("CrmApp");
        let src_dir = project_dir.join("src");
        let persistence_ctx = src_dir.join("Persistence").join("Contexts");
        fs::create_dir_all(&persistence_ctx).unwrap();

        let db_context_file = persistence_ctx.join("BaseDbContext.cs");
        fs::write(
            &db_context_file,
            "namespace CrmApp.Persistence.Contexts;\n\npublic class BaseDbContext : DbContext\n{\n}\n",
        ).unwrap();

        let manifest_content = r#"
[project]
name = "CrmApp"
lang = "csharp"
arch = "clean-cqrs"
secured = true
"#;
        let manifest = ProjectManifest::from_toml(manifest_content).unwrap();
        let templates_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("templates");

        let opts = AddSecurityOptions {
            project_path: &project_dir,
            manifest: &manifest,
            templates_dir: &templates_dir,
            mode: SecurityMode::Basic,
        };

        let result = add_security(&opts).unwrap();
        assert!(!result.files_created.is_empty());
        assert!(!result.files_modified.is_empty());

        // Verify User entity was created
        let user_entity = src_dir.join("Domain").join("Entities").join("User.cs");
        assert!(user_entity.is_file(), "User.cs should be created at {}", user_entity.display());
        let user_content = fs::read_to_string(user_entity).unwrap();
        assert!(user_content.contains("public class User"));
        assert!(user_content.contains("CrmApp.Domain.Entities"));

        // Verify DbSets injected
        let ctx_content = fs::read_to_string(db_context_file).unwrap();
        assert!(ctx_content.contains("DbSet<User> Users"));
        assert!(ctx_content.contains("DbSet<OperationClaim> OperationClaims"));
    }
}
