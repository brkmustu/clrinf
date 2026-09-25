// @clrinf:generated — Language-Idiomatic Project Module Wiring
//
// Dil Felsefesi:
// • C#: .NET ekosisteminin doğal standardı olan IServiceCollection DI uzantıları (IoC).
// • Rust: Derleme zamanı modül ağacı (`pub mod ...;`) ve açık struct/trait kompozisyonu.
//         Rust'ta çalışma zamanı DI konteyneri YOKTUR; bileşenler derleme zamanında bağlanır.
// • TypeScript: ESM barrel export'ları (`export * as ...`) ve fonksiyonel middleware composition.

use anyhow::{Context, Result};
use heck::{ToSnakeCase, ToUpperCamelCase};
use std::fs;
use std::path::{Path, PathBuf};

pub struct ModuleWiring;

impl ModuleWiring {
    /// Projeye yeni bir modül eklendiğinde, ilgili dilin kendi doğal yapısına uygun olarak bağlar.
    pub fn wire(project_dir: &Path, lang: &str, module_name: &str) -> Result<()> {
        match lang.to_lowercase().as_str() {
            "csharp" | "cs" | "dotnet" => Self::wire_csharp_service_registration(project_dir, module_name),
            "rust" | "rs" => Self::wire_rust_module_tree(project_dir, module_name),
            "typescript" | "ts" | "js" => Self::wire_typescript_barrel_export(project_dir, module_name),
            _ => Ok(()),
        }
    }

    /// Projeden bir modül çıkarıldığında, ilgili dilin kendi doğal yapısından bağlantısını kaldırır.
    pub fn unwire(project_dir: &Path, lang: &str, module_name: &str) -> Result<()> {
        match lang.to_lowercase().as_str() {
            "csharp" | "cs" | "dotnet" => Self::unwire_csharp_service_registration(project_dir, module_name),
            "rust" | "rs" => Self::unwire_rust_module_tree(project_dir, module_name),
            "typescript" | "ts" | "js" => Self::unwire_typescript_barrel_export(project_dir, module_name),
            _ => Ok(()),
        }
    }

    // ─── C# Doğası: IServiceCollection Service Registration ────────────────────

    fn wire_csharp_service_registration(project_dir: &Path, module_name: &str) -> Result<()> {
        let reg_file = Self::find_or_create_csharp_registration_file(project_dir)?;
        let content = fs::read_to_string(&reg_file).unwrap_or_default();

        let call = match module_name {
            "caching" => "services.AddClrinfCaching();".to_string(),
            "logging" => "services.AddClrinfLogging();".to_string(),
            "transaction" => "services.AddClrinfTransaction();".to_string(),
            "authentication" | "auth" => "services.AddClrinfAuth();".to_string(),
            "authorization" | "authz" => "services.AddClrinfCedarAuthorization();".to_string(),
            other => format!("services.Add{}Module();", other.to_upper_camel_case()),
        };

        if !content.contains(&call) {
            let marker = "// @clrinf:module-registrations";
            let updated = if content.contains(marker) {
                content.replace(marker, &format!("        {}\n        {}", call, marker))
            } else if content.contains("return services;") {
                content.replace("return services;", &format!("        {}\n        return services;", call))
            } else {
                format!("{}\n// {}\n", content, call)
            };
            fs::write(&reg_file, updated).with_context(|| format!("Failed to update {}", reg_file.display()))?;
        }

        Ok(())
    }

    fn unwire_csharp_service_registration(project_dir: &Path, module_name: &str) -> Result<()> {
        if let Ok(reg_file) = Self::find_csharp_registration_file(project_dir) {
            let content = fs::read_to_string(&reg_file)?;
            let call = match module_name {
                "caching" => "services.AddClrinfCaching();".to_string(),
                "logging" => "services.AddClrinfLogging();".to_string(),
                "transaction" => "services.AddClrinfTransaction();".to_string(),
                "authentication" | "auth" => "services.AddClrinfAuth();".to_string(),
                "authorization" | "authz" => "services.AddClrinfCedarAuthorization();".to_string(),
                other => format!("services.Add{}Module();", other.to_upper_camel_case()),
            };

            if content.contains(&call) {
                let updated: Vec<String> = content
                    .lines()
                    .filter(|l| !l.trim().contains(&call))
                    .map(|l| l.to_string())
                    .collect();
                fs::write(&reg_file, updated.join("\n") + "\n")?;
            }
        }
        Ok(())
    }

    fn find_csharp_registration_file(project_dir: &Path) -> Result<PathBuf> {
        let candidates = [
            project_dir.join("src/Common/ServiceRegistration.cs"),
            project_dir.join("src/Common/DependencyInjection.cs"),
            project_dir.join("Common/ServiceRegistration.cs"),
            project_dir.join("ServiceRegistration.cs"),
        ];

        for c in &candidates {
            if c.is_file() {
                return Ok(c.clone());
            }
        }

        for entry in walkdir::WalkDir::new(project_dir)
            .max_depth(4)
            .into_iter()
            .flatten()
        {
            let p = entry.path();
            if p.is_file() {
                let file_name = p.file_name().and_then(|n| n.to_str()).unwrap_or("");
                if file_name == "ServiceRegistration.cs" || file_name == "DependencyInjection.cs" {
                    return Ok(p.to_path_buf());
                }
            }
        }

        anyhow::bail!("C# ServiceRegistration.cs not found")
    }

    fn find_or_create_csharp_registration_file(project_dir: &Path) -> Result<PathBuf> {
        if let Ok(file) = Self::find_csharp_registration_file(project_dir) {
            return Ok(file);
        }

        let common_dir = project_dir.join("src/Common");
        fs::create_dir_all(&common_dir)?;
        let di_file = common_dir.join("ServiceRegistration.cs");

        let content = r#"// @clrinf:generated — C# Service Registration
using Microsoft.Extensions.DependencyInjection;

namespace Common;

public static class ServiceRegistration
{
    public static IServiceCollection AddClrinfModules(this IServiceCollection services)
    {
        // @clrinf:module-registrations
        return services;
    }
}
"#;
        fs::write(&di_file, content)?;
        Ok(di_file)
    }

    // ─── Rust Doğası: Modül Ağacı (pub mod ...) Deklarasyonu ────────────────────

    fn wire_rust_module_tree(project_dir: &Path, module_name: &str) -> Result<()> {
        let entry_file = Self::find_or_create_rust_entry_file(project_dir)?;
        let content = fs::read_to_string(&entry_file).unwrap_or_default();

        let mod_decl = match module_name {
            "caching" => "pub mod caching;".to_string(),
            "logging" => "pub mod logging;".to_string(),
            "transaction" => "pub mod transaction;".to_string(),
            "authentication" | "auth" => "pub mod auth;".to_string(),
            "authorization" | "authz" => "pub mod authz;".to_string(),
            other => format!("pub mod {};", other.to_snake_case()),
        };

        if !content.contains(&mod_decl) {
            let mut lines: Vec<String> = content.lines().map(|l| l.to_string()).collect();
            let mut insert_idx = 0;
            for (idx, line) in lines.iter().enumerate() {
                if line.starts_with("//") || line.starts_with("#!") || line.is_empty() {
                    insert_idx = idx + 1;
                } else {
                    break;
                }
            }
            lines.insert(insert_idx, mod_decl);
            fs::write(&entry_file, lines.join("\n") + "\n")?;
        }

        Ok(())
    }

    fn unwire_rust_module_tree(project_dir: &Path, module_name: &str) -> Result<()> {
        if let Ok(entry_file) = Self::find_rust_entry_file(project_dir) {
            let content = fs::read_to_string(&entry_file)?;
            let mod_decl = match module_name {
                "caching" => "pub mod caching;".to_string(),
                "logging" => "pub mod logging;".to_string(),
                "transaction" => "pub mod transaction;".to_string(),
                "authentication" | "auth" => "pub mod auth;".to_string(),
                "authorization" | "authz" => "pub mod authz;".to_string(),
                other => format!("pub mod {};", other.to_snake_case()),
            };

            if content.contains(&mod_decl) {
                let updated: Vec<String> = content
                    .lines()
                    .filter(|l| !l.trim().contains(&mod_decl))
                    .map(|l| l.to_string())
                    .collect();
                fs::write(&entry_file, updated.join("\n") + "\n")?;
            }
        }
        Ok(())
    }

    fn find_rust_entry_file(project_dir: &Path) -> Result<PathBuf> {
        let lib_rs = project_dir.join("src/lib.rs");
        if lib_rs.is_file() {
            return Ok(lib_rs);
        }
        let main_rs = project_dir.join("src/main.rs");
        if main_rs.is_file() {
            return Ok(main_rs);
        }
        anyhow::bail!("Rust entry file (src/lib.rs or src/main.rs) not found")
    }

    fn find_or_create_rust_entry_file(project_dir: &Path) -> Result<PathBuf> {
        if let Ok(file) = Self::find_rust_entry_file(project_dir) {
            return Ok(file);
        }
        let src = project_dir.join("src");
        fs::create_dir_all(&src)?;
        let lib_rs = src.join("lib.rs");
        fs::write(&lib_rs, "// @clrinf:generated — Rust Library Root\n\n")?;
        Ok(lib_rs)
    }

    // ─── TypeScript Doğası: ESM Barrel Export'ları ────────────────────────────

    fn wire_typescript_barrel_export(project_dir: &Path, module_name: &str) -> Result<()> {
        let index_file = Self::find_or_create_typescript_entry_file(project_dir)?;
        let content = fs::read_to_string(&index_file).unwrap_or_default();

        let export_stmt = match module_name {
            "caching" => "export * as caching from \"./caching/index.js\";",
            "logging" => "export * as logging from \"./logging/index.js\";",
            "transaction" => "export * as transaction from \"./transaction/index.js\";",
            "authentication" | "auth" => "export * as auth from \"./auth/index.js\";",
            "authorization" | "authz" => "export * as authz from \"./authz/index.js\";",
            _ => return Ok(()),
        };

        if !content.contains(export_stmt) {
            let mut updated = content.trim_end().to_string();
            if !updated.is_empty() {
                updated.push('\n');
            }
            updated.push_str(export_stmt);
            updated.push('\n');
            fs::write(&index_file, updated)?;
        }

        Ok(())
    }

    fn unwire_typescript_barrel_export(project_dir: &Path, module_name: &str) -> Result<()> {
        if let Ok(index_file) = Self::find_typescript_entry_file(project_dir) {
            let content = fs::read_to_string(&index_file)?;
            let export_prefix = match module_name {
                "caching" => "caching",
                "logging" => "logging",
                "transaction" => "transaction",
                "authentication" | "auth" => "auth",
                "authorization" | "authz" => "authz",
                _ => return Ok(()),
            };

            let updated: Vec<String> = content
                .lines()
                .filter(|l| !l.contains(&format!("./{}/", export_prefix)))
                .map(|l| l.to_string())
                .collect();
            fs::write(&index_file, updated.join("\n") + "\n")?;
        }
        Ok(())
    }

    fn find_typescript_entry_file(project_dir: &Path) -> Result<PathBuf> {
        let index_ts = project_dir.join("src/index.ts");
        if index_ts.is_file() {
            return Ok(index_ts);
        }
        let main_ts = project_dir.join("src/main.ts");
        if main_ts.is_file() {
            return Ok(main_ts);
        }
        anyhow::bail!("TypeScript entry file (src/index.ts or src/main.ts) not found")
    }

    fn find_or_create_typescript_entry_file(project_dir: &Path) -> Result<PathBuf> {
        if let Ok(file) = Self::find_typescript_entry_file(project_dir) {
            return Ok(file);
        }
        let src = project_dir.join("src");
        fs::create_dir_all(&src)?;
        let index_ts = src.join("index.ts");
        fs::write(&index_ts, "// @clrinf:generated — TypeScript Package Entry\n\n")?;
        Ok(index_ts)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_rust_module_tree_wiring() {
        let tmp = tempdir().unwrap();
        let project_dir = tmp.path();

        ModuleWiring::wire(project_dir, "rust", "caching").unwrap();
        let lib_rs = project_dir.join("src/lib.rs");
        assert!(lib_rs.is_file());
        let content = fs::read_to_string(&lib_rs).unwrap();
        assert!(content.contains("pub mod caching;"));

        ModuleWiring::wire(project_dir, "rust", "authz").unwrap();
        let content2 = fs::read_to_string(&lib_rs).unwrap();
        assert!(content2.contains("pub mod caching;"));
        assert!(content2.contains("pub mod authz;"));

        ModuleWiring::unwire(project_dir, "rust", "caching").unwrap();
        let content3 = fs::read_to_string(&lib_rs).unwrap();
        assert!(!content3.contains("pub mod caching;"));
        assert!(content3.contains("pub mod authz;"));
    }

    #[test]
    fn test_typescript_barrel_export_wiring() {
        let tmp = tempdir().unwrap();
        let project_dir = tmp.path();

        ModuleWiring::wire(project_dir, "typescript", "caching").unwrap();
        let index_ts = project_dir.join("src/index.ts");
        assert!(index_ts.is_file());
        let content = fs::read_to_string(&index_ts).unwrap();
        assert!(content.contains("./caching/index.js"));

        ModuleWiring::unwire(project_dir, "typescript", "caching").unwrap();
        let content2 = fs::read_to_string(&index_ts).unwrap();
        assert!(!content2.contains("./caching/index.js"));
    }

    #[test]
    fn test_csharp_service_registration_wiring() {
        let tmp = tempdir().unwrap();
        let project_dir = tmp.path();

        ModuleWiring::wire(project_dir, "csharp", "caching").unwrap();
        let di_file = project_dir.join("src/Common/ServiceRegistration.cs");
        assert!(di_file.is_file());
        let content = fs::read_to_string(&di_file).unwrap();
        assert!(content.contains("services.AddClrinfCaching();"));

        ModuleWiring::unwire(project_dir, "csharp", "caching").unwrap();
        let content2 = fs::read_to_string(&di_file).unwrap();
        assert!(!content2.contains("services.AddClrinfCaching();"));
    }
}
