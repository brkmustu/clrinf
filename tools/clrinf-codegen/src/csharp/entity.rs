use crate::csharp::injector;
use crate::manifest::ProjectManifest;
use anyhow::{Context, Result};
use heck::ToUpperCamelCase;
use serde::Serialize;
use std::fs;
use std::path::{Path, PathBuf};
use tera::Context as TeraContext;

/// Entity property definition
#[derive(Debug, Clone, Serialize)]
pub struct EntityProperty {
    pub name: String,
    #[serde(rename = "type")]
    pub prop_type: String,
    pub namespace: Option<String>,
}

/// Entity definition for template rendering
#[derive(Debug, Clone, Serialize)]
pub struct EntityDef {
    pub name: String,
    pub name_plural: String,
    pub id_type: String,
    pub id_default: String,
    pub properties: Vec<EntityProperty>,
}

/// Options for adding an entity with vertical slice pattern
pub struct AddEntityOptions<'a> {
    pub entity_name: &'a str,
    pub module_name: &'a str,
    pub properties: &'a [(&'a str, &'a str)],
    pub id_type: Option<&'a str>,
    pub project_path: &'a Path,
    pub manifest: &'a ProjectManifest,
    pub templates_dir: &'a Path,
}

/// Result of adding an entity
pub struct AddEntityResult {
    pub files_created: Vec<PathBuf>,
    pub files_modified: Vec<PathBuf>,
}

/// Pluralize an entity name (simple English rules).
fn pluralize(name: &str) -> String {
    if name.ends_with('s') || name.ends_with("sh") || name.ends_with("ch") || name.ends_with('x') || name.ends_with('z') {
        format!("{}es", name)
    } else if name.ends_with('y') && !name.ends_with("ay") && !name.ends_with("ey") && !name.ends_with("oy") && !name.ends_with("uy") {
        format!("{}ies", &name[..name.len() - 1])
    } else {
        format!("{}s", name)
    }
}

/// Map short type names to C# full type names
fn resolve_csharp_type(short: &str) -> &str {
    match short.to_lowercase().as_str() {
        "string" | "str" => "string",
        "int" | "i32" | "integer" => "int",
        "long" | "i64" => "long",
        "float" | "f32" => "float",
        "double" | "f64" => "double",
        "decimal" | "money" => "decimal",
        "bool" | "boolean" => "bool",
        "guid" | "uuid" => "Guid",
        "datetime" | "date" => "DateTime",
        "datetimeoffset" => "DateTimeOffset",
        _ => short,
    }
}

/// Add an entity using vertical slice templates.
/// Generates <Entity>.cs, Create.cs, Update.cs, Delete.cs, GetById.cs, GetList.cs under Modules/<module>.
pub fn add_entity(opts: &AddEntityOptions) -> Result<AddEntityResult> {
    let entity_name = opts.entity_name.to_upper_camel_case();
    let module_name = opts.module_name.to_upper_camel_case();

    // Build entity definition
    let properties: Vec<EntityProperty> = opts.properties.iter().map(|(name, ptype)| {
        EntityProperty {
            name: name.to_upper_camel_case(),
            prop_type: resolve_csharp_type(ptype).to_string(),
            namespace: None,
        }
    }).collect();

    let id_type_raw = opts.id_type
        .or_else(|| {
            opts.manifest.declared_modules.iter()
                .find(|m| m.name.eq_ignore_ascii_case(&entity_name))
                .map(|m| m.id_type.as_str())
        })
        .unwrap_or("Guid");
    let id_type_cs = resolve_csharp_type(id_type_raw);
    let id_default = match id_type_cs {
        "string" => "Guid.NewGuid().ToString()",
        "int" | "long" => "0",
        _ => "Guid.NewGuid()",
    };

    let entity = EntityDef {
        name: entity_name.clone(),
        name_plural: pluralize(&entity_name),
        id_type: id_type_cs.to_string(),
        id_default: id_default.to_string(),
        properties,
    };

    // Resolve configuration from manifest
    let project_namespace = opts.manifest.project.namespace
        .as_deref()
        .unwrap_or(&opts.manifest.project.name);
    let db_context_name = opts.manifest.project.db_context
        .as_deref()
        .unwrap_or("BaseDbContext");
    let folder_name = opts.manifest.project.folder_name
        .as_deref()
        .unwrap_or("Modules");
    let is_secured = opts.manifest.project.secured;

    // Load templates via TemplateResolver (cascades pattern -> paradigm -> dispatcher with zero template conditionals)
    let resolver = crate::csharp::template_resolver::TemplateResolver::new(
        crate::csharp::template_resolver::TemplateResolutionContext {
            templates_dir: opts.templates_dir,
            pattern: opts.manifest.project.pattern,
            paradigm: opts.manifest.project.paradigm,
            dispatcher: opts.manifest.project.dispatcher,
            api_style: opts.manifest.project.api_style,
        },
    );
    let tera = resolver.build_tera()
        .with_context(|| "Failed to build Tera engine via TemplateResolver")?;

    // Build template context
    let mut ctx = TeraContext::new();
    ctx.insert("entity", &entity);
    ctx.insert("module_name", &module_name);
    ctx.insert("project_namespace", project_namespace);
    ctx.insert("db_context_name", db_context_name);
    ctx.insert("folder_name", folder_name);
    ctx.insert("is_secured", &is_secured);

    // Determine output directory: <project_path>/src/<host_or_app>/Modules/<module>
    let modules_dir = resolve_modules_dir(opts.project_path, opts.manifest)?;
    let entity_module_dir = if entity_name == module_name {
        modules_dir.join(&module_name)
    } else {
        modules_dir.join(&module_name).join(&entity_name)
    };
    fs::create_dir_all(&entity_module_dir)
        .with_context(|| format!("Failed to create module dir: {}", entity_module_dir.display()))?;

    let mut files_created = Vec::new();

    // 1. Render Entity domain model (<Entity>.cs)
    let entity_model_file = format!("{}.cs", entity_name);
    let entity_model_path = entity_module_dir.join(&entity_model_file);
    if !entity_model_path.exists() && tera.get_template_names().any(|t| t == "Entity.cs.tera") {
        let rendered = tera.render("Entity.cs.tera", &ctx)
            .with_context(|| "Failed to render Entity.cs.tera")?;
        fs::write(&entity_model_path, &rendered)
            .with_context(|| format!("Failed to write {}", entity_model_path.display()))?;
        println!("  ✅ {}", entity_model_path.display());
        files_created.push(entity_model_path);
    }

    // 2. Render slice files (Create, Update, Delete, GetById, GetList)
    let slice_files = ["Create", "Update", "Delete", "GetById", "GetList"];

    for slice_name in &slice_files {
        let template_name = format!("{}.cs.tera", slice_name);
        let output_path = entity_module_dir.join(format!("{}.cs", slice_name));

        if output_path.exists() {
            eprintln!("  ⚠️  Skipping {} (already exists)", output_path.display());
            continue;
        }

        let rendered = tera.render(&template_name, &ctx)
            .with_context(|| format!("Failed to render template {}", template_name))?;

        fs::write(&output_path, &rendered)
            .with_context(|| format!("Failed to write {}", output_path.display()))?;

        println!("  ✅ {}", output_path.display());
        files_created.push(output_path);
    }

    // 3. Inject entity into DbContext
    let mut files_modified = Vec::new();
    let persistence_layer = resolve_persistence_layer(opts.manifest);
    if let Some(ctx_path) = injector::find_dbcontext(opts.project_path, &persistence_layer, db_context_name) {
        let using_line = format!("using {}.Modules.{};", project_namespace, module_name);
        injector::add_using(&ctx_path, &using_line)?;

        if injector::add_dbset_property(&ctx_path, &entity_name, &entity.name_plural)? {
            println!("  ✅ DbSet<{}> injected into {}", entity_name, ctx_path.display());
            files_modified.push(ctx_path);
        }
    } else {
        eprintln!("  ⚠️  DbContext '{}' not found — skipping DbSet injection", db_context_name);
    }

    Ok(AddEntityResult {
        files_created,
        files_modified,
    })
}

/// Resolve the Modules/ directory based on project architecture style.
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

/// Resolve the persistence layer directory name from manifest.
fn resolve_persistence_layer(manifest: &ProjectManifest) -> String {
    use crate::manifest::ArchStyle;
    match manifest.project.arch {
        ArchStyle::Flat => {
            if let Some(ref sp) = manifest.project.src_path {
                let trimmed = sp.trim_start_matches("./").trim_start_matches('/');
                if trimmed.is_empty() {
                    manifest.project.name.clone()
                } else {
                    trimmed.to_string()
                }
            } else {
                manifest.project.name.clone()
            }
        }
        ArchStyle::Layered => format!("{}.Core", manifest.project.name),
        ArchStyle::CleanCqrs => "Persistence".to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_pluralize() {
        assert_eq!(pluralize("User"), "Users");
        assert_eq!(pluralize("Address"), "Addresses");
        assert_eq!(pluralize("Category"), "Categories");
        assert_eq!(pluralize("Key"), "Keys");
    }

    #[test]
    fn test_resolve_csharp_type() {
        assert_eq!(resolve_csharp_type("string"), "string");
        assert_eq!(resolve_csharp_type("int"), "int");
        assert_eq!(resolve_csharp_type("guid"), "Guid");
        assert_eq!(resolve_csharp_type("CustomType"), "CustomType");
    }

    #[test]
    fn test_add_entity_vertical_slice() -> Result<()> {
        let dir = tempdir()?;
        let project_dir = dir.path().join("MyProject");
        let src_app_modules = project_dir.join("src").join("Application").join("Modules");
        fs::create_dir_all(&src_app_modules)?;

        let persistence_dir = project_dir.join("Persistence");
        fs::create_dir_all(&persistence_dir)?;
        let db_context_file = persistence_dir.join("BaseDbContext.cs");
        fs::write(
            &db_context_file,
            "using Microsoft.EntityFrameworkCore;\n\npublic class BaseDbContext : DbContext\n{\n}\n",
        )?;

        let toml = r#"
[project]
name = "MyProject"
lang = "csharp"
arch = "clean-cqrs"
"#;
        let manifest = ProjectManifest::from_toml(toml)?;

        let templates_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("templates");

        let props = [("title", "string"), ("price", "decimal")];
        let opts = AddEntityOptions {
            entity_name: "Book",
            module_name: "Catalog",
            properties: &props,
            id_type: None,
            project_path: &project_dir,
            manifest: &manifest,
            templates_dir: &templates_dir,
        };

        let res = add_entity(&opts)?;
        assert_eq!(res.files_created.len(), 6); // Book.cs + 5 slice files
        assert_eq!(res.files_modified.len(), 1); // BaseDbContext.cs

        let catalog_book_dir = src_app_modules.join("Catalog").join("Book");
        assert!(catalog_book_dir.join("Book.cs").exists());
        assert!(catalog_book_dir.join("Create.cs").exists());
        assert!(catalog_book_dir.join("Update.cs").exists());
        assert!(catalog_book_dir.join("Delete.cs").exists());
        assert!(catalog_book_dir.join("GetById.cs").exists());
        assert!(catalog_book_dir.join("GetList.cs").exists());

        let create_content = fs::read_to_string(catalog_book_dir.join("Create.cs"))?;
        assert!(create_content.contains("namespace MyProject.Modules.Catalog;"));
        assert!(create_content.contains("public static class Create"));
        assert!(!create_content.contains("{% if"));
        assert!(create_content.contains("Result<Response, DomainError>"));

        let db_context_content = fs::read_to_string(&db_context_file)?;
        assert!(db_context_content.contains("public DbSet<Book> Books { get; set; }"));
        assert!(db_context_content.contains("using MyProject.Modules.Catalog;"));

        Ok(())
    }

    #[test]
    fn test_add_entity_vertical_slice_oop() -> Result<()> {
        let dir = tempdir()?;
        let project_dir = dir.path().join("OopProject");
        let src_app_modules = project_dir.join("src").join("Application").join("Modules");
        fs::create_dir_all(&src_app_modules)?;

        let persistence_dir = project_dir.join("Persistence");
        fs::create_dir_all(&persistence_dir)?;
        let db_context_file = persistence_dir.join("BaseDbContext.cs");
        fs::write(
            &db_context_file,
            "using Microsoft.EntityFrameworkCore;\n\npublic class BaseDbContext : DbContext\n{\n}\n",
        )?;

        let toml = r#"
[project]
name = "OopProject"
lang = "csharp"
arch = "clean-cqrs"
paradigm = "oop"
"#;
        let manifest = ProjectManifest::from_toml(toml)?;

        let templates_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("templates");

        let props = [("name", "string"), ("sku", "string")];
        let opts = AddEntityOptions {
            entity_name: "Item",
            module_name: "Inventory",
            properties: &props,
            id_type: None,
            project_path: &project_dir,
            manifest: &manifest,
            templates_dir: &templates_dir,
        };

        let res = add_entity(&opts)?;
        assert_eq!(res.files_created.len(), 6);

        let item_dir = src_app_modules.join("Inventory").join("Item");
        let create_content = fs::read_to_string(item_dir.join("Create.cs"))?;
        assert!(create_content.contains("ICommand<Response>"));
        assert!(create_content.contains("new Response(entity.Id)"));
        assert!(!create_content.contains("Result<"));
        assert!(!create_content.contains("{% if"));

        let update_content = fs::read_to_string(item_dir.join("Update.cs"))?;
        assert!(update_content.contains("throw new KeyNotFoundException"));
        assert!(!create_content.contains("{% if"));

        Ok(())
    }
}


