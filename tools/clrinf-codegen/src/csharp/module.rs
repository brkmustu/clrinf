use crate::csharp::injector;
use crate::manifest::ProjectManifest;
use anyhow::{Context, Result};
use heck::ToUpperCamelCase;
use std::fs;
use std::path::{Path, PathBuf};

pub struct AddModuleOptions<'a> {
    pub module_name: &'a str,
    pub project_path: &'a Path,
    pub manifest: &'a ProjectManifest,
    pub pattern: Option<crate::manifest::PatternStyle>,
}

pub struct AddModuleResult {
    pub files_created: Vec<PathBuf>,
    pub files_modified: Vec<PathBuf>,
}

/// Add a domain module to a C# project.
/// Creates the module directory, domain files, module registration class, and wires it if possible.
/// Supports two primary paradigms:
/// - 'Flat' (default): multi-file flat layout (<Module>Objects.cs, <Module>Rules.cs, <Module>Handlers.cs, <Module>Module.cs)
/// - 'Basic': single-file layout (<Module>Module.cs containing records, commands, queries, repo, rules, handlers, and DI)
pub fn add_module(opts: &AddModuleOptions) -> Result<AddModuleResult> {
    let module_name = opts.module_name.to_upper_camel_case();
    let project_namespace = opts.manifest.project.namespace
        .as_deref()
        .unwrap_or(&opts.manifest.project.name);
    let folder_name = opts.manifest.project.folder_name
        .as_deref()
        .unwrap_or("Modules");
    let pattern = opts.pattern.unwrap_or(opts.manifest.project.pattern);

    let modules_dir = resolve_modules_dir(opts.project_path, opts.manifest)?;
    let module_dir = modules_dir.join(&module_name);
    fs::create_dir_all(&module_dir)
        .with_context(|| format!("Failed to create module directory: {}", module_dir.display()))?;

    let mut files_created = Vec::new();
    let mut files_modified = Vec::new();

    match pattern {
        crate::manifest::PatternStyle::Basic => {
            // ─── Basic Pattern: Single-File per Module ──────────────────────
            let module_cs_path = module_dir.join(format!("{}Module.cs", module_name));
            if !module_cs_path.exists() {
                let content = format!(
                    r#"namespace {project_namespace}.{folder_name}.{module_name};

using System;
using System.Collections.Generic;
using System.Threading;
using System.Threading.Tasks;
using Microsoft.Extensions.DependencyInjection;

// ─── Domain Objects & Entities ──────────────────────────────────────────────
public sealed record {module_name}Entity(
    string Id,
    string TenantId,
    string Status
);

// ─── Commands & Queries ────────────────────────────────────────────────────
public sealed record Create{module_name}Command(
    string TenantId,
    string Name
);

public sealed record Get{module_name}Query(
    string TenantId,
    string Id
);

// ─── Repositories ──────────────────────────────────────────────────────────
public interface I{module_name}Repository
{{
    ValueTask<{module_name}Entity?> GetByIdAsync(string tenantId, string id);
    ValueTask<{module_name}Entity> SaveAsync({module_name}Entity entity);
}}

public sealed class InMemory{module_name}Repository : I{module_name}Repository
{{
    private readonly Dictionary<string, {module_name}Entity> _storage = new();

    public ValueTask<{module_name}Entity?> GetByIdAsync(string tenantId, string id)
    {{
        _storage.TryGetValue($"{{tenantId}}:{{id}}", out var entity);
        return ValueTask.FromResult(entity);
    }}

    public ValueTask<{module_name}Entity> SaveAsync({module_name}Entity entity)
    {{
        _storage[$"{{entity.TenantId}}:{{entity.Id}}"] = entity;
        return ValueTask.FromResult(entity);
    }}
}}

// ─── Business Rules ────────────────────────────────────────────────────────
public static class {module_name}Rules
{{
    // Define module-specific domain invariants and rules here
}}

// ─── Handlers ──────────────────────────────────────────────────────────────
public sealed class Create{module_name}Handler(I{module_name}Repository repository)
{{
    public async ValueTask<{module_name}Entity> HandleAsync(Create{module_name}Command command, CancellationToken cancellationToken = default)
    {{
        var entity = new {module_name}Entity(
            Id: Guid.NewGuid().ToString("N")[..8],
            TenantId: command.TenantId,
            Status: "Active"
        );
        return await repository.SaveAsync(entity);
    }}
}}

public sealed class Get{module_name}Handler(I{module_name}Repository repository)
{{
    public async ValueTask<{module_name}Entity?> HandleAsync(Get{module_name}Query query, CancellationToken cancellationToken = default)
    {{
        return await repository.GetByIdAsync(query.TenantId, query.Id);
    }}
}}

// ─── Module Registration ───────────────────────────────────────────────────
public static class {module_name}Module
{{
    public static IServiceCollection Add{module_name}Module(this IServiceCollection services)
    {{
        services.AddSingleton<I{module_name}Repository, InMemory{module_name}Repository>();
        services.AddTransient<Create{module_name}Handler>();
        services.AddTransient<Get{module_name}Handler>();
        return services;
    }}
}}
"#
                );
                fs::write(&module_cs_path, content)
                    .with_context(|| format!("Failed to write {}", module_cs_path.display()))?;
                println!("  ✅ {}", module_cs_path.display());
                files_created.push(module_cs_path);
            }
        }
        _ => {
            // ─── Flat Pattern (Default): Multi-file Flat Module Layout ──────
            // 1. <ModuleName>Objects.cs (Entities, Commands, Queries, Repositories)
            let objects_cs_path = module_dir.join(format!("{}Objects.cs", module_name));
            if !objects_cs_path.exists() {
                let content = format!(
                    r#"namespace {project_namespace}.{folder_name}.{module_name};

using System;
using System.Collections.Generic;
using System.Threading.Tasks;

// ─── Domain Objects & Entities ──────────────────────────────────────────────
public sealed record {module_name}Entity(
    string Id,
    string TenantId,
    string Status
);

// ─── Commands & Queries ────────────────────────────────────────────────────
public sealed record Create{module_name}Command(
    string TenantId,
    string Name
);

public sealed record Get{module_name}Query(
    string TenantId,
    string Id
);

// ─── Repositories ──────────────────────────────────────────────────────────
public interface I{module_name}Repository
{{
    ValueTask<{module_name}Entity?> GetByIdAsync(string tenantId, string id);
    ValueTask<{module_name}Entity> SaveAsync({module_name}Entity entity);
}}

public sealed class InMemory{module_name}Repository : I{module_name}Repository
{{
    private readonly Dictionary<string, {module_name}Entity> _storage = new();

    public ValueTask<{module_name}Entity?> GetByIdAsync(string tenantId, string id)
    {{
        _storage.TryGetValue($"{{tenantId}}:{{id}}", out var entity);
        return ValueTask.FromResult(entity);
    }}

    public ValueTask<{module_name}Entity> SaveAsync({module_name}Entity entity)
    {{
        _storage[$"{{entity.TenantId}}:{{entity.Id}}"] = entity;
        return ValueTask.FromResult(entity);
    }}
}}
"#
                );
                fs::write(&objects_cs_path, content)
                    .with_context(|| format!("Failed to write {}", objects_cs_path.display()))?;
                println!("  ✅ {}", objects_cs_path.display());
                files_created.push(objects_cs_path);
            }

            // 2. <ModuleName>Rules.cs (Business rules)
            let rules_cs_path = module_dir.join(format!("{}Rules.cs", module_name));
            if !rules_cs_path.exists() {
                let content = format!(
                    r#"namespace {project_namespace}.{folder_name}.{module_name};

public static class {module_name}Rules
{{
    // Define module-specific domain invariants and rules here
}}
"#
                );
                fs::write(&rules_cs_path, content)
                    .with_context(|| format!("Failed to write {}", rules_cs_path.display()))?;
                println!("  ✅ {}", rules_cs_path.display());
                files_created.push(rules_cs_path);
            }

            // 3. <ModuleName>Handlers.cs (Request handlers)
            let handlers_cs_path = module_dir.join(format!("{}Handlers.cs", module_name));
            if !handlers_cs_path.exists() {
                let content = format!(
                    r#"namespace {project_namespace}.{folder_name}.{module_name};

using System;
using System.Threading;
using System.Threading.Tasks;

public sealed class Create{module_name}Handler(I{module_name}Repository repository)
{{
    public async ValueTask<{module_name}Entity> HandleAsync(Create{module_name}Command command, CancellationToken cancellationToken = default)
    {{
        var entity = new {module_name}Entity(
            Id: Guid.NewGuid().ToString("N")[..8],
            TenantId: command.TenantId,
            Status: "Active"
        );
        return await repository.SaveAsync(entity);
    }}
}}

public sealed class Get{module_name}Handler(I{module_name}Repository repository)
{{
    public async ValueTask<{module_name}Entity?> HandleAsync(Get{module_name}Query query, CancellationToken cancellationToken = default)
    {{
        return await repository.GetByIdAsync(query.TenantId, query.Id);
    }}
}}
"#
                );
                fs::write(&handlers_cs_path, content)
                    .with_context(|| format!("Failed to write {}", handlers_cs_path.display()))?;
                println!("  ✅ {}", handlers_cs_path.display());
                files_created.push(handlers_cs_path);
            }

            // 4. <ModuleName>Module.cs (DI registration)
            let module_cs_path = module_dir.join(format!("{}Module.cs", module_name));
            if !module_cs_path.exists() {
                let content = format!(
                    r#"namespace {project_namespace}.{folder_name}.{module_name};

using Microsoft.Extensions.DependencyInjection;

public static class {module_name}Module
{{
    public static IServiceCollection Add{module_name}Module(this IServiceCollection services)
    {{
        services.AddSingleton<I{module_name}Repository, InMemory{module_name}Repository>();
        services.AddTransient<Create{module_name}Handler>();
        services.AddTransient<Get{module_name}Handler>();
        return services;
    }}
}}
"#
                );
                fs::write(&module_cs_path, content)
                    .with_context(|| format!("Failed to write {}", module_cs_path.display()))?;
                println!("  ✅ {}", module_cs_path.display());
                files_created.push(module_cs_path);
            }
        }
    }

    // 3. Attempt to wire into Program.cs if it exists
    let program_candidates = [
        opts.project_path.join("Program.cs"),
        opts.project_path.join("src").join("Api").join("Program.cs"),
        opts.project_path.join("src").join("WebAPI").join("Program.cs"),
        opts.project_path.join("src").join(project_namespace).join("Program.cs"),
        opts.project_path.join("Api").join("Program.cs"),
    ];

    for prog in &program_candidates {
        if prog.exists() {
            let using_line = format!("using {}.{}.{};", project_namespace, folder_name, module_name);
            let _ = injector::add_using(prog, &using_line);

            let wire_line = format!("builder.Services.Add{}Module();", module_name);
            if let Ok(true) = injector::add_line_to_method(prog, "Main", &wire_line) {
                files_modified.push(prog.clone());
            } else {
                // If top-level statements: check if builder.Services exists
                if let Ok(content) = fs::read_to_string(prog) {
                    if content.contains("builder.Services") && !content.contains(&wire_line) {
                        let mut new_lines = Vec::new();
                        let mut inserted = false;
                        for line in content.lines() {
                            new_lines.push(line.to_string());
                            if !inserted && (line.contains("var app = builder.Build();") || line.contains("app = builder.Build();")) {
                                new_lines.insert(new_lines.len() - 1, format!("builder.Services.Add{}Module();", module_name));
                                inserted = true;
                            }
                        }
                        if inserted {
                            let _ = fs::write(prog, new_lines.join("\n"));
                            files_modified.push(prog.clone());
                        }
                    }
                }
            }
            break;
        }
    }

    Ok(AddModuleResult {
        files_created,
        files_modified,
    })
}

fn resolve_modules_dir(project_path: &Path, manifest: &ProjectManifest) -> Result<PathBuf> {
    use crate::manifest::ArchStyle;

    let folder = manifest.project.folder_name
        .as_deref()
        .unwrap_or("Modules");

    let src = if let Some(ref sp) = manifest.project.src_path {
        let p = Path::new(sp);
        if p.is_absolute() {
            p.to_path_buf()
        } else {
            project_path.join(p)
        }
    } else {
        project_path.join("src")
    };

    let modules_dir = match manifest.project.arch {
        ArchStyle::Flat => {
            let direct = project_path.join(folder);
            let in_src = src.join(folder);
            let in_proj = src.join(&manifest.project.name).join(folder);
            if direct.is_dir() {
                direct
            } else if in_src.is_dir() {
                in_src
            } else if in_proj.is_dir() {
                in_proj
            } else if src.is_dir() {
                in_src
            } else if manifest.project.src_path.is_some() {
                in_src
            } else {
                direct
            }
        }
        ArchStyle::Layered => {
            let core_name = format!("{}.Core", manifest.project.name);
            src.join(&core_name).join(folder)
        }
        ArchStyle::CleanCqrs => {
            let direct = project_path.join(folder);
            if direct.is_dir() {
                direct
            } else {
                src.join("Application").join(folder)
            }
        }
    };

    Ok(modules_dir)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::manifest::PatternStyle;
    use tempfile::tempdir;

    #[test]
    fn test_add_module_csharp_flat_default() -> Result<()> {
        let dir = tempdir()?;
        let project_dir = dir.path().join("ShopApp");
        let src_modules = project_dir.join("Modules");
        fs::create_dir_all(&src_modules)?;

        let toml = r#"
[project]
name = "ShopApp"
lang = "csharp"
arch = "clean-cqrs"
"#;
        let manifest = ProjectManifest::from_toml(toml)?;

        let opts = AddModuleOptions {
            module_name: "Billing",
            project_path: &project_dir,
            manifest: &manifest,
            pattern: None,
        };

        let res = add_module(&opts)?;
        assert_eq!(res.files_created.len(), 4);
        assert!(src_modules.join("Billing").join("BillingObjects.cs").exists());
        assert!(src_modules.join("Billing").join("BillingRules.cs").exists());
        assert!(src_modules.join("Billing").join("BillingHandlers.cs").exists());
        assert!(src_modules.join("Billing").join("BillingModule.cs").exists());

        let module_content = fs::read_to_string(src_modules.join("Billing").join("BillingModule.cs"))?;
        assert!(module_content.contains("namespace ShopApp.Modules.Billing;"));
        assert!(module_content.contains("public static class BillingModule"));
        assert!(module_content.contains("AddBillingModule"));

        let objects_content = fs::read_to_string(src_modules.join("Billing").join("BillingObjects.cs"))?;
        assert!(objects_content.contains("public sealed record BillingEntity"));
        assert!(objects_content.contains("CreateBillingCommand"));

        Ok(())
    }

    #[test]
    fn test_add_module_csharp_basic() -> Result<()> {
        let dir = tempdir()?;
        let project_dir = dir.path().join("ShopApp");
        let src_modules = project_dir.join("Modules");
        fs::create_dir_all(&src_modules)?;

        let toml = r#"
[project]
name = "ShopApp"
lang = "csharp"
arch = "clean-cqrs"
pattern = "basic"
"#;
        let manifest = ProjectManifest::from_toml(toml)?;

        let opts = AddModuleOptions {
            module_name: "Billing",
            project_path: &project_dir,
            manifest: &manifest,
            pattern: Some(PatternStyle::Basic),
        };

        let res = add_module(&opts)?;
        assert_eq!(res.files_created.len(), 1);
        assert!(src_modules.join("Billing").join("BillingModule.cs").exists());
        assert!(!src_modules.join("Billing").join("BillingObjects.cs").exists());

        let module_content = fs::read_to_string(src_modules.join("Billing").join("BillingModule.cs"))?;
        assert!(module_content.contains("namespace ShopApp.Modules.Billing;"));
        assert!(module_content.contains("public sealed record BillingEntity"));
        assert!(module_content.contains("CreateBillingHandler"));
        assert!(module_content.contains("public static class BillingModule"));
        assert!(module_content.contains("AddBillingModule"));

        Ok(())
    }
}
