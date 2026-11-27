use serde::Serialize;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use syn::{GenericArgument, Item, PathArguments, Type};
use walkdir::WalkDir;

const OP_PAIRS: &[(&str, &str)] = &[
    ("Create", "create"),
    ("GetById", "get_by_id"),
    ("GetList", "get_list"),
    ("Update", "update"),
    ("Delete", "delete"),
];

const SKIP_DIRS: &[&str] = &["target", ".git", "node_modules", ".vs", "bin", "obj"];

#[derive(Debug, Clone, Serialize)]
pub struct CrudRuleInfo {
    pub name: String,
    pub hook: String,
    pub file: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct CrudEntityMap {
    pub entity: String,
    pub file: String,
    pub route_prefix: String,
    pub generated: Vec<String>,
    pub ejected: Vec<String>,
    pub rules: Vec<CrudRuleInfo>,
}

#[derive(Debug, Clone, Serialize)]
pub struct CrudMapReport {
    pub language: String,
    pub root: String,
    pub entities: Vec<CrudEntityMap>,
}

#[derive(Debug, Clone, Serialize)]
pub struct EjectReport {
    pub language: String,
    pub entity: String,
    pub operation: String,
    pub route_prefix: String,
    pub entity_file: String,
    pub handler_file: String,
}

pub fn canonical_op(raw: &str) -> Option<(&'static str, &'static str)> {
    let lower = raw.trim().to_ascii_lowercase();
    match lower.as_str() {
        "create" => Some(("Create", "create")),
        "get_by_id" | "getbyid" => Some(("GetById", "get_by_id")),
        "get_list" | "getlist" => Some(("GetList", "get_list")),
        "update" => Some(("Update", "update")),
        "delete" => Some(("Delete", "delete")),
        _ => None,
    }
}

pub fn to_snake_case(pascal: &str) -> String {
    let mut out = String::new();
    for (i, ch) in pascal.chars().enumerate() {
        if ch.is_uppercase() {
            if i > 0 {
                out.push('_');
            }
            for lc in ch.to_lowercase() {
                out.push(lc);
            }
        } else {
            out.push(ch);
        }
    }
    out
}

fn pluralize_snake(pascal: &str) -> String {
    let snake = to_snake_case(pascal);
    if snake.ends_with('s') {
        format!("{snake}es")
    } else if let Some(stem) = snake.strip_suffix('y') {
        if !stem.ends_with(['a', 'e', 'i', 'o', 'u']) {
            return format!("{stem}ies");
        }
        format!("{snake}s")
    } else {
        format!("{snake}s")
    }
}

fn has_derive_clrinf_crud(attrs: &[syn::Attribute]) -> bool {
    attrs.iter().any(|attr| {
        if !attr.path().is_ident("derive") {
            return false;
        }
        let mut found = false;
        let _ = attr.parse_nested_meta(|meta| {
            if meta
                .path
                .segments
                .last()
                .is_some_and(|s| s.ident == "ClrinfCrud")
            {
                found = true;
            }
            Ok(())
        });
        found
    })
}

fn extract_clrinf_crud_meta(attrs: &[syn::Attribute], entity: &str) -> (String, Vec<String>) {
    let mut route: Option<String> = None;
    let mut skipped: Vec<String> = Vec::new();

    for attr in attrs {
        if !attr.path().is_ident("clrinf_crud") {
            continue;
        }
        let _ = attr.parse_nested_meta(|meta| {
            if meta.path.is_ident("route") {
                if let Ok(val) = meta.value() {
                    if let Ok(s) = val.parse::<syn::LitStr>() {
                        route = Some(s.value());
                    }
                }
            } else if meta.path.is_ident("skip") {
                let _ = meta.parse_nested_meta(|nested| {
                    if let Some(ident) = nested.path.get_ident() {
                        if let Some((pascal, _)) = canonical_op(&ident.to_string()) {
                            let s = pascal.to_string();
                            if !skipped.contains(&s) {
                                skipped.push(s);
                            }
                        }
                    }
                    Ok(())
                });
            }
            Ok(())
        });
    }

    let route_prefix = route.unwrap_or_else(|| format!("/api/{}", pluralize_snake(entity)));
    (route_prefix, skipped)
}

fn type_last_ident(ty: &Type) -> Option<String> {
    match ty {
        Type::Path(tp) => tp.path.segments.last().map(|s| s.ident.to_string()),
        _ => None,
    }
}

fn rel_slash(root: &Path, path: &Path) -> String {
    path.strip_prefix(root)
        .unwrap_or(path)
        .to_string_lossy()
        .replace('\\', "/")
}

fn discover_rs_files(root: &Path) -> Vec<PathBuf> {
    if root.is_file() {
        return vec![root.to_path_buf()];
    }
    let mut files = Vec::new();
    let walker = WalkDir::new(root)
        .sort_by_file_name()
        .into_iter()
        .filter_entry(|e| {
            if e.depth() == 0 || !e.file_type().is_dir() {
                return true;
            }
            e.file_name()
                .to_str()
                .is_none_or(|n| !SKIP_DIRS.contains(&n))
        });
    for entry in walker.flatten() {
        if entry.file_type().is_file() && entry.path().extension().is_some_and(|ext| ext == "rs") {
            files.push(entry.into_path());
        }
    }
    files
}

pub fn build_crud_map(root: &Path) -> Result<CrudMapReport, Box<dyn std::error::Error>> {
    let scan_root = if root.is_file() {
        root.parent().unwrap_or_else(|| Path::new("."))
    } else {
        root
    };
    let mut entities = Vec::new();
    let mut rules_by_entity: BTreeMap<String, Vec<CrudRuleInfo>> = BTreeMap::new();

    for path in discover_rs_files(scan_root) {
        let Ok(content) = std::fs::read_to_string(&path) else {
            continue;
        };
        let Ok(parsed) = syn::parse_file(&content) else {
            continue;
        };
        let rel = rel_slash(scan_root, &path);

        for item in &parsed.items {
            match item {
                Item::Struct(s) if has_derive_clrinf_crud(&s.attrs) => {
                    let entity_name = s.ident.to_string();
                    let (route_prefix, ejected) = extract_clrinf_crud_meta(&s.attrs, &entity_name);
                    let generated = OP_PAIRS
                        .iter()
                        .map(|(pascal, _)| (*pascal).to_string())
                        .filter(|op| !ejected.contains(op))
                        .collect();
                    entities.push(CrudEntityMap {
                        entity: entity_name,
                        file: rel.clone(),
                        route_prefix,
                        generated,
                        ejected,
                        rules: Vec::new(),
                    });
                }
                Item::Impl(imp) => {
                    let Some((_, trait_path, _)) = &imp.trait_ else {
                        continue;
                    };
                    let Some(last_seg) = trait_path.segments.last() else {
                        continue;
                    };
                    let hook = match last_seg.ident.to_string().as_str() {
                        "BeforeCreateRule" => "before-create",
                        "BeforeUpdateRule" => "before-update",
                        "BeforeDeleteRule" => "before-delete",
                        _ => continue,
                    };
                    let target_entity = match &last_seg.arguments {
                        PathArguments::AngleBracketed(ab) => ab.args.iter().find_map(|arg| {
                            if let GenericArgument::Type(ty) = arg {
                                type_last_ident(ty)
                            } else {
                                None
                            }
                        }),
                        _ => None,
                    };
                    let Some(entity_name) = target_entity else {
                        continue;
                    };
                    let rule_name = type_last_ident(&imp.self_ty).unwrap_or_else(|| "Rule".into());
                    rules_by_entity
                        .entry(entity_name)
                        .or_default()
                        .push(CrudRuleInfo {
                            name: rule_name,
                            hook: hook.into(),
                            file: rel.clone(),
                        });
                }
                _ => {}
            }
        }
    }

    for entity in &mut entities {
        if let Some(rules) = rules_by_entity.remove(&entity.entity) {
            entity.rules = rules;
        }
    }
    entities.sort_by(|a, b| a.entity.cmp(&b.entity));

    Ok(CrudMapReport {
        language: "rust".into(),
        root: scan_root.display().to_string(),
        entities,
    })
}

fn find_struct_line(lines: &[&str], entity: &str) -> Option<usize> {
    let needle = format!("struct {entity}");
    lines.iter().position(|line| {
        let trimmed = line.trim_start();
        !trimmed.starts_with("//")
            && trimmed.contains(&needle)
            && trimmed.split_whitespace().any(|tok| {
                tok == entity
                    || tok.starts_with(&format!("{entity}<"))
                    || tok.starts_with(&format!("{entity}{{"))
            })
    })
}

fn rewrite_skip_attribute(content: &str, entity: &str, op_snake: &str) -> Result<String, String> {
    let eol = if content.contains("\r\n") {
        "\r\n"
    } else {
        "\n"
    };
    let mut lines: Vec<String> = content.lines().map(ToString::to_string).collect();
    let line_refs: Vec<&str> = lines.iter().map(String::as_str).collect();
    let Some(struct_idx) = find_struct_line(&line_refs, entity) else {
        return Err(format!("Could not locate 'struct {entity}' declaration"));
    };

    // Walk contiguous attribute lines right above the struct declaration.
    let mut attr_start = struct_idx;
    while attr_start > 0 {
        let prev = lines[attr_start - 1].trim();
        if prev.starts_with('#') || prev.starts_with("///") {
            attr_start -= 1;
        } else {
            break;
        }
    }

    if let Some(idx) =
        (attr_start..struct_idx).find(|&i| lines[i].trim_start().starts_with("#[clrinf_crud"))
    {
        let line = &lines[idx];
        if let (Some(open), Some(close)) = (line.find('('), line.rfind(')')) {
            let inner = &line[open + 1..close];
            if let Some(skip_pos) = inner.find("skip(") {
                let after_skip = &inner[skip_pos + 5..];
                if let Some(skip_end_rel) = after_skip.find(')') {
                    let existing = &after_skip[..skip_end_rel];
                    let already = existing
                        .split(',')
                        .any(|part| canonical_op(part).is_some_and(|(_, s)| s == op_snake));
                    if !already {
                        let insert_at = open + 1 + skip_pos + 5 + skip_end_rel;
                        let addition = if existing.trim().is_empty() {
                            op_snake.to_string()
                        } else {
                            format!(", {op_snake}")
                        };
                        let mut updated = line.clone();
                        updated.insert_str(insert_at, &addition);
                        lines[idx] = updated;
                    }
                }
            } else {
                let addition = if inner.trim().is_empty() {
                    format!("skip({op_snake})")
                } else {
                    format!(", skip({op_snake})")
                };
                let mut updated = line.clone();
                updated.insert_str(close, &addition);
                lines[idx] = updated;
            }
        }
    } else {
        let indent: String = lines[struct_idx]
            .chars()
            .take_while(|c| c.is_whitespace())
            .collect();
        lines.insert(
            struct_idx,
            format!("{indent}#[clrinf_crud(skip({op_snake}))]"),
        );
    }

    let mut out = lines.join(eol);
    if content.ends_with('\n') {
        out.push_str(eol);
    }
    Ok(out)
}

fn render_ejected_handler(
    entity: &str,
    op_pascal: &str,
    op_snake: &str,
    route_prefix: &str,
) -> String {
    let route_suffix = match op_snake {
        "create" | "get_list" => route_prefix.to_string(),
        _ => format!("{route_prefix}/:id"),
    };
    let (http_method, axum_fn) = match op_snake {
        "create" => ("POST", "post"),
        "get_by_id" => ("GET", "get"),
        "get_list" => ("GET", "get"),
        "update" => ("PUT", "put"),
        "delete" => ("DELETE", "delete"),
        _ => ("GET", "get"),
    };
    let handler_fn = format!("{op_snake}_{}", to_snake_case(entity));
    format!(
        r#"//! Custom {op_pascal} handler ejected from `#[derive(ClrinfCrud)]` for `{entity}`.
//! Mount in your router alongside `crud_router::<{entity}, _>(store, rules)`:
//!   `.route("{route_suffix}", axum::routing::{axum_fn}({handler_fn}))`

use axum::{{http::StatusCode, Extension, Json}};
use clrinf_adapters::AppError;
use clrinf_core::RequestContext;

pub async fn {handler_fn}(
    _ctx: Option<Extension<RequestContext>>,
) -> Result<(StatusCode, Json<serde_json::Value>), AppError> {{
    // TODO: Implement custom {http_method} {route_suffix} behavior for {entity}.
    Ok((
        StatusCode::OK,
        Json(serde_json::json!({{
            "entity": "{entity}",
            "operation": "{op_pascal}",
            "status": "ejected"
        }})),
    ))
}}
"#
    )
}

pub fn eject_operation(
    root: &Path,
    entity_name: &str,
    op_raw: &str,
    force: bool,
) -> Result<EjectReport, Box<dyn std::error::Error>> {
    let (op_pascal, op_snake) = canonical_op(op_raw).ok_or_else(|| {
        format!(
            "Unknown operation '{op_raw}'. Expected Create, Update, Delete, GetById, or GetList."
        )
    })?;

    let scan_root = if root.is_file() {
        root.parent().unwrap_or_else(|| Path::new("."))
    } else {
        root
    };
    let map = build_crud_map(scan_root)?;
    let entry = map
        .entities
        .iter()
        .find(|e| e.entity.eq_ignore_ascii_case(entity_name))
        .ok_or_else(|| {
            format!(
                "Entity '{entity_name}' with #[derive(ClrinfCrud)] not found under {}",
                scan_root.display()
            )
        })?;

    let entity_path = scan_root.join(&entry.file);
    let content = std::fs::read_to_string(&entity_path)?;
    let updated = rewrite_skip_attribute(&content, &entry.entity, op_snake)?;
    if updated != content {
        std::fs::write(&entity_path, updated)?;
    }

    let base_dir = if scan_root.join("src").is_dir() {
        scan_root.join("src/features")
    } else {
        scan_root.join("features")
    };
    let handler_dir = base_dir.join(to_snake_case(&entry.entity));
    std::fs::create_dir_all(&handler_dir)?;
    let handler_path = handler_dir.join(format!("{op_snake}.rs"));
    if handler_path.exists() && !force {
        return Err(format!(
            "Refusing to overwrite existing handler {} without --force",
            handler_path.display()
        )
        .into());
    }
    std::fs::write(
        &handler_path,
        render_ejected_handler(&entry.entity, op_pascal, op_snake, &entry.route_prefix),
    )?;

    Ok(EjectReport {
        language: "rust".into(),
        entity: entry.entity.clone(),
        operation: op_pascal.into(),
        route_prefix: entry.route_prefix.clone(),
        entity_file: rel_slash(scan_root, &entity_path),
        handler_file: rel_slash(scan_root, &handler_path),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn crud_map_and_eject_rewrites_skip_and_emits_handler() {
        let dir = std::env::temp_dir().join(format!("clrinfrs-crud-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(dir.join("src/domain")).unwrap();
        std::fs::create_dir_all(dir.join("src/rules")).unwrap();

        std::fs::write(
            dir.join("src/domain/category.rs"),
            r#"use clrinf_macros::ClrinfCrud;

#[derive(Debug, Clone, ClrinfCrud)]
#[clrinf_crud(route = "/api/categories")]
pub struct Category {
    pub id: String,
    pub name: String,
    pub tenant_id: String,
}
"#,
        )
        .unwrap();

        std::fs::write(
            dir.join("src/rules/check.rs"),
            r#"pub struct ActiveCheck;
impl BeforeDeleteRule<Category> for ActiveCheck {}
"#,
        )
        .unwrap();

        let before = build_crud_map(&dir).unwrap();
        assert_eq!(before.entities.len(), 1);
        assert_eq!(before.entities[0].generated.len(), 5);
        assert!(before.entities[0].ejected.is_empty());
        assert_eq!(before.entities[0].rules.len(), 1);
        assert_eq!(before.entities[0].rules[0].hook, "before-delete");

        let ej = eject_operation(&dir, "Category", "Update", false).unwrap();
        assert_eq!(ej.operation, "Update");
        assert_eq!(ej.handler_file, "src/features/category/update.rs");
        assert!(dir.join("src/features/category/update.rs").is_file());

        let entity_src = std::fs::read_to_string(dir.join("src/domain/category.rs")).unwrap();
        assert!(entity_src.contains("skip(update)"));

        let after = build_crud_map(&dir).unwrap();
        assert_eq!(after.entities[0].ejected, vec!["Update".to_string()]);
        assert!(!after.entities[0].generated.contains(&"Update".to_string()));

        let _ = std::fs::remove_dir_all(dir);
    }
}
