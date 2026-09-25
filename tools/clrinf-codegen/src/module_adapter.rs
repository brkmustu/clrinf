// @clrinf:generated — Sophisticated Module Adaptation and Transplantation Engine
//
// Implements `clrinf module adopt`:
// Enables developers to transplant any module (cross-cutting concerns or domain features like CRM Deals/Sales)
// into ANY existing project (C# .csproj, Rust Cargo.toml crate, TypeScript package.json directory)
// in either raw (zero-dependency pure code) or wired (auto-hooked) mode.

use anyhow::{bail, Context, Result};
use heck::{ToKebabCase, ToLowerCamelCase, ToSnakeCase, ToUpperCamelCase};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use std::str::FromStr;

fn diff_paths(path: &Path, base: &Path) -> Option<PathBuf> {
    if let Ok(rel) = path.strip_prefix(base) {
        return Some(rel.to_path_buf());
    }
    path.file_name().map(PathBuf::from)
}

use crate::manifest::{Lang, ProjectManifest};
use crate::module_wiring::ModuleWiring;

// ─── Target Project Resolution ───────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct TargetProject {
    pub lang: Lang,
    pub project_root: PathBuf,
    pub project_name: String,
    #[allow(dead_code)]
    pub target_file: Option<PathBuf>,
}

impl TargetProject {
    /// Resolves target project language, root directory, and name from a given path (file or directory).
    pub fn resolve(target_path: &Path) -> Result<Self> {
        let path = if target_path.is_relative() {
            std::env::current_dir()?.join(target_path)
        } else {
            target_path.to_path_buf()
        };

        if !path.exists() {
            bail!("Hedef proje yolu bulunamadı: {}", path.display());
        }

        // Case A: target_path is a file
        if path.is_file() {
            let file_name = path
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("")
                .to_string();
            let parent_dir = path
                .parent()
                .map(|p| p.to_path_buf())
                .unwrap_or_else(|| PathBuf::from("."));

            if file_name.ends_with(".csproj") {
                let name = file_name.trim_end_matches(".csproj").to_string();
                return Ok(Self {
                    lang: Lang::CSharp,
                    project_root: parent_dir,
                    project_name: name,
                    target_file: Some(path),
                });
            }

            if file_name == "Cargo.toml" || file_name.ends_with(".rs") {
                let root = if file_name == "Cargo.toml" {
                    parent_dir
                } else {
                    Self::find_parent_with_file(&path, "Cargo.toml").unwrap_or(parent_dir)
                };
                let name = Self::extract_cargo_pkg_name(&root.join("Cargo.toml"))
                    .unwrap_or_else(|| Self::dir_name(&root));
                return Ok(Self {
                    lang: Lang::Rust,
                    project_root: root,
                    project_name: name,
                    target_file: Some(path),
                });
            }

            if file_name == "package.json"
                || file_name.ends_with(".ts")
                || file_name.ends_with(".js")
            {
                let root = if file_name == "package.json" {
                    parent_dir
                } else {
                    Self::find_parent_with_file(&path, "package.json").unwrap_or(parent_dir)
                };
                let name = Self::extract_npm_pkg_name(&root.join("package.json"))
                    .unwrap_or_else(|| Self::dir_name(&root));
                return Ok(Self {
                    lang: Lang::TypeScript,
                    project_root: root,
                    project_name: name,
                    target_file: Some(path),
                });
            }

            bail!(
                "Belirtilen dosya bilinen bir proje dosyası değil (.csproj, Cargo.toml, package.json): {}",
                path.display()
            );
        }

        // Case B: target_path is a directory
        // 1. Check clrinf.toml first
        let clrinf_toml = path.join("clrinf.toml");
        if clrinf_toml.is_file() {
            if let Ok(manifest) = ProjectManifest::load(&clrinf_toml) {
                return Ok(Self {
                    lang: manifest.project.lang,
                    project_root: path.clone(),
                    project_name: manifest.project.name,
                    target_file: Some(clrinf_toml),
                });
            }
        }

        // 2. Check for .csproj in path or immediate subdirectories
        if let Ok(entries) = fs::read_dir(&path) {
            for entry in entries.flatten() {
                let p = entry.path();
                if p.is_file() && p.extension().map_or(false, |ext| ext == "csproj") {
                    let name = p
                        .file_stem()
                        .and_then(|n| n.to_str())
                        .unwrap_or("App")
                        .to_string();
                    return Ok(Self {
                        lang: Lang::CSharp,
                        project_root: path,
                        project_name: name,
                        target_file: Some(p),
                    });
                }
            }
        }

        // 3. Check for Cargo.toml
        let cargo_toml = path.join("Cargo.toml");
        if cargo_toml.is_file() {
            let name = Self::extract_cargo_pkg_name(&cargo_toml)
                .unwrap_or_else(|| Self::dir_name(&path));
            return Ok(Self {
                lang: Lang::Rust,
                project_root: path,
                project_name: name,
                target_file: Some(cargo_toml),
            });
        }

        // 4. Check for package.json
        let pkg_json = path.join("package.json");
        if pkg_json.is_file() {
            let name = Self::extract_npm_pkg_name(&pkg_json)
                .unwrap_or_else(|| Self::dir_name(&path));
            return Ok(Self {
                lang: Lang::TypeScript,
                project_root: path,
                project_name: name,
                target_file: Some(pkg_json),
            });
        }

        // 5. Fallback checks for source files
        if path.join("src").is_dir() {
            if fs::read_dir(path.join("src"))
                .map(|mut r| r.any(|e| e.map_or(false, |x| x.path().extension().map_or(false, |ex| ex == "rs"))))
                .unwrap_or(false)
            {
                return Ok(Self {
                    lang: Lang::Rust,
                    project_root: path.clone(),
                    project_name: Self::dir_name(&path),
                    target_file: None,
                });
            }
            if fs::read_dir(path.join("src"))
                .map(|mut r| r.any(|e| e.map_or(false, |x| x.path().extension().map_or(false, |ex| ex == "cs"))))
                .unwrap_or(false)
            {
                return Ok(Self {
                    lang: Lang::CSharp,
                    project_root: path.clone(),
                    project_name: Self::dir_name(&path),
                    target_file: None,
                });
            }
        }

        bail!(
            "Hedef dizinde proje türü otomatik tespit edilemedi (.csproj, Cargo.toml, package.json bulunamadı): {}",
            path.display()
        );
    }

    fn dir_name(p: &Path) -> String {
        p.file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("App")
            .to_string()
    }

    fn find_parent_with_file(start: &Path, file_name: &str) -> Option<PathBuf> {
        let mut curr = start.parent();
        while let Some(dir) = curr {
            if dir.join(file_name).is_file() {
                return Some(dir.to_path_buf());
            }
            curr = dir.parent();
        }
        None
    }

    fn extract_cargo_pkg_name(cargo_path: &Path) -> Option<String> {
        let content = fs::read_to_string(cargo_path).ok()?;
        let parsed: toml::Value = toml::from_str(&content).ok()?;
        parsed
            .get("package")?
            .get("name")?
            .as_str()
            .map(|s| s.to_string())
    }

    fn extract_npm_pkg_name(pkg_path: &Path) -> Option<String> {
        let content = fs::read_to_string(pkg_path).ok()?;
        let parsed: serde_json::Value = serde_json::from_str(&content).ok()?;
        parsed
            .get("name")?
            .as_str()
            .map(|s| s.trim_start_matches('@').replace('/', "-"))
    }
}

// ─── Module Adoption Options ─────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AdoptMode {
    Raw,
    Wired,
}

impl Default for AdoptMode {
    fn default() -> Self {
        AdoptMode::Raw
    }
}

impl FromStr for AdoptMode {
    type Err = anyhow::Error;
    fn from_str(s: &str) -> Result<Self> {
        match s.to_lowercase().as_str() {
            "raw" | "pure" | "ham" => Ok(AdoptMode::Raw),
            "wired" | "integrated" | "hooked" => Ok(AdoptMode::Wired),
            _ => bail!(
                "Bilinmeyen adopt modu: '{}'. (Desteklenen modlar: 'raw', 'wired')",
                s
            ),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CatalogModule {
    pub id: String,
    pub name: String,
    pub category: String,
    pub description: String,
    pub default_target_dir: String,
    pub supports_raw: bool,
    pub supports_wired: bool,
    pub entities: Vec<String>,
}

pub fn get_built_in_catalog() -> Vec<CatalogModule> {
    vec![
        CatalogModule {
            id: "deals".to_string(),
            name: "CRM Deals / Sales Pipeline".to_string(),
            category: "Domain Feature".to_string(),
            description: "Satış fırsatları hunisi (Prospect, Qualified, Proposal, Won, Lost), aşama geçiş kuralları ve Cedar politikası.".to_string(),
            default_target_dir: "src/Features/Sales".to_string(),
            supports_raw: true,
            supports_wired: true,
            entities: vec!["Deal".to_string()],
        },
        CatalogModule {
            id: "contacts".to_string(),
            name: "CRM Contacts / Customers".to_string(),
            category: "Domain Feature".to_string(),
            description: "Müşteri ve aday yönetimi, RFC e-posta doğrulaması, kiracı-içi mükerrerlik kontrolü ve Cedar politikası.".to_string(),
            default_target_dir: "src/Features/Contacts".to_string(),
            supports_raw: true,
            supports_wired: true,
            entities: vec!["Contact".to_string()],
        },
        CatalogModule {
            id: "activities".to_string(),
            name: "CRM Activities / Interactions".to_string(),
            category: "Domain Feature".to_string(),
            description: "İletişim, görüşme ve görev takibi, kayıp anlaşma koruma kuralı ve Cedar politikası.".to_string(),
            default_target_dir: "src/Features/Activities".to_string(),
            supports_raw: true,
            supports_wired: true,
            entities: vec!["Activity".to_string()],
        },
        CatalogModule {
            id: "crm".to_string(),
            name: "CRM Full Suite".to_string(),
            category: "Domain Suite".to_string(),
            description: "Tam teşekküllü B2B CRM paketi (Deals + Contacts + Activities + birleşik Cedar güvenlik muhafızı).".to_string(),
            default_target_dir: "src/Features/Crm".to_string(),
            supports_raw: true,
            supports_wired: true,
            entities: vec!["Deal".to_string(), "Contact".to_string(), "Activity".to_string()],
        },
        CatalogModule {
            id: "caching".to_string(),
            name: "Caching / In-Memory & Redis".to_string(),
            category: "Cross-Cutting Concern".to_string(),
            description: "Yüksek başarımlı önbellekleme soyutlaması (ICacheService, MemoryCache).".to_string(),
            default_target_dir: "src/Common/Caching".to_string(),
            supports_raw: true,
            supports_wired: true,
            entities: vec![],
        },
        CatalogModule {
            id: "logging".to_string(),
            name: "Structured Logging".to_string(),
            category: "Cross-Cutting Concern".to_string(),
            description: "Yapılandırılmış loglama ve işlem yürütme zamanı ölçüm hattı.".to_string(),
            default_target_dir: "src/Common/Logging".to_string(),
            supports_raw: true,
            supports_wired: true,
            entities: vec![],
        },
        CatalogModule {
            id: "transaction".to_string(),
            name: "Transaction & Unit of Work".to_string(),
            category: "Cross-Cutting Concern".to_string(),
            description: "İşlem sınırları ve atomik Unit of Work yönetimi.".to_string(),
            default_target_dir: "src/Common/Transaction".to_string(),
            supports_raw: true,
            supports_wired: true,
            entities: vec![],
        },
        CatalogModule {
            id: "auth".to_string(),
            name: "Authentication / JWT".to_string(),
            category: "Cross-Cutting Concern".to_string(),
            description: "JWT kimlik doğrulama, token üretimi ve talepler (claims) denetimi.".to_string(),
            default_target_dir: "src/Common/Auth".to_string(),
            supports_raw: true,
            supports_wired: true,
            entities: vec![],
        },
        CatalogModule {
            id: "authz".to_string(),
            name: "Multi-Tenant Cedar Authorization".to_string(),
            category: "Cross-Cutting Concern".to_string(),
            description: "Amazon Cedar tabanlı, varsayılan çok kiracılı (multi-tenant by default) yetkilendirme motoru.".to_string(),
            default_target_dir: "src/Common/Authz".to_string(),
            supports_raw: true,
            supports_wired: true,
            entities: vec![],
        },
        CatalogModule {
            id: "idempotency".to_string(),
            name: "Idempotency Store".to_string(),
            category: "Cross-Cutting Concern".to_string(),
            description: "Kiracı-yalıtımlı mükerrer işlem engelleme portu (claim/complete/release).".to_string(),
            default_target_dir: "src/Common/Idempotency".to_string(),
            supports_raw: true,
            supports_wired: true,
            entities: vec![],
        },
        CatalogModule {
            id: "outbox".to_string(),
            name: "Transactional Outbox".to_string(),
            category: "Cross-Cutting Concern".to_string(),
            description: "Güvenilir, kayıpsız olay dağıtımı için transactional outbox kuyruğu.".to_string(),
            default_target_dir: "src/Common/Outbox".to_string(),
            supports_raw: true,
            supports_wired: true,
            entities: vec![],
        },
    ]
}

pub fn print_catalog() {
    println!("📦 clrinf Yerleşik Modül Kataloğu (Built-In Module Catalog)\n");
    println!("Kullanım:");
    println!("  clrinf module adopt <MODÜL> --to-project <PROJE> [--mode raw|wired] [--as <TAKMA_AD>]\n");

    let modules = get_built_in_catalog();

    println!("🏢 Domain Modülleri & Paketler (Domain Features & Suites):");
    for m in modules.iter().filter(|m| m.category.starts_with("Domain")) {
        println!("  • {:12} — {:32} [raw ✅ | wired ✅]", m.id, m.name);
        println!("    Tanım: {}", m.description);
        println!("    Varsayılan Dizin: {}", m.default_target_dir);
        if !m.entities.is_empty() {
            println!("    Varlıklar: {}", m.entities.join(", "));
        }
        println!();
    }

    println!("⚙️  Altyapı Modülleri (Cross-Cutting Concerns):");
    for m in modules.iter().filter(|m| m.category.starts_with("Cross")) {
        println!("  • {:12} — {:32} [raw ✅ | wired ✅]", m.id, m.name);
        println!("    Tanım: {}", m.description);
        println!("    Varsayılan Dizin: {}", m.default_target_dir);
        println!();
    }
}

#[derive(Debug, Clone)]
pub struct ModuleAdoptOptions {
    pub module: String,
    pub to_project: PathBuf,
    pub target_dir: Option<PathBuf>,
    pub mode: AdoptMode,
    pub r#as: Option<String>,
    pub source_project: Option<PathBuf>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdoptResult {
    pub module_name: String,
    pub alias: Option<String>,
    pub mode: String,
    pub lang: String,
    pub target_dir: PathBuf,
    pub files_created: Vec<PathBuf>,
    pub files_modified: Vec<PathBuf>,
    pub policy_created: Option<PathBuf>,
    pub notes: Vec<String>,
}

// ─── Module Adoption Engine ──────────────────────────────────────────────────

pub struct ModuleAdapter;

impl ModuleAdapter {
    /// Adopts and transplants a module into the target project according to options.
    pub fn adopt(options: &ModuleAdoptOptions) -> Result<AdoptResult> {
        let target = TargetProject::resolve(&options.to_project)?;

        let raw_module_name = options.module.trim().to_lowercase();
        let alias_opt = options.r#as.as_ref().map(|a| a.trim().to_string());
        let effective_name = alias_opt
            .clone()
            .unwrap_or_else(|| Self::normalize_catalog_name(&raw_module_name));

        // Determine destination folder inside target project
        let destination_dir = if let Some(ref explicit_dir) = options.target_dir {
            if explicit_dir.is_absolute() {
                explicit_dir.clone()
            } else {
                target.project_root.join(explicit_dir)
            }
        } else {
            Self::default_target_dir(&target, &effective_name)
        };

        fs::create_dir_all(&destination_dir).with_context(|| {
            format!("Hedef dizin oluşturulamadı: {}", destination_dir.display())
        })?;

        let mut files_modified = Vec::new();
        let mut notes = Vec::new();

        // Check if external source project is provided
        let (files_created, policy_created) = if let Some(ref src_proj) = options.source_project {
            let (created, policy) = Self::transplant_from_external(
                src_proj,
                &raw_module_name,
                &effective_name,
                &destination_dir,
                &target,
                options.mode,
            )?;
            notes.push(format!(
                "Modül dış projeden aktarıldı: {}",
                src_proj.display()
            ));
            (created, policy)
        } else {
            // Adopt from Built-In Catalog
            match target.lang {
                Lang::CSharp => Self::adopt_csharp(
                    &target,
                    &raw_module_name,
                    &effective_name,
                    &destination_dir,
                    options.mode,
                )?,
                Lang::Rust => Self::adopt_rust(
                    &target,
                    &raw_module_name,
                    &effective_name,
                    &destination_dir,
                    options.mode,
                )?,
                Lang::TypeScript => Self::adopt_typescript(
                    &target,
                    &raw_module_name,
                    &effective_name,
                    &destination_dir,
                    options.mode,
                )?,
                Lang::Elixir => {
                    anyhow::bail!("Built-in catalog adoption for Elixir is coming soon. Use 'clrinf scaffold otp <Module>' instead.");
                }
            }
        };

        // In Wired Mode, auto-hook into target project entry
        if options.mode == AdoptMode::Wired {
            match target.lang {
                Lang::CSharp => {
                    let di_res = Self::wire_csharp_project(&target, &effective_name);
                    if let Ok(p) = di_res {
                        files_modified.push(p);
                        notes.push(format!(
                            "C# ServiceRegistration DI kaydı yapıldı: 'services.Add{}Module();'",
                            effective_name.to_upper_camel_case()
                        ));
                    }
                }
                Lang::Rust => {
                    let rust_mod = effective_name.to_snake_case();
                    let wire_res = ModuleWiring::wire(&target.project_root, "rust", &rust_mod);
                    if wire_res.is_ok() {
                        let entry = target.project_root.join("src/lib.rs");
                        if entry.exists() {
                            files_modified.push(entry);
                        } else {
                            let main_rs = target.project_root.join("src/main.rs");
                            if main_rs.exists() {
                                files_modified.push(main_rs);
                            }
                        }
                        notes.push(format!("Rust modül ağacına bağlandı: 'pub mod {};'", rust_mod));
                    }
                }
                Lang::TypeScript => {
                    let ts_mod = effective_name.to_lower_camel_case();
                    let rel_path = diff_paths(&destination_dir, &target.project_root.join("src"))
                        .unwrap_or_else(|| PathBuf::from(format!("./modules/{}", effective_name.to_kebab_case())));
                    let wire_res = Self::wire_typescript_project(&target, &ts_mod, &rel_path);
                    if let Ok(p) = wire_res {
                        files_modified.push(p);
                        notes.push(format!(
                            "TypeScript barrel export bağlandı: 'export * as {} from ...'",
                            ts_mod
                        ));
                    }
                }
                Lang::Elixir => {}
            }
        } else {
            notes.push("Raw modu devrede: Sıfır harici paket bağımlılığıyla izole olarak yerleştirildi.".to_string());
        }

        Ok(AdoptResult {
            module_name: raw_module_name,
            alias: alias_opt,
            mode: match options.mode {
                AdoptMode::Raw => "raw".to_string(),
                AdoptMode::Wired => "wired".to_string(),
            },
            lang: target.lang.as_str().to_string(),
            target_dir: destination_dir,
            files_created,
            files_modified,
            policy_created,
            notes,
        })
    }

    fn normalize_catalog_name(input: &str) -> String {
        let s = input.trim().to_lowercase();
        if s.starts_with("crm.") {
            s.trim_start_matches("crm.").to_string()
        } else if s.starts_with("crm-") {
            s.trim_start_matches("crm-").to_string()
        } else {
            s
        }
    }

    fn default_target_dir(target: &TargetProject, effective_name: &str) -> PathBuf {
        match target.lang {
            Lang::CSharp => {
                let folder_name = if let Ok(content) = fs::read_to_string(target.project_root.join("codegen.toml")) {
                    if content.contains("folder_name = \"Modules\"") || content.contains("mode = \"module\"") {
                        "Modules"
                    } else {
                        "Features"
                    }
                } else if target.project_root.join("Modules").is_dir()
                    || target.project_root.join("src/Modules").is_dir()
                {
                    "Modules"
                } else {
                    "Features"
                };

                let has_src = target.project_root.join("src").is_dir();
                if has_src {
                    target
                        .project_root
                        .join("src")
                        .join(folder_name)
                        .join(effective_name.to_upper_camel_case())
                } else {
                    target
                        .project_root
                        .join(folder_name)
                        .join(effective_name.to_upper_camel_case())
                }
            }
            Lang::Rust => target
                .project_root
                .join("src")
                .join(effective_name.to_snake_case()),
            Lang::TypeScript => target
                .project_root
                .join("src/modules")
                .join(effective_name.to_kebab_case()),
            Lang::Elixir => target
                .project_root
                .join("lib")
                .join(effective_name.to_snake_case()),
        }
    }

    // ─── C# Kod Adaptörü ──────────────────────────────────────────────────────

    fn adopt_csharp(
        target: &TargetProject,
        raw_name: &str,
        effective_name: &str,
        dest_dir: &Path,
        mode: AdoptMode,
    ) -> Result<(Vec<PathBuf>, Option<PathBuf>)> {
        let mut created = Vec::new();
        let domain_cap = effective_name.to_upper_camel_case();
        let singular_cap = if domain_cap.ends_with('s') && domain_cap.len() > 1 {
            domain_cap[..domain_cap.len() - 1].to_string()
        } else {
            domain_cap.clone()
        };
        let domain_lower = domain_cap.to_lowercase();

        // Calculate namespace based on relative path from project root
        let rel_dest = diff_paths(dest_dir, &target.project_root)
            .unwrap_or_else(|| PathBuf::from("Features").join(&domain_cap));
        let rel_parts: Vec<String> = rel_dest
            .iter()
            .filter_map(|p| p.to_str())
            .filter(|p| *p != "src" && *p != ".")
            .map(|p| p.to_upper_camel_case())
            .collect();
        let namespace = if rel_parts.is_empty() {
            target.project_name.clone()
        } else {
            format!("{}.{}", target.project_name, rel_parts.join("."))
        };

        match raw_name {
            "crm" | "crm.all" => {
                let deals_dir = dest_dir.join("Deals");
                let contacts_dir = dest_dir.join("Contacts");
                let activities_dir = dest_dir.join("Activities");
                fs::create_dir_all(&deals_dir)?;
                fs::create_dir_all(&contacts_dir)?;
                fs::create_dir_all(&activities_dir)?;

                let (d_files, _) = Self::adopt_csharp(target, "deals", "Deals", &deals_dir, mode)?;
                let (c_files, _) = Self::adopt_csharp(target, "contacts", "Contacts", &contacts_dir, mode)?;
                let (a_files, _) = Self::adopt_csharp(target, "activities", "Activities", &activities_dir, mode)?;

                created.extend(d_files);
                created.extend(c_files);
                created.extend(a_files);

                if mode == AdoptMode::Wired {
                    let di_file = dest_dir.join("CrmServiceRegistration.cs");
                    let di_code = format!(
                        r#"// @clrinf:generated — Full CRM Suite Service Registration (Wired Mode)
using Microsoft.Extensions.DependencyInjection;
using {namespace}.Deals;
using {namespace}.Contacts;
using {namespace}.Activities;

namespace {namespace};

public static class CrmServiceRegistration
{{
    public static IServiceCollection AddCrmServices(this IServiceCollection services)
    {{
        services.AddSingleton<IDealRepository, InMemoryDealRepository>();
        services.AddSingleton<IContactRepository, InMemoryContactRepository>();
        services.AddSingleton<IActivityRepository, InMemoryActivityRepository>();
        return services;
    }}
}}
"#
                    );
                    fs::write(&di_file, di_code)?;
                    created.push(di_file);
                }

                let policy_path = Self::render_crm_suite_cedar_policy(target)?;
                return Ok((created, Some(policy_path)));
            }
            "deals" | "sales" | "crm.deals" | "crm-deals" => {
                let file_path = dest_dir.join(format!("{}Module.cs", domain_cap));
                let code = format!(
                    r#"// @clrinf:generated — Adopted Domain Module: {domain_cap} ({mode:?})
using System;
using System.Collections.Concurrent;
using System.Collections.Generic;
using System.Threading.Tasks;

namespace {namespace};

// ─── Self-Contained Operation Claim Contracts (Pure Raw Mode) ───────────────
public sealed record OperationClaim(string Resource, string Action, string? TenantId = null)
{{
    public static OperationClaim WithTenant(string resource, string action, string tenantId) =>
        new(resource, action, tenantId);
}}

public interface IRequireOperationClaim
{{
    OperationClaim GetRequiredClaim();
}}

public interface IRequest<out TResponse> {{ }}

// ─── Domain Entity & CQRS Contracts ──────────────────────────────────────────
public sealed record {singular_cap}(
    string Id,
    string TenantId,
    string Title,
    string ContactId,
    decimal Amount,
    string Stage,
    int Probability
);

public sealed record Create{singular_cap}Command(
    string TenantId,
    string Title,
    string ContactId,
    decimal Amount
) : IRequest<{singular_cap}>, IRequireOperationClaim
{{
    public OperationClaim GetRequiredClaim() =>
        OperationClaim.WithTenant("{domain_cap}", "{domain_lower}.create", TenantId);
}}

public sealed record Change{singular_cap}StageCommand(
    string TenantId,
    string {singular_cap}Id,
    string NewStage
) : IRequest<{singular_cap}>, IRequireOperationClaim
{{
    public OperationClaim GetRequiredClaim() =>
        OperationClaim.WithTenant("{domain_cap}", "{domain_lower}.stage_change", TenantId);
}}

public sealed record Get{singular_cap}Query(
    string TenantId,
    string {singular_cap}Id
) : IRequest<{singular_cap}?>, IRequireOperationClaim
{{
    public OperationClaim GetRequiredClaim() =>
        OperationClaim.WithTenant("{domain_cap}", "{domain_lower}.read", TenantId);
}}

// ─── Repository Port & Thread-Safe In-Memory Adapter ─────────────────────────
public interface I{singular_cap}Repository
{{
    ValueTask<{singular_cap}?> GetByIdAsync(string tenantId, string id);
    ValueTask<{singular_cap}> SaveAsync({singular_cap} entity);
}}

public sealed class InMemory{singular_cap}Repository : I{singular_cap}Repository
{{
    private readonly ConcurrentDictionary<string, {singular_cap}> _storage = new();

    public ValueTask<{singular_cap}?> GetByIdAsync(string tenantId, string id)
    {{
        _storage.TryGetValue($"{{tenantId}}:{{id}}", out var entity);
        return ValueTask.FromResult(entity);
    }}

    public ValueTask<{singular_cap}> SaveAsync({singular_cap} entity)
    {{
        _storage[$"{{entity.TenantId}}:{{entity.Id}}"] = entity;
        return ValueTask.FromResult(entity);
    }}
}}

// ─── Functional Business Rules ───────────────────────────────────────────────
public static class {singular_cap}Rules
{{
    private static readonly HashSet<string> ValidStages = new(StringComparer.OrdinalIgnoreCase)
    {{
        "Prospect", "Qualified", "Proposal", "Negotiation", "ClosedWon", "ClosedLost"
    }};

    public static void EnsureAmountPositive(decimal amount)
    {{
        if (amount <= 0)
            throw new ArgumentException("{singular_cap} amount must be greater than zero.", nameof(amount));
    }}

    public static void EnsureStageValid(string stage)
    {{
        if (!ValidStages.Contains(stage))
            throw new ArgumentException($"Stage '{{stage}}' is not a valid {domain_lower} stage.", nameof(stage));
    }}
}}
"#,
                    domain_cap = domain_cap,
                    singular_cap = singular_cap,
                    domain_lower = domain_lower,
                    namespace = namespace,
                    mode = mode
                );

                fs::write(&file_path, code)?;
                created.push(file_path);
            }
            "contacts" | "customers" | "crm.contacts" | "crm-contacts" => {
                let file_path = dest_dir.join(format!("{}Module.cs", domain_cap));
                let code = format!(
                    r#"// @clrinf:generated — Adopted Domain Module: {domain_cap} ({mode:?})
using System;
using System.Collections.Concurrent;
using System.Collections.Generic;
using System.Threading.Tasks;

namespace {namespace};

public sealed record OperationClaim(string Resource, string Action, string? TenantId = null)
{{
    public static OperationClaim WithTenant(string resource, string action, string tenantId) =>
        new(resource, action, tenantId);
}}

public interface IRequireOperationClaim
{{
    OperationClaim GetRequiredClaim();
}}

public interface IRequest<out TResponse> {{ }}

public sealed record {singular_cap}(
    string Id,
    string TenantId,
    string FirstName,
    string LastName,
    string Email,
    string Company,
    string Status
);

public sealed record Create{singular_cap}Command(
    string TenantId,
    string FirstName,
    string LastName,
    string Email,
    string Company
) : IRequest<{singular_cap}>, IRequireOperationClaim
{{
    public OperationClaim GetRequiredClaim() =>
        OperationClaim.WithTenant("{domain_cap}", "{domain_lower}.create", TenantId);
}}

public sealed record Get{singular_cap}Query(
    string TenantId,
    string {singular_cap}Id
) : IRequest<{singular_cap}?>, IRequireOperationClaim
{{
    public OperationClaim GetRequiredClaim() =>
        OperationClaim.WithTenant("{domain_cap}", "{domain_lower}.read", TenantId);
}}

public interface I{singular_cap}Repository
{{
    ValueTask<{singular_cap}?> GetByIdAsync(string tenantId, string id);
    ValueTask<{singular_cap}> SaveAsync({singular_cap} entity);
}}

public sealed class InMemory{singular_cap}Repository : I{singular_cap}Repository
{{
    private readonly ConcurrentDictionary<string, {singular_cap}> _storage = new();

    public ValueTask<{singular_cap}?> GetByIdAsync(string tenantId, string id)
    {{
        _storage.TryGetValue($"{{tenantId}}:{{id}}", out var entity);
        return ValueTask.FromResult(entity);
    }}

    public ValueTask<{singular_cap}> SaveAsync({singular_cap} entity)
    {{
        _storage[$"{{entity.TenantId}}:{{entity.Id}}"] = entity;
        return ValueTask.FromResult(entity);
    }}
}}
"#,
                    domain_cap = domain_cap,
                    singular_cap = singular_cap,
                    domain_lower = domain_lower,
                    namespace = namespace,
                    mode = mode
                );

                fs::write(&file_path, code)?;
                created.push(file_path);
            }
            "activities" | "crm.activities" | "crm-activities" => {
                let file_path = dest_dir.join(format!("{}Module.cs", domain_cap));
                let code = format!(
                    r#"// @clrinf:generated — Adopted Domain Module: {domain_cap} ({mode:?})
using System;
using System.Collections.Concurrent;
using System.Threading.Tasks;

namespace {namespace};

public sealed record OperationClaim(string Resource, string Action, string? TenantId = null)
{{
    public static OperationClaim WithTenant(string resource, string action, string tenantId) =>
        new(resource, action, tenantId);
}}

public interface IRequireOperationClaim
{{
    OperationClaim GetRequiredClaim();
}}

public interface IRequest<out TResponse> {{ }}

public sealed record {singular_cap}(
    string Id,
    string TenantId,
    string Title,
    string? DealId,
    string ActivityType,
    string Status
);

public sealed record Log{singular_cap}Command(
    string TenantId,
    string Title,
    string? DealId,
    string ActivityType
) : IRequest<{singular_cap}>, IRequireOperationClaim
{{
    public OperationClaim GetRequiredClaim() =>
        OperationClaim.WithTenant("{domain_cap}", "{domain_lower}.create", TenantId);
}}

public interface I{singular_cap}Repository
{{
    ValueTask<{singular_cap}?> GetByIdAsync(string tenantId, string id);
    ValueTask<{singular_cap}> SaveAsync({singular_cap} entity);
}}

public sealed class InMemory{singular_cap}Repository : I{singular_cap}Repository
{{
    private readonly ConcurrentDictionary<string, {singular_cap}> _storage = new();

    public ValueTask<{singular_cap}?> GetByIdAsync(string tenantId, string id)
    {{
        _storage.TryGetValue($"{{tenantId}}:{{id}}", out var entity);
        return ValueTask.FromResult(entity);
    }}

    public ValueTask<{singular_cap}> SaveAsync({singular_cap} entity)
    {{
        _storage[$"{{entity.TenantId}}:{{entity.Id}}"] = entity;
        return ValueTask.FromResult(entity);
    }}
}}
"#,
                    domain_cap = domain_cap,
                    singular_cap = singular_cap,
                    domain_lower = domain_lower,
                    namespace = namespace,
                    mode = mode
                );

                fs::write(&file_path, code)?;
                created.push(file_path);
            }
            "caching" | "cache" => {
                let file_path = dest_dir.join("CacheService.cs");
                let code = format!(
                    r#"// @clrinf:generated — Pure Self-Contained In-Memory Caching
using System;
using System.Collections.Concurrent;
using System.Threading.Tasks;

namespace {namespace};

public interface ICacheService
{{
    ValueTask<T?> GetAsync<T>(string key);
    ValueTask SetAsync<T>(string key, T value, TimeSpan? duration = null);
    ValueTask RemoveAsync(string key);
}}

public sealed class MemoryCacheService : ICacheService
{{
    private readonly ConcurrentDictionary<string, (object Value, DateTime ExpiresAt)> _cache = new();

    public ValueTask<T?> GetAsync<T>(string key)
    {{
        if (_cache.TryGetValue(key, out var item))
        {{
            if (DateTime.UtcNow <= item.ExpiresAt)
                return ValueTask.FromResult((T?)item.Value);
            _cache.TryRemove(key, out _);
        }}
        return ValueTask.FromResult(default(T));
    }}

    public ValueTask SetAsync<T>(string key, T value, TimeSpan? duration = null)
    {{
        var exp = DateTime.UtcNow.Add(duration ?? TimeSpan.FromMinutes(30));
        _cache[key] = (value!, exp);
        return ValueTask.CompletedTask;
    }}

    public ValueTask RemoveAsync(string key)
    {{
        _cache.TryRemove(key, out _);
        return ValueTask.CompletedTask;
    }}
}}
"#
                );
                fs::write(&file_path, code)?;
                created.push(file_path);
            }
            "authorization" | "authz" => {
                let file_path = dest_dir.join("CedarAuthorizer.cs");
                let code = format!(
                    r#"// @clrinf:generated — Pure Self-Contained Multi-Tenant Authorization
using System;
using System.Collections.Generic;

namespace {namespace};

public sealed class CedarPolicyRule
{{
    public string? PrincipalRole {{ get; set; }}
    public string? Action {{ get; set; }}
    public string? ResourceType {{ get; set; }}
    public bool RequireTenantMatch {{ get; set; }} = true;
    public bool EffectPermit {{ get; set; }} = true;
}}

public sealed class MultiTenantCedarAuthorizer
{{
    private readonly string _platformAdminRole;
    private readonly List<CedarPolicyRule> _rules = new();

    public MultiTenantCedarAuthorizer(string platformAdminRole = "PlatformAdmin")
    {{
        _platformAdminRole = platformAdminRole;
    }}

    public void AddRule(CedarPolicyRule rule) => _rules.Add(rule);

    public bool IsAuthorized(string role, string? action, string resourceType, string? tenantId, string? targetTenantId)
    {{
        if (role == _platformAdminRole) return true;
        if (tenantId != targetTenantId) return false;

        foreach (var rule in _rules)
        {{
            if (rule.ResourceType != null && !rule.ResourceType.Equals(resourceType, StringComparison.OrdinalIgnoreCase))
                continue;
            if (rule.PrincipalRole != null && !rule.PrincipalRole.Equals(role, StringComparison.OrdinalIgnoreCase))
                continue;
            if (rule.Action != null && action != null && !rule.Action.Equals(action, StringComparison.OrdinalIgnoreCase))
                continue;
            return rule.EffectPermit;
        }}
        return false;
    }}
}}
"#
                );
                fs::write(&file_path, code)?;
                created.push(file_path);
            }
            _ => {
                // Generic Domain Module Scaffold
                let file_path = dest_dir.join(format!("{}Module.cs", domain_cap));
                let code = format!(
                    r#"// @clrinf:generated — Adopted Module: {domain_cap} ({mode:?})
using System;
using System.Collections.Concurrent;
using System.Threading.Tasks;

namespace {namespace};

public sealed record OperationClaim(string Resource, string Action, string? TenantId = null);

public sealed record {singular_cap}(string Id, string TenantId, string Name);

public interface I{singular_cap}Repository
{{
    ValueTask<{singular_cap}?> GetByIdAsync(string tenantId, string id);
    ValueTask<{singular_cap}> SaveAsync({singular_cap} item);
}}

public sealed class InMemory{singular_cap}Repository : I{singular_cap}Repository
{{
    private readonly ConcurrentDictionary<string, {singular_cap}> _storage = new();

    public ValueTask<{singular_cap}?> GetByIdAsync(string tenantId, string id)
    {{
        _storage.TryGetValue($"{{tenantId}}:{{id}}", out var item);
        return ValueTask.FromResult(item);
    }}

    public ValueTask<{singular_cap}> SaveAsync({singular_cap} item)
    {{
        _storage[$"{{item.TenantId}}:{{item.Id}}"] = item;
        return ValueTask.FromResult(item);
    }}
}}
"#
                );
                fs::write(&file_path, code)?;
                created.push(file_path);
            }
        }

        // Render multi-tenant Cedar policy
        let policy_path = Self::render_cedar_policy(target, &domain_cap, &domain_lower)?;

        Ok((created, Some(policy_path)))
    }

    // ─── Rust Kod Adaptörü ───────────────────────────────────────────────────

    fn adopt_rust(
        target: &TargetProject,
        raw_name: &str,
        effective_name: &str,
        dest_dir: &Path,
        mode: AdoptMode,
    ) -> Result<(Vec<PathBuf>, Option<PathBuf>)> {
        let mut created = Vec::new();
        let domain_snake = effective_name.to_snake_case();
        let domain_cap = effective_name.to_upper_camel_case();
        let singular_cap = if domain_cap.ends_with('s') && domain_cap.len() > 1 {
            domain_cap[..domain_cap.len() - 1].to_string()
        } else {
            domain_cap.clone()
        };
        let domain_lower = domain_cap.to_lowercase();

        let mod_rs = dest_dir.join("mod.rs");

        match raw_name {
            "crm" | "crm.all" => {
                let deals_dir = dest_dir.join("deals");
                let contacts_dir = dest_dir.join("contacts");
                let activities_dir = dest_dir.join("activities");
                fs::create_dir_all(&deals_dir)?;
                fs::create_dir_all(&contacts_dir)?;
                fs::create_dir_all(&activities_dir)?;

                let (d_files, _) = Self::adopt_rust(target, "deals", "deals", &deals_dir, mode)?;
                let (c_files, _) = Self::adopt_rust(target, "contacts", "contacts", &contacts_dir, mode)?;
                let (a_files, _) = Self::adopt_rust(target, "activities", "activities", &activities_dir, mode)?;

                created.extend(d_files);
                created.extend(c_files);
                created.extend(a_files);

                let mod_rs = dest_dir.join("mod.rs");
                let mod_content = r#"// @clrinf:generated — CRM Full Suite Root
pub mod deals;
pub mod contacts;
pub mod activities;
"#;
                fs::write(&mod_rs, mod_content)?;
                created.push(mod_rs);

                let policy_path = Self::render_crm_suite_cedar_policy(target)?;
                return Ok((created, Some(policy_path)));
            }
            "deals" | "sales" | "crm.deals" | "crm-deals" => {
                let code = format!(
                    r#"// @clrinf:generated — Adopted Domain Module: {domain_cap} ({mode:?})
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

// ─── Self-Contained Domain Contracts (Zero clrinf_core Dependency) ──────────
#[derive(Debug, Clone, PartialEq)]
pub struct OperationClaim {{
    pub resource: String,
    pub action: String,
    pub tenant_id: Option<String>,
}}

impl OperationClaim {{
    pub fn with_tenant(resource: &str, action: &str, tenant_id: &str) -> Self {{
        Self {{
            resource: resource.to_string(),
            action: action.to_string(),
            tenant_id: Some(tenant_id.to_string()),
        }}
    }}
}}

#[derive(Debug, Clone, PartialEq)]
pub struct {singular_cap} {{
    pub id: String,
    pub tenant_id: String,
    pub title: String,
    pub contact_id: String,
    pub amount: f64,
    pub stage: String,
    pub probability: u32,
}}

#[derive(Debug, Clone)]
pub struct Create{singular_cap}Command {{
    pub tenant_id: String,
    pub title: String,
    pub contact_id: String,
    pub amount: f64,
}}

#[derive(Debug, Clone)]
pub struct Change{singular_cap}StageCommand {{
    pub tenant_id: String,
    pub {domain_snake}_id: String,
    pub new_stage: String,
}}

#[derive(Debug, Clone)]
pub struct Get{singular_cap}Query {{
    pub tenant_id: String,
    pub {domain_snake}_id: String,
}}

// ─── In-Memory Thread-Safe Repository ────────────────────────────────────────
#[derive(Clone, Default)]
pub struct InMemory{singular_cap}Repository {{
    storage: Arc<RwLock<HashMap<String, {singular_cap}>>>,
}}

impl InMemory{singular_cap}Repository {{
    pub fn new() -> Self {{
        Self {{
            storage: Arc::new(RwLock::new(HashMap::new())),
        }}
    }}

    pub async fn get_by_id(&self, tenant_id: &str, id: &str) -> Option<{singular_cap}> {{
        let store = self.storage.read().await;
        store.get(&format!("{{tenant_id}}:{{id}}")).cloned()
    }}

    pub async fn save(&self, item: {singular_cap}) -> {singular_cap} {{
        let mut store = self.storage.write().await;
        let key = format!("{{}}:{{}}", item.tenant_id, item.id);
        store.insert(key, item.clone());
        item
    }}
}}

// ─── Functional Business Rules ───────────────────────────────────────────────
pub fn validate_{domain_snake}_amount(amount: f64) -> Result<(), &'static str> {{
    if amount <= 0.0 {{
        Err("Amount must be greater than zero")
    }} else {{
        Ok(())
    }}
}}

pub fn validate_stage_progression(new_stage: &str) -> Result<(), &'static str> {{
    let valid_stages = ["Prospect", "Qualified", "Proposal", "Negotiation", "ClosedWon", "ClosedLost"];
    if valid_stages.contains(&new_stage) {{
        Ok(())
    }} else {{
        Err("Invalid stage progression")
    }}
}}
"#,
                    domain_cap = domain_cap,
                    singular_cap = singular_cap,
                    domain_snake = domain_snake,
                    mode = mode
                );
                fs::write(&mod_rs, code)?;
                created.push(mod_rs);
            }
            "contacts" | "customers" | "crm.contacts" | "crm-contacts" => {
                let code = format!(
                    r#"// @clrinf:generated — Adopted Domain Module: {domain_cap} ({mode:?})
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Debug, Clone, PartialEq)]
pub struct {singular_cap} {{
    pub id: String,
    pub tenant_id: String,
    pub first_name: String,
    pub last_name: String,
    pub email: String,
    pub company: String,
    pub status: String,
}}

#[derive(Debug, Clone)]
pub struct Create{singular_cap}Command {{
    pub tenant_id: String,
    pub first_name: String,
    pub last_name: String,
    pub email: String,
    pub company: String,
}}

#[derive(Clone, Default)]
pub struct InMemory{singular_cap}Repository {{
    storage: Arc<RwLock<HashMap<String, {singular_cap}>>>,
}}

impl InMemory{singular_cap}Repository {{
    pub fn new() -> Self {{
        Self {{
            storage: Arc::new(RwLock::new(HashMap::new())),
        }}
    }}

    pub async fn get_by_id(&self, tenant_id: &str, id: &str) -> Option<{singular_cap}> {{
        let store = self.storage.read().await;
        store.get(&format!("{{tenant_id}}:{{id}}")).cloned()
    }}

    pub async fn save(&self, item: {singular_cap}) -> {singular_cap} {{
        let mut store = self.storage.write().await;
        let key = format!("{{}}:{{}}", item.tenant_id, item.id);
        store.insert(key, item.clone());
        item
    }}
}}
"#,
                    domain_cap = domain_cap,
                    singular_cap = singular_cap,
                    mode = mode
                );
                fs::write(&mod_rs, code)?;
                created.push(mod_rs);
            }
            "caching" | "cache" => {
                let code = r#"// @clrinf:generated — Pure Self-Contained In-Memory Cache
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Instant;
use tokio::sync::RwLock;

#[derive(Clone, Default)]
pub struct MemoryCache {
    store: Arc<RwLock<HashMap<String, (Vec<u8>, Option<Instant>)>>>,
}

impl MemoryCache {
    pub fn new() -> Self {
        Self {
            store: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn get(&self, key: &str) -> Option<Vec<u8>> {
        let store = self.store.read().await;
        if let Some((val, exp)) = store.get(key) {
            if let Some(deadline) = exp {
                if Instant::now() > *deadline {
                    return None;
                }
            }
            return Some(val.clone());
        }
        None
    }

    pub async fn set(&self, key: &str, value: Vec<u8>, ttl: Option<std::time::Duration>) {
        let mut store = self.store.write().await;
        let exp = ttl.map(|d| Instant::now() + d);
        store.insert(key.to_string(), (value, exp));
    }
}
"#;
                fs::write(&mod_rs, code)?;
                created.push(mod_rs);
            }
            _ => {
                let code = format!(
                    r#"// @clrinf:generated — Adopted Module: {domain_cap} ({mode:?})
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Debug, Clone, PartialEq)]
pub struct {singular_cap} {{
    pub id: String,
    pub tenant_id: String,
    pub name: String,
}}

#[derive(Clone, Default)]
pub struct InMemory{singular_cap}Repository {{
    storage: Arc<RwLock<HashMap<String, {singular_cap}>>>,
}}

impl InMemory{singular_cap}Repository {{
    pub fn new() -> Self {{
        Self {{
            storage: Arc::new(RwLock::new(HashMap::new())),
        }}
    }}

    pub async fn get_by_id(&self, tenant_id: &str, id: &str) -> Option<{singular_cap}> {{
        let store = self.storage.read().await;
        store.get(&format!("{{tenant_id}}:{{id}}")).cloned()
    }}

    pub async fn save(&self, item: {singular_cap}) -> {singular_cap} {{
        let mut store = self.storage.write().await;
        let key = format!("{{}}:{{}}", item.tenant_id, item.id);
        store.insert(key, item.clone());
        item
    }}
}}
"#,
                    domain_cap = domain_cap,
                    singular_cap = singular_cap,
                    mode = mode
                );
                fs::write(&mod_rs, code)?;
                created.push(mod_rs);
            }
        }

        // Render multi-tenant Cedar policy
        let policy_path = Self::render_cedar_policy(target, &domain_cap, &domain_lower)?;

        Ok((created, Some(policy_path)))
    }

    // ─── TypeScript Kod Adaptörü ─────────────────────────────────────────────

    fn adopt_typescript(
        target: &TargetProject,
        raw_name: &str,
        effective_name: &str,
        dest_dir: &Path,
        mode: AdoptMode,
    ) -> Result<(Vec<PathBuf>, Option<PathBuf>)> {
        let mut created = Vec::new();
        let domain_cap = effective_name.to_upper_camel_case();
        let singular_cap = if domain_cap.ends_with('s') && domain_cap.len() > 1 {
            domain_cap[..domain_cap.len() - 1].to_string()
        } else {
            domain_cap.clone()
        };
        let domain_lower = domain_cap.to_lowercase();
        let domain_camel = effective_name.to_lower_camel_case();

        let index_ts = dest_dir.join("index.ts");

        match raw_name {
            "crm" | "crm.all" => {
                let deals_dir = dest_dir.join("deals");
                let contacts_dir = dest_dir.join("contacts");
                let activities_dir = dest_dir.join("activities");
                fs::create_dir_all(&deals_dir)?;
                fs::create_dir_all(&contacts_dir)?;
                fs::create_dir_all(&activities_dir)?;

                let (d_files, _) = Self::adopt_typescript(target, "deals", "deals", &deals_dir, mode)?;
                let (c_files, _) = Self::adopt_typescript(target, "contacts", "contacts", &contacts_dir, mode)?;
                let (a_files, _) = Self::adopt_typescript(target, "activities", "activities", &activities_dir, mode)?;

                created.extend(d_files);
                created.extend(c_files);
                created.extend(a_files);

                let index_ts = dest_dir.join("index.ts");
                let index_content = r#"// @clrinf:generated — CRM Full Suite Root
export * as deals from "./deals/index.js";
export * as contacts from "./contacts/index.js";
export * as activities from "./activities/index.js";
"#;
                fs::write(&index_ts, index_content)?;
                created.push(index_ts);

                let policy_path = Self::render_crm_suite_cedar_policy(target)?;
                return Ok((created, Some(policy_path)));
            }
            "deals" | "sales" | "crm.deals" | "crm-deals" => {
                let code = format!(
                    r#"// @clrinf:generated — Adopted Domain Module: {domain_cap} ({mode:?})
// Self-contained pure TypeScript with zero forced @clrinf/core dependency

export interface OperationClaim {{
  resource: string;
  action: string;
  tenant_id?: string;
}}

export interface {singular_cap} {{
  id: string;
  tenant_id: string;
  title: string;
  contact_id: string;
  amount: number;
  stage: string;
  probability: number;
}}

export interface Create{singular_cap}Input {{
  tenant_id: string;
  title: string;
  contact_id: string;
  amount: number;
}}

export interface Change{singular_cap}StageInput {{
  tenant_id: string;
  {domain_camel}Id: string;
  new_stage: string;
}}

export class InMemory{singular_cap}Repository {{
  private storage = new Map<string, {singular_cap}>();

  async getById(tenantId: string, id: string): Promise<{singular_cap} | null> {{
    return this.storage.get(`${{tenantId}}:${{id}}`) ?? null;
  }}

  async save(item: {singular_cap}): Promise<{singular_cap}> {{
    this.storage.set(`${{item.tenant_id}}:${{item.id}}`, item);
    return item;
  }}
}}

// ─── Functional Business Rules ─────────────────────────────────────────────
export interface RuleResult {{
  passed: boolean;
  code?: string;
  message?: string;
}}

export const rulePassed = (): RuleResult => ({{ passed: true }});
export const ruleFailed = (code: string, message: string): RuleResult => ({{ passed: false, code, message }});

export const ensure{singular_cap}AmountPositive = (amount: number): RuleResult => {{
  if (amount <= 0) {{
    return ruleFailed("INVALID_{domain_cap}_AMOUNT", "{singular_cap} amount must be greater than zero.");
  }}
  return rulePassed();
}};

export const validStages = new Set(["Prospect", "Qualified", "Proposal", "Negotiation", "ClosedWon", "ClosedLost"]);

export const ensureValidStage = (stage: string): RuleResult => {{
  if (!validStages.has(stage)) {{
    return ruleFailed("INVALID_STAGE", `Stage '${{stage}}' is not valid.`);
  }}
  return rulePassed();
}};
"#,
                    domain_cap = domain_cap,
                    singular_cap = singular_cap,
                    domain_camel = domain_camel,
                    mode = mode
                );
                fs::write(&index_ts, code)?;
                created.push(index_ts);
            }
            "contacts" | "customers" | "crm.contacts" | "crm-contacts" => {
                let code = format!(
                    r#"// @clrinf:generated — Adopted Domain Module: {domain_cap} ({mode:?})

export interface {singular_cap} {{
  id: string;
  tenant_id: string;
  first_name: string;
  last_name: string;
  email: string;
  company: string;
  status: string;
}}

export interface Create{singular_cap}Input {{
  tenant_id: string;
  first_name: string;
  last_name: string;
  email: string;
  company: string;
}}

export class InMemory{singular_cap}Repository {{
  private storage = new Map<string, {singular_cap}>();

  async getById(tenantId: string, id: string): Promise<{singular_cap} | null> {{
    return this.storage.get(`${{tenantId}}:${{id}}`) ?? null;
  }}

  async save(item: {singular_cap}): Promise<{singular_cap}> {{
    this.storage.set(`${{item.tenant_id}}:${{item.id}}`, item);
    return item;
  }}
}}
"#,
                    domain_cap = domain_cap,
                    singular_cap = singular_cap,
                    mode = mode
                );
                fs::write(&index_ts, code)?;
                created.push(index_ts);
            }
            "caching" | "cache" => {
                let code = r#"// @clrinf:generated — Pure Self-Contained In-Memory Cache
export class MemoryCache {
  private store = new Map<string, { val: unknown; expiresAt: number }>();

  get<T>(key: string): T | null {
    const item = this.store.get(key);
    if (!item) return null;
    if (Date.now() > item.expiresAt) {
      this.store.delete(key);
      return null;
    }
    return item.val as T;
  }

  set<T>(key: string, value: T, ttlMs = 1800000): void {
    this.store.set(key, { val: value, expiresAt: Date.now() + ttlMs });
  }

  delete(key: string): void {
    this.store.delete(key);
  }
}
"#;
                fs::write(&index_ts, code)?;
                created.push(index_ts);
            }
            _ => {
                let code = format!(
                    r#"// @clrinf:generated — Adopted Module: {domain_cap} ({mode:?})
export interface {singular_cap} {{
  id: string;
  tenant_id: string;
  name: string;
}}

export class InMemory{singular_cap}Repository {{
  private storage = new Map<string, {singular_cap}>();

  async getById(tenantId: string, id: string): Promise<{singular_cap} | null> {{
    return this.storage.get(`${{tenantId}}:${{id}}`) ?? null;
  }}

  async save(item: {singular_cap}): Promise<{singular_cap}> {{
    this.storage.set(`${{item.tenant_id}}:${{item.id}}`, item);
    return item;
  }}
}}
"#,
                    domain_cap = domain_cap,
                    singular_cap = singular_cap,
                    mode = mode
                );
                fs::write(&index_ts, code)?;
                created.push(index_ts);
            }
        }

        // Render multi-tenant Cedar policy
        let policy_path = Self::render_cedar_policy(target, &domain_cap, &domain_lower)?;

        Ok((created, Some(policy_path)))
    }

    // ─── Multi-Tenant Cedar Policy Emitter ───────────────────────────────────

    fn render_cedar_policy(
        target: &TargetProject,
        domain_cap: &str,
        domain_lower: &str,
    ) -> Result<PathBuf> {
        let policies_dir = target.project_root.join("policies");
        fs::create_dir_all(&policies_dir)?;
        let policy_file = policies_dir.join(format!("{}.cedar", domain_lower));

        let cedar_content = format!(
            r#"// @clrinf:generated — Multi-Tenant Cedar Policy: {domain_cap}
// Language-neutral, deterministic authorization rules

// 1. Platform Admin: Super-user access across all tenants
permit (
    principal in Role::"PlatformAdmin",
    action,
    resource
);

// 2. Tenant Admin: Full administrative privileges within their own tenant
permit (
    principal in Role::"TenantAdmin",
    action,
    resource in ResourceType::"{domain_cap}"
) when {{
    resource.tenant_id == principal.tenant_id
}};

// 3. Module Manager: Full CRUD on domain resource within tenant
permit (
    principal in Role::"{domain_cap}Manager",
    action in [
        Action::"{domain_lower}.create",
        Action::"{domain_lower}.read",
        Action::"{domain_lower}.update",
        Action::"{domain_lower}.stage_change",
        Action::"{domain_lower}.delete"
    ],
    resource in ResourceType::"{domain_cap}"
) when {{
    context.tenant_id == principal.tenant_id &&
    resource.tenant_id == principal.tenant_id
}};

// 4. Sales / Domain Representative: Operational access within tenant
permit (
    principal in Role::"SalesRep",
    action in [
        Action::"{domain_lower}.create",
        Action::"{domain_lower}.read",
        Action::"{domain_lower}.stage_change"
    ],
    resource in ResourceType::"{domain_cap}"
) when {{
    context.tenant_id == principal.tenant_id &&
    resource.tenant_id == principal.tenant_id
}};

// 5. Strict Multi-Tenant Forbid Guard:
// Absolute boundary defense against cross-tenant data leakage
forbid (
    principal,
    action,
    resource in ResourceType::"{domain_cap}"
) when {{
    resource.tenant_id != principal.tenant_id
}} unless {{
    principal in Role::"PlatformAdmin"
}};
"#
        );

        fs::write(&policy_file, cedar_content)?;
        Ok(policy_file)
    }

    fn render_crm_suite_cedar_policy(target: &TargetProject) -> Result<PathBuf> {
        let policies_dir = target.project_root.join("policies");
        fs::create_dir_all(&policies_dir)?;
        let policy_file = policies_dir.join("crm.cedar");

        let cedar_content = r#"// @clrinf:generated — Multi-Tenant Cedar Policy: CRM Full Suite (Deals, Contacts, Activities)
// Language-neutral, deterministic authorization rules for complete CRM suite

// 1. Platform Admin: Super-user access across all tenants and resources
permit (
    principal in Role::"PlatformAdmin",
    action,
    resource
);

// 2. Tenant Admin: Full administrative privileges within own tenant
permit (
    principal in Role::"TenantAdmin",
    action,
    resource in [ResourceType::"Deals", ResourceType::"Contacts", ResourceType::"Activities"]
) when {
    resource.tenant_id == principal.tenant_id
};

// 3. Sales Manager: Full CRUD on CRM resources in own tenant
permit (
    principal in Role::"SalesManager",
    action in [
        Action::"deals.create", Action::"deals.read", Action::"deals.update", Action::"deals.stage_change", Action::"deals.delete",
        Action::"contacts.create", Action::"contacts.read", Action::"contacts.update", Action::"contacts.delete",
        Action::"activities.create", Action::"activities.read", Action::"activities.complete", Action::"activities.delete"
    ],
    resource in [ResourceType::"Deals", ResourceType::"Contacts", ResourceType::"Activities"]
) when {
    context.tenant_id == principal.tenant_id &&
    resource.tenant_id == principal.tenant_id
};

// 4. Sales Representative: Daily operations within own tenant
permit (
    principal in Role::"SalesRep",
    action in [
        Action::"deals.create", Action::"deals.read", Action::"deals.stage_change",
        Action::"contacts.create", Action::"contacts.read",
        Action::"activities.create", Action::"activities.read", Action::"activities.complete"
    ],
    resource in [ResourceType::"Deals", ResourceType::"Contacts", ResourceType::"Activities"]
) when {
    context.tenant_id == principal.tenant_id &&
    resource.tenant_id == principal.tenant_id
};

// 5. Strict Multi-Tenant Forbid Guard:
// Absolute boundary defense against cross-tenant data leakage across all CRM resources
forbid (
    principal,
    action,
    resource in [ResourceType::"Deals", ResourceType::"Contacts", ResourceType::"Activities"]
) when {
    resource.tenant_id != principal.tenant_id
} unless {
    principal in Role::"PlatformAdmin"
};
"#;

        fs::write(&policy_file, cedar_content)?;
        Ok(policy_file)
    }

    // ─── External Project Source Transplantation ─────────────────────────────

    fn transplant_from_external(
        source_root: &Path,
        raw_name: &str,
        effective_name: &str,
        dest_dir: &Path,
        target: &TargetProject,
        _mode: AdoptMode,
    ) -> Result<(Vec<PathBuf>, Option<PathBuf>)> {
        let mut created = Vec::new();

        // 1. Search for matching folder or file in source project
        let mut found_dir = None;
        let mut found_file = None;

        for entry in walkdir::WalkDir::new(source_root)
            .max_depth(4)
            .into_iter()
            .flatten()
        {
            let p = entry.path();
            let file_name = p.file_name().and_then(|n| n.to_str()).unwrap_or("");
            if file_name.eq_ignore_ascii_case(raw_name) {
                if p.is_dir() {
                    found_dir = Some(p.to_path_buf());
                    break;
                } else if p.is_file() {
                    found_file = Some(p.to_path_buf());
                }
            } else if file_name.to_lowercase().starts_with(&raw_name.to_lowercase())
                && (file_name.ends_with(".cs") || file_name.ends_with(".rs") || file_name.ends_with(".ts"))
            {
                if found_file.is_none() {
                    found_file = Some(p.to_path_buf());
                }
            }
        }

        if let Some(src_d) = found_dir {
            for entry in walkdir::WalkDir::new(&src_d).into_iter().flatten() {
                let p = entry.path();
                if p.is_file() {
                    let rel = diff_paths(p, &src_d).unwrap_or_else(|| p.to_path_buf());
                    let target_file = dest_dir.join(&rel);
                    if let Some(parent) = target_file.parent() {
                        fs::create_dir_all(parent)?;
                    }
                    fs::copy(p, &target_file)?;
                    created.push(target_file);
                }
            }
        } else if let Some(src_f) = found_file {
            let target_file = dest_dir.join(
                src_f
                    .file_name()
                    .unwrap_or_else(|| std::ffi::OsStr::new("module.txt")),
            );
            fs::copy(&src_f, &target_file)?;
            created.push(target_file);
        } else {
            bail!(
                "Kaynak projede '{}' modülü veya dosyası bulunamadı: {}",
                raw_name,
                source_root.display()
            );
        }

        // Render Cedar policy if not already present
        let policy_path = Self::render_cedar_policy(
            target,
            &effective_name.to_upper_camel_case(),
            &effective_name.to_lowercase(),
        )?;

        Ok((created, Some(policy_path)))
    }

    // ─── Wired Mode Helper: C# IServiceCollection Injection ──────────────────

    fn wire_csharp_project(target: &TargetProject, effective_name: &str) -> Result<PathBuf> {
        let candidates = [
            target.project_root.join("src/Common/ServiceRegistration.cs"),
            target.project_root.join("src/ServiceRegistration.cs"),
            target.project_root.join("Common/ServiceRegistration.cs"),
            target.project_root.join("ServiceRegistration.cs"),
        ];

        let reg_file = candidates
            .iter()
            .find(|p| p.is_file())
            .cloned()
            .unwrap_or_else(|| {
                let p = target.project_root.join("src/Common/ServiceRegistration.cs");
                if let Some(parent) = p.parent() {
                    let _ = fs::create_dir_all(parent);
                }
                let initial = r#"// @clrinf:generated — Service Registration
using Microsoft.Extensions.DependencyInjection;

namespace Common;

public static class ServiceRegistration
{
    public static IServiceCollection AddAppServices(this IServiceCollection services)
    {
        // @clrinf:module-registrations
        return services;
    }
}
"#;
                let _ = fs::write(&p, initial);
                p
            });

        let content = fs::read_to_string(&reg_file).unwrap_or_default();
        let call = format!(
            "services.Add{}Module();",
            effective_name.to_upper_camel_case()
        );

        if !content.contains(&call) {
            let marker = "// @clrinf:module-registrations";
            let updated = if content.contains(marker) {
                content.replace(marker, &format!("        {}\n        {}", call, marker))
            } else if content.contains("return services;") {
                content.replace(
                    "return services;",
                    &format!("        {}\n        return services;", call),
                )
            } else {
                format!("{}\n// {}\n", content, call)
            };
            fs::write(&reg_file, updated)?;
        }

        Ok(reg_file)
    }

    // ─── Wired Mode Helper: TypeScript Barrel Export Injection ───────────────

    fn wire_typescript_project(
        target: &TargetProject,
        effective_name: &str,
        rel_path: &Path,
    ) -> Result<PathBuf> {
        let index_file = target.project_root.join("src/index.ts");
        if let Some(parent) = index_file.parent() {
            fs::create_dir_all(parent)?;
        }

        let content = if index_file.is_file() {
            fs::read_to_string(&index_file)?
        } else {
            "// @clrinf:generated — Package Entry\n\n".to_string()
        };

        let norm_path = rel_path
            .to_string_lossy()
            .replace('\\', "/");
        let import_path = if norm_path.starts_with('.') {
            norm_path
        } else {
            format!("./{}", norm_path)
        };

        let export_stmt = format!(
            "export * as {} from \"{}/index.js\";",
            effective_name, import_path
        );

        if !content.contains(&export_stmt) {
            let mut updated = content.trim_end().to_string();
            if !updated.is_empty() {
                updated.push('\n');
            }
            updated.push_str(&export_stmt);
            updated.push('\n');
            fs::write(&index_file, updated)?;
        }

        Ok(index_file)
    }
}

// ─── Unit & Adaptation Tests ─────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_resolve_csharp_csproj_file() {
        let dir = tempdir().unwrap();
        let csproj = dir.path().join("BillingService.csproj");
        fs::write(&csproj, "<Project Sdk=\"Microsoft.NET.Sdk\"></Project>").unwrap();

        let resolved = TargetProject::resolve(&csproj).unwrap();
        assert_eq!(resolved.lang, Lang::CSharp);
        assert_eq!(resolved.project_name, "BillingService");
        assert_eq!(resolved.project_root, dir.path());
    }

    #[test]
    fn test_resolve_rust_cargo_toml() {
        let dir = tempdir().unwrap();
        let cargo = dir.path().join("Cargo.toml");
        fs::write(
            &cargo,
            r#"[package]
name = "order-worker"
version = "0.1.0"
edition = "2021"
"#,
        )
        .unwrap();

        let resolved = TargetProject::resolve(&cargo).unwrap();
        assert_eq!(resolved.lang, Lang::Rust);
        assert_eq!(resolved.project_name, "order-worker");
        assert_eq!(resolved.project_root, dir.path());
    }

    #[test]
    fn test_resolve_typescript_package_json() {
        let dir = tempdir().unwrap();
        let pkg = dir.path().join("package.json");
        fs::write(
            &pkg,
            r#"{"name": "@company/storefront", "version": "1.0.0"}"#,
        )
        .unwrap();

        let resolved = TargetProject::resolve(&pkg).unwrap();
        assert_eq!(resolved.lang, Lang::TypeScript);
        assert_eq!(resolved.project_name, "company-storefront");
        assert_eq!(resolved.project_root, dir.path());
    }

    #[test]
    fn test_adopt_raw_deals_into_csharp_with_alias() {
        let dir = tempdir().unwrap();
        let csproj = dir.path().join("FinanceApp.csproj");
        fs::write(&csproj, "<Project></Project>").unwrap();

        let opts = ModuleAdoptOptions {
            module: "deals".to_string(),
            to_project: csproj.clone(),
            target_dir: Some(PathBuf::from("src/Features/Sales")),
            mode: AdoptMode::Raw,
            r#as: Some("Sales".to_string()),
            source_project: None,
        };

        let result = ModuleAdapter::adopt(&opts).unwrap();
        assert_eq!(result.mode, "raw");
        assert_eq!(result.alias.as_deref(), Some("Sales"));

        let sales_module = dir.path().join("src/Features/Sales/SalesModule.cs");
        assert!(sales_module.exists());

        let content = fs::read_to_string(&sales_module).unwrap();
        assert!(content.contains("public sealed record Sale("));
        assert!(content.contains("CreateSaleCommand"));
        assert!(content.contains("public sealed record OperationClaim"));

        // Cedar policy should be rendered
        let cedar_policy = dir.path().join("policies/sales.cedar");
        assert!(cedar_policy.exists());
        let cedar_content = fs::read_to_string(&cedar_policy).unwrap();
        assert!(cedar_content.contains("ResourceType::\"Sales\""));
    }

    #[test]
    fn test_adopt_raw_caching_into_rust_crate() {
        let dir = tempdir().unwrap();
        let cargo = dir.path().join("Cargo.toml");
        fs::write(&cargo, "[package]\nname = \"my-crate\"\nversion = \"0.1.0\"\n").unwrap();

        let opts = ModuleAdoptOptions {
            module: "caching".to_string(),
            to_project: cargo,
            target_dir: Some(PathBuf::from("src/cache")),
            mode: AdoptMode::Raw,
            r#as: None,
            source_project: None,
        };

        let result = ModuleAdapter::adopt(&opts).unwrap();
        assert_eq!(result.mode, "raw");

        let cache_mod = dir.path().join("src/cache/mod.rs");
        assert!(cache_mod.exists());
        let content = fs::read_to_string(&cache_mod).unwrap();
        assert!(content.contains("pub struct MemoryCache"));
    }
}
