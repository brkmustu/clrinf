use crate::schema::parse_schema_file;
use anyhow::{ensure, Context, Result};
use serde_json::Value;
use std::collections::HashSet;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

pub fn discover_files(root: &Path, extensions: &[&str]) -> Result<Vec<PathBuf>> {
    ensure!(
        root.exists(),
        "Input path not found: {}. Pass an explicit schema path when using an installed CLI.",
        root.display()
    );
    let mut files = Vec::new();
    for entry in WalkDir::new(root).sort_by_file_name() {
        let entry = entry.with_context(|| format!("Traversing {}", root.display()))?;
        ensure!(
            !entry.file_type().is_symlink(),
            "Symlinked schema inputs are unsupported: {}",
            entry.path().display()
        );
        if entry.file_type().is_file()
            && entry
                .path()
                .extension()
                .and_then(|e| e.to_str())
                .is_some_and(|e| extensions.contains(&e))
        {
            files.push(entry.into_path());
        }
    }
    Ok(files)
}

pub fn validate_schemas(schema_dir: &Path) -> Result<usize> {
    let files = discover_files(schema_dir, &["json"])?;
    ensure!(
        !files.is_empty(),
        "No JSON schemas found in {}",
        schema_dir.display()
    );
    let mut ids = HashSet::new();
    let mut message_types = HashSet::new();
    for path in &files {
        let json: Value = serde_json::from_str(&std::fs::read_to_string(path)?)?;
        ensure!(
            json.is_object(),
            "Schema must be an object: {}",
            path.display()
        );
        if let Some(dialect) = json.get("$schema") {
            ensure!(
                dialect.as_str() == Some("https://json-schema.org/draft/2020-12/schema"),
                "Unsupported schema dialect in {}: only Draft 2020-12 is supported",
                path.display()
            );
        }
        jsonschema::meta::validate(&json)
            .map_err(|e| anyhow::anyhow!("Invalid JSON Schema at {}: {e}", path.display()))?;
        if let Some(id) = json.get("$id").and_then(Value::as_str) {
            ensure!(ids.insert(id.to_owned()), "Duplicate schema $id: {id}");
        }
        for key in ["x-event-type", "x-command-type", "x-query-type"] {
            if let Some(value) = json.get(key) {
                let name = value.as_str().context("Message type must be a string")?;
                ensure!(
                    message_types.insert(name.to_owned()),
                    "Duplicate message type: {name}"
                );
            }
        }
        eprintln!("  Valid: {}", path.display());
    }
    Ok(files.len())
}

pub fn check_schemas(schema_dir: &Path) -> Result<usize> {
    let count = validate_schemas(schema_dir)?;
    let mut titles = HashSet::new();
    let mut stems = HashSet::new();
    let mut ts_exports = HashSet::new();
    for path in discover_files(schema_dir, &["json"])? {
        let schema = parse_schema_file(&path)?;
        ensure!(
            titles.insert(schema.title.clone()),
            "Duplicate generated type: {}",
            schema.title
        );
        ensure!(
            stems.insert(schema.file_stem.clone()),
            "Duplicate generated filename: {}",
            schema.file_stem
        );
        ensure!(
            ts_exports.insert(schema.title.clone()),
            "Duplicate TypeScript export: {}",
            schema.title
        );
        let suffix = if schema.event_type.is_some() {
            Some("Type")
        } else if schema.command_type.is_some() {
            Some("CommandType")
        } else if schema.query_type.is_some() {
            Some("QueryType")
        } else {
            None
        };
        if let Some(suffix) = suffix {
            let name = format!("{}{suffix}", schema.title);
            ensure!(
                ts_exports.insert(name.clone()),
                "Duplicate TypeScript export: {name}"
            );
        }
    }
    Ok(count)
}
