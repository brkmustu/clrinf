// @clrinf:generated — Cross-Cutting Concern Module Manager for clrinf
//
// Zero-friction module lifecycle:
// • Add: Enables module in manifest, renders language-idiomatic templates, wires project.
// • Remove: Disables module in manifest, safely cleans up generated files, unwires project.
// • Sync: Reconciles manifest state with project files deterministically.

use crate::manifest::{
    AuthProvider, AuthzProvider, CachingProvider, IdempotencyProvider, LoggingProvider,
    OutboxProvider, ProjectManifest, TransactionProvider,
};
use crate::module_wiring::ModuleWiring;
use anyhow::{bail, Context, Result};
use std::fs;
use std::path::{Path, PathBuf};
use tera::{Context as TeraContext, Tera};

#[derive(Debug, Default)]
pub struct SyncReport {
    pub synced: Vec<String>,
    pub cleaned: Vec<String>,
}

pub struct ModuleManager {
    #[allow(dead_code)]
    templates_dir: PathBuf,
    tera: Tera,
}

impl ModuleManager {
    pub fn new(templates_dir: Option<PathBuf>) -> Result<Self> {
        let dir = match templates_dir {
            Some(d) => d,
            None => Self::detect_templates_dir()?,
        };

        let template_pattern = format!("{}/**/*.tera", dir.display());
        let tera = Tera::new(&template_pattern).with_context(|| {
            format!("Failed to initialize Tera from templates dir: {}", dir.display())
        })?;

        Ok(Self {
            templates_dir: dir,
            tera,
        })
    }

    #[allow(dead_code)]
    pub fn templates_dir(&self) -> &Path {
        &self.templates_dir
    }

    fn detect_templates_dir() -> Result<PathBuf> {
        if let Ok(env_dir) = std::env::var("CLRINF_TEMPLATES_DIR") {
            let p = PathBuf::from(env_dir);
            if p.is_dir() {
                return Ok(p);
            }
        }

        if let Ok(cwd) = std::env::current_dir() {
            let mut current = Some(cwd.as_path());
            while let Some(dir) = current {
                let codegen_templates = dir.join("tools/clrinf-codegen/templates");
                if codegen_templates.is_dir() {
                    return Ok(codegen_templates);
                }
                let local_templates = dir.join("templates");
                if local_templates.is_dir() && local_templates.join("csharp").is_dir() {
                    return Ok(local_templates);
                }
                current = dir.parent();
            }
        }

        if let Ok(exe) = std::env::current_exe() {
            let mut current = exe.parent();
            while let Some(dir) = current {
                let codegen_templates = dir.join("tools/clrinf-codegen/templates");
                if codegen_templates.is_dir() {
                    return Ok(codegen_templates);
                }
                let local_templates = dir.join("templates");
                if local_templates.is_dir() && local_templates.join("csharp").is_dir() {
                    return Ok(local_templates);
                }
                current = dir.parent();
            }
        }

        bail!("Templates directory not found. Set CLRINF_TEMPLATES_DIR or run from clrinf workspace.")
    }

    /// Add / enable a cross-cutting module and render its templates.
    pub fn add_module(
        &self,
        manifest: &mut ProjectManifest,
        manifest_path: &Path,
        module_name: &str,
        provider: Option<&str>,
    ) -> Result<Vec<PathBuf>> {
        let project_dir = manifest_path.parent().unwrap_or_else(|| Path::new("."));

        // 1. Update manifest configuration
        self.apply_module_config(manifest, module_name, provider, true)?;
        manifest.save(manifest_path)?;

        // 2. Render templates for this module and language
        let created_files = self.render_module_templates(manifest, project_dir, module_name)?;

        // 3. Wire module into language-specific project entry (skip if custom)
        if !Self::is_module_custom(manifest, module_name) {
            ModuleWiring::wire(project_dir, manifest.project.lang.as_str(), module_name)?;
        }

        Ok(created_files)
    }

    /// Remove / disable a cross-cutting module, clean up generated files and unwire.
    pub fn remove_module(
        &self,
        manifest: &mut ProjectManifest,
        manifest_path: &Path,
        module_name: &str,
    ) -> Result<Vec<PathBuf>> {
        let project_dir = manifest_path.parent().unwrap_or_else(|| Path::new("."));

        if module_name == "error_handling" || module_name == "validation" {
            bail!("'{}' is a fundamental module and cannot be disabled.", module_name);
        }

        let was_custom = Self::is_module_custom(manifest, module_name);

        // 1. Update manifest configuration
        self.apply_module_config(manifest, module_name, None, false)?;
        manifest.save(manifest_path)?;

        // 2. Unwire module from language entry (skip if custom)
        if !was_custom {
            ModuleWiring::unwire(project_dir, manifest.project.lang.as_str(), module_name)?;
        }

        // 3. Remove generated files for this module (skip if custom)
        let removed_files = if was_custom {
            Vec::new()
        } else {
            self.cleanup_module_files(manifest, project_dir, module_name)?
        };

        Ok(removed_files)
    }

    /// Reconciles project files and wiring with manifest state.
    pub fn sync_modules(
        &self,
        manifest: &ProjectManifest,
        project_dir: &Path,
    ) -> Result<SyncReport> {
        let mut report = SyncReport::default();
        let all_modules = [
            "caching",
            "logging",
            "transaction",
            "authentication",
            "authorization",
            "idempotency",
            "outbox",
        ];

        for &mod_name in &all_modules {
            let is_enabled = match mod_name {
                "caching" => manifest.modules.caching.enabled,
                "logging" => manifest.modules.logging.enabled,
                "transaction" => manifest.modules.transaction.enabled,
                "authentication" => manifest.modules.authentication.enabled,
                "authorization" => manifest.modules.authorization.enabled,
                "idempotency" => manifest.modules.idempotency.enabled,
                "outbox" => manifest.modules.outbox.enabled,
                _ => false,
            };

            if Self::is_module_custom(manifest, mod_name) {
                if is_enabled {
                    report.synced.push(mod_name.to_string());
                }
                continue;
            }

            if is_enabled {
                self.render_module_templates(manifest, project_dir, mod_name)?;
                ModuleWiring::wire(project_dir, manifest.project.lang.as_str(), mod_name)?;
                report.synced.push(mod_name.to_string());
            } else {
                let cleaned = self.cleanup_module_files(manifest, project_dir, mod_name)?;
                if !cleaned.is_empty() {
                    report.cleaned.push(mod_name.to_string());
                }
                ModuleWiring::unwire(project_dir, manifest.project.lang.as_str(), mod_name)?;
            }
        }

        Ok(report)
    }

    fn apply_module_config(
        &self,
        m: &mut ProjectManifest,
        module_name: &str,
        provider: Option<&str>,
        enable: bool,
    ) -> Result<()> {
        match module_name.to_lowercase().as_str() {
            "caching" => {
                m.modules.caching.enabled = enable;
                if let Some(p) = provider {
                    m.modules.caching.provider = match p.to_lowercase().as_str() {
                        "memory" => CachingProvider::Memory,
                        "redis" => CachingProvider::Redis,
                        "distributed" => CachingProvider::Distributed,
                        "custom" => CachingProvider::Custom,
                        _ => bail!("Unsupported caching provider: {}", p),
                    };
                }
            }
            "logging" => {
                m.modules.logging.enabled = enable;
                if let Some(p) = provider {
                    m.modules.logging.provider = match p.to_lowercase().as_str() {
                        "structured" => LoggingProvider::Structured,
                        "console" => LoggingProvider::Console,
                        "opentelemetry" => LoggingProvider::Opentelemetry,
                        "custom" => LoggingProvider::Custom,
                        _ => bail!("Unsupported logging provider: {}", p),
                    };
                }
            }
            "transaction" => {
                m.modules.transaction.enabled = enable;
                if let Some(p) = provider {
                    m.modules.transaction.provider = match p.to_lowercase().as_str() {
                        "native" => TransactionProvider::Native,
                        "saga" => TransactionProvider::Saga,
                        "custom" => TransactionProvider::Custom,
                        _ => bail!("Unsupported transaction provider: {}", p),
                    };
                }
            }
            "authentication" | "auth" => {
                m.modules.authentication.enabled = enable;
                if let Some(p) = provider {
                    m.modules.authentication.provider = match p.to_lowercase().as_str() {
                        "jwt" => AuthProvider::Jwt,
                        "oauth2" => AuthProvider::Oauth2,
                        "apikey" => AuthProvider::Apikey,
                        "custom" => AuthProvider::Custom,
                        _ => bail!("Unsupported auth provider: {}", p),
                    };
                }
            }
            "authorization" | "authz" => {
                m.modules.authorization.enabled = enable;
                if let Some(p) = provider {
                    m.modules.authorization.provider = match p.to_lowercase().as_str() {
                        "policy" => AuthzProvider::Policy,
                        "rbac" => AuthzProvider::Rbac,
                        "abac" => AuthzProvider::Abac,
                        "cedar" => AuthzProvider::Cedar,
                        "custom" => AuthzProvider::Custom,
                        _ => bail!("Unsupported authz provider: {}", p),
                    };
                }
            }
            "idempotency" => {
                m.modules.idempotency.enabled = enable;
                if let Some(p) = provider {
                    m.modules.idempotency.provider = match p.to_lowercase().as_str() {
                        "memory" => IdempotencyProvider::Memory,
                        "redis" => IdempotencyProvider::Redis,
                        "database" => IdempotencyProvider::Database,
                        "custom" => IdempotencyProvider::Custom,
                        _ => bail!("Unsupported idempotency provider: {}", p),
                    };
                }
            }
            "outbox" => {
                m.modules.outbox.enabled = enable;
                if let Some(p) = provider {
                    m.modules.outbox.provider = match p.to_lowercase().as_str() {
                        "database" => OutboxProvider::Database,
                        "memory" => OutboxProvider::Memory,
                        "custom" => OutboxProvider::Custom,
                        _ => bail!("Unsupported outbox provider: {}", p),
                    };
                }
            }
            _ => bail!(
                "Unknown module: '{}'. Supported: caching, logging, transaction, authentication, authorization, idempotency, outbox",
                module_name
            ),
        }
        Ok(())
    }

    fn is_module_custom(manifest: &ProjectManifest, module_name: &str) -> bool {
        match module_name {
            "caching" => manifest.modules.caching.provider == CachingProvider::Custom,
            "logging" => manifest.modules.logging.provider == LoggingProvider::Custom,
            "transaction" => manifest.modules.transaction.provider == TransactionProvider::Custom,
            "authentication" | "auth" => manifest.modules.authentication.provider == AuthProvider::Custom,
            "authorization" | "authz" => manifest.modules.authorization.provider == AuthzProvider::Custom,
            "idempotency" => manifest.modules.idempotency.provider == IdempotencyProvider::Custom,
            "outbox" => manifest.modules.outbox.provider == OutboxProvider::Custom,
            _ => false,
        }
    }

    fn render_module_templates(
        &self,
        manifest: &ProjectManifest,
        project_dir: &Path,
        module_name: &str,
    ) -> Result<Vec<PathBuf>> {
        if Self::is_module_custom(manifest, module_name) {
            return Ok(Vec::new());
        }
        let lang = manifest.project.lang;
        let mut created = Vec::new();
        let mut ctx = TeraContext::new();
        ctx.insert("project_name", &manifest.project.name);
        ctx.insert("namespace", &manifest.project.name);
        ctx.insert("domain", &manifest.project.name);
        ctx.insert("dispatcher", manifest.project.dispatcher.as_str());
        ctx.insert("paradigm", "fp");

        match lang {
            crate::manifest::Lang::Rust => {
                match module_name {
                    "caching" => {
                        ctx.insert("provider", "memory");
                        let out = project_dir.join("src/caching/mod.rs");
                        self.render_file("rust/caching/memory_cache.rs.tera", &ctx, &out)?;
                        created.push(out);
                    }
                    "logging" => {
                        ctx.insert("provider", "structured");
                        let out = project_dir.join("src/logging/mod.rs");
                        self.render_file("rust/logging/logging_middleware.rs.tera", &ctx, &out)?;
                        created.push(out);
                    }
                    "transaction" => {
                        ctx.insert("provider", "native");
                        let out = project_dir.join("src/transaction/mod.rs");
                        self.render_file("rust/transaction/transaction_middleware.rs.tera", &ctx, &out)?;
                        created.push(out);
                    }
                    "authentication" | "auth" => {
                        ctx.insert("provider", "jwt");
                        let auth_dir = project_dir.join("src/auth");
                        fs::create_dir_all(&auth_dir)?;

                        let jwt_out = auth_dir.join("jwt_service.rs");
                        self.render_file("rust/auth/jwt_service.rs.tera", &ctx, &jwt_out)?;
                        created.push(jwt_out);

                        let mid_out = auth_dir.join("auth_middleware.rs");
                        self.render_file("rust/auth/auth_middleware.rs.tera", &ctx, &mid_out)?;
                        created.push(mid_out);

                        let mod_out = auth_dir.join("mod.rs");
                        if !mod_out.exists() {
                            fs::write(&mod_out, "// @clrinf:generated — Auth Module Root\npub mod jwt_service;\npub mod auth_middleware;\npub use jwt_service::*;\npub use auth_middleware::*;\n")?;
                            created.push(mod_out);
                        }
                    }
                    "authorization" | "authz" => {
                        ctx.insert("provider", "cedar");
                        let out = project_dir.join("src/authz/mod.rs");
                        self.render_file("rust/authz/authorization_middleware.rs.tera", &ctx, &out)?;
                        created.push(out);

                        // Render multi-tenant Cedar policy
                        let policy_out = project_dir.join("policies/default.cedar");
                        self.render_file("cedar/policy.tera", &ctx, &policy_out)?;
                        created.push(policy_out);
                    }
                    _ => {}
                }
            }
            crate::manifest::Lang::TypeScript => {
                match module_name {
                    "caching" => {
                        ctx.insert("provider", "memory");
                        let out = project_dir.join("src/caching/index.ts");
                        self.render_file("typescript/caching/memory-cache.ts.tera", &ctx, &out)?;
                        created.push(out);
                    }
                    "logging" => {
                        ctx.insert("provider", "structured");
                        let out = project_dir.join("src/logging/index.ts");
                        self.render_file("typescript/logging/logging-middleware.ts.tera", &ctx, &out)?;
                        created.push(out);
                    }
                    "transaction" => {
                        ctx.insert("provider", "native");
                        let out = project_dir.join("src/transaction/index.ts");
                        self.render_file("typescript/transaction/transaction-middleware.ts.tera", &ctx, &out)?;
                        created.push(out);
                    }
                    "authentication" | "auth" => {
                        ctx.insert("provider", "jwt");
                        let auth_dir = project_dir.join("src/auth");
                        fs::create_dir_all(&auth_dir)?;

                        let jwt_out = auth_dir.join("jwt-service.ts");
                        self.render_file("typescript/auth/jwt-service.ts.tera", &ctx, &jwt_out)?;
                        created.push(jwt_out);

                        let mid_out = auth_dir.join("auth-middleware.ts");
                        self.render_file("typescript/auth/auth-middleware.ts.tera", &ctx, &mid_out)?;
                        created.push(mid_out);

                        let index_out = auth_dir.join("index.ts");
                        if !index_out.exists() {
                            fs::write(&index_out, "// @clrinf:generated — Auth Module Root\nexport * from \"./jwt-service.js\";\nexport * from \"./auth-middleware.js\";\n")?;
                            created.push(index_out);
                        }
                    }
                    "authorization" | "authz" => {
                        ctx.insert("provider", "cedar");
                        let out = project_dir.join("src/authz/index.ts");
                        self.render_file("typescript/authz/authorization-middleware.ts.tera", &ctx, &out)?;
                        created.push(out);

                        // Render multi-tenant Cedar policy
                        let policy_out = project_dir.join("policies/default.cedar");
                        self.render_file("cedar/policy.tera", &ctx, &policy_out)?;
                        created.push(policy_out);
                    }
                    _ => {}
                }
            }
            crate::manifest::Lang::CSharp => {
                match module_name {
                    "caching" => {
                        ctx.insert("provider", "memory");
                        let dir = project_dir.join("src/Common/Caching");
                        let f1 = dir.join("ICacheService.cs");
                        let f2 = dir.join("MemoryCacheService.cs");
                        let f3 = dir.join("CachingBehavior.cs");
                        let f4 = dir.join("CacheRegistration.cs");
                        self.render_file("csharp/caching/ICacheService.cs.tera", &ctx, &f1)?;
                        self.render_file("csharp/caching/MemoryCacheService.cs.tera", &ctx, &f2)?;
                        self.render_file("csharp/caching/CachingBehavior.cs.tera", &ctx, &f3)?;
                        self.render_file("csharp/caching/CacheRegistration.cs.tera", &ctx, &f4)?;
                        created.extend([f1, f2, f3, f4]);
                    }
                    "logging" => {
                        ctx.insert("provider", "structured");
                        let dir = project_dir.join("src/Common/Logging");
                        let f1 = dir.join("LoggingBehavior.cs");
                        let f2 = dir.join("LogRegistration.cs");
                        self.render_file("csharp/logging/LoggingBehavior.cs.tera", &ctx, &f1)?;
                        self.render_file("csharp/logging/LogRegistration.cs.tera", &ctx, &f2)?;
                        created.extend([f1, f2]);
                    }
                    "transaction" => {
                        ctx.insert("provider", "native");
                        let dir = project_dir.join("src/Common/Transaction");
                        let f1 = dir.join("IUnitOfWork.cs");
                        let f2 = dir.join("TransactionBehavior.cs");
                        let f3 = dir.join("TransactionRegistration.cs");
                        self.render_file("csharp/transaction/IUnitOfWork.cs.tera", &ctx, &f1)?;
                        self.render_file("csharp/transaction/TransactionBehavior.cs.tera", &ctx, &f2)?;
                        self.render_file("csharp/transaction/TransactionRegistration.cs.tera", &ctx, &f3)?;
                        created.extend([f1, f2, f3]);
                    }
                    "authentication" | "auth" => {
                        ctx.insert("provider", "jwt");
                        let dir = project_dir.join("src/Common/Auth");
                        let f1 = dir.join("JwtTokenService.cs");
                        let f2 = dir.join("AuthRegistration.cs");
                        self.render_file("csharp/auth/JwtTokenService.cs.tera", &ctx, &f1)?;
                        self.render_file("csharp/auth/AuthRegistration.cs.tera", &ctx, &f2)?;
                        created.extend([f1, f2]);
                    }
                    "authorization" | "authz" => {
                        ctx.insert("provider", "cedar");
                        let dir = project_dir.join("src/Common/Authz");
                        let f1 = dir.join("AuthorizationBehavior.cs");
                        let f2 = dir.join("CedarPolicyService.cs");
                        let f3 = dir.join("CedarRegistration.cs");
                        self.render_file("csharp/authz/AuthorizationBehavior.cs.tera", &ctx, &f1)?;
                        self.render_file("csharp/authz/CedarPolicyService.cs.tera", &ctx, &f2)?;
                        self.render_file("csharp/authz/CedarRegistration.cs.tera", &ctx, &f3)?;
                        created.extend([f1, f2, f3]);

                        // Render multi-tenant Cedar policy
                        let policy_out = project_dir.join("policies/default.cedar");
                        self.render_file("cedar/policy.tera", &ctx, &policy_out)?;
                        created.push(policy_out);
                    }
                    "idempotency" => {
                        let dir = project_dir.join("src/Common/Idempotency");
                        let f1 = dir.join("IdempotencyStore.cs");
                        self.render_file("csharp/idempotency/IdempotencyStore.cs.tera", &ctx, &f1)?;
                        created.push(f1);
                    }
                    "outbox" => {
                        let dir = project_dir.join("src/Common/Outbox");
                        let f1 = dir.join("OutboxStore.cs");
                        self.render_file("csharp/outbox/OutboxStore.cs.tera", &ctx, &f1)?;
                        created.push(f1);
                    }
                    _ => {}
                }
            }
            crate::manifest::Lang::Elixir => {}
        }

        Ok(created)
    }

    fn render_file(&self, template_name: &str, ctx: &TeraContext, out_path: &Path) -> Result<()> {
        if let Some(parent) = out_path.parent() {
            fs::create_dir_all(parent)?;
        }
        let rendered = self.tera.render(template_name, ctx).with_context(|| {
            format!("Failed to render template: {}", template_name)
        })?;
        fs::write(out_path, rendered).with_context(|| {
            format!("Failed to write rendered file to: {}", out_path.display())
        })?;
        Ok(())
    }

    fn cleanup_module_files(
        &self,
        manifest: &ProjectManifest,
        project_dir: &Path,
        module_name: &str,
    ) -> Result<Vec<PathBuf>> {
        let lang = manifest.project.lang;
        let mut removed = Vec::new();

        let target_dirs = match lang {
            crate::manifest::Lang::Rust => match module_name {
                "caching" => vec![project_dir.join("src/caching")],
                "logging" => vec![project_dir.join("src/logging")],
                "transaction" => vec![project_dir.join("src/transaction")],
                "authentication" | "auth" => vec![project_dir.join("src/auth")],
                "authorization" | "authz" => vec![project_dir.join("src/authz")],
                _ => vec![],
            },
            crate::manifest::Lang::TypeScript => match module_name {
                "caching" => vec![project_dir.join("src/caching")],
                "logging" => vec![project_dir.join("src/logging")],
                "transaction" => vec![project_dir.join("src/transaction")],
                "authentication" | "auth" => vec![project_dir.join("src/auth")],
                "authorization" | "authz" => vec![project_dir.join("src/authz")],
                _ => vec![],
            },
            crate::manifest::Lang::CSharp => match module_name {
                "caching" => vec![project_dir.join("src/Common/Caching")],
                "logging" => vec![project_dir.join("src/Common/Logging")],
                "transaction" => vec![project_dir.join("src/Common/Transaction")],
                "authentication" | "auth" => vec![project_dir.join("src/Common/Auth")],
                "authorization" | "authz" => vec![project_dir.join("src/Common/Authz")],
                "idempotency" => vec![project_dir.join("src/Common/Idempotency")],
                "outbox" => vec![project_dir.join("src/Common/Outbox")],
                _ => vec![],
            },
            crate::manifest::Lang::Elixir => vec![],
        };

        for dir in target_dirs {
            if dir.is_dir() {
                if let Ok(entries) = fs::read_dir(&dir) {
                    for entry in entries.flatten() {
                        let path = entry.path();
                        if path.is_file() {
                            if let Ok(content) = fs::read_to_string(&path) {
                                if content.contains("@clrinf:generated") {
                                    let _ = fs::remove_file(&path);
                                    removed.push(path);
                                }
                            }
                        }
                    }
                }
                // If directory is now empty, remove directory
                let _ = fs::remove_dir(&dir);
            }
        }

        Ok(removed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::manifest::{ArchStyle, Lang, ProjectConfig};
    use tempfile::tempdir;

    fn get_contracts_templates_dir() -> PathBuf {
        let cwd = std::env::current_dir().unwrap();
        let mut current = Some(cwd.as_path());
        while let Some(dir) = current {
            let candidate = dir.join("tools/clrinf-codegen/templates");
            if candidate.is_dir() {
                return candidate;
            }
            let candidate = dir.join("templates");
            if candidate.is_dir() {
                return candidate;
            }
            current = dir.parent();
        }
        panic!("tools/clrinf-codegen/templates not found");
    }

    #[test]
    fn test_rust_module_add_and_remove() {
        let tmp = tempdir().unwrap();
        let project_dir = tmp.path();
        let manifest_path = project_dir.join("clrinf.toml");

        let mut manifest = ProjectManifest {
            project: ProjectConfig {
                name: "TestStore".to_string(),
                lang: Lang::Rust,
                arch: ArchStyle::CleanCqrs,
                ..Default::default()
            },
            ..Default::default()
        };
        manifest.save(&manifest_path).unwrap();

        let manager = ModuleManager::new(Some(get_contracts_templates_dir())).unwrap();

        // 1. Add Caching
        let created = manager
            .add_module(&mut manifest, &manifest_path, "caching", Some("memory"))
            .unwrap();
        assert!(!created.is_empty());
        assert!(project_dir.join("src/caching/mod.rs").is_file());

        // 2. Add Authorization with Cedar
        let created_authz = manager
            .add_module(&mut manifest, &manifest_path, "authorization", Some("cedar"))
            .unwrap();
        assert!(!created_authz.is_empty());
        assert!(project_dir.join("src/authz/mod.rs").is_file());
        assert!(project_dir.join("policies/default.cedar").is_file());

        // Verify multi-tenant policy content
        let cedar_content = fs::read_to_string(project_dir.join("policies/default.cedar")).unwrap();
        assert!(cedar_content.contains("resource.tenant_id != principal.tenant_id"));
        assert!(cedar_content.contains("PlatformAdmin"));

        // 3. Remove Caching
        let removed = manager
            .remove_module(&mut manifest, &manifest_path, "caching")
            .unwrap();
        assert!(!removed.is_empty());
        assert!(!project_dir.join("src/caching/mod.rs").exists());
    }

    #[test]
    fn test_typescript_module_add_and_sync() {
        let tmp = tempdir().unwrap();
        let project_dir = tmp.path();
        let manifest_path = project_dir.join("clrinf.toml");

        let mut manifest = ProjectManifest {
            project: ProjectConfig {
                name: "StoreJs".to_string(),
                lang: Lang::TypeScript,
                arch: ArchStyle::CleanCqrs,
                ..Default::default()
            },
            ..Default::default()
        };
        manifest.save(&manifest_path).unwrap();

        let manager = ModuleManager::new(Some(get_contracts_templates_dir())).unwrap();

        // Add logging
        manager
            .add_module(&mut manifest, &manifest_path, "logging", None)
            .unwrap();
        assert!(project_dir.join("src/logging/index.ts").is_file());

        // Sync
        let report = manager.sync_modules(&manifest, project_dir).unwrap();
        assert!(report.synced.contains(&"logging".to_string()));
    }
}
