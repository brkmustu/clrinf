//! Event change impact analysis.
//!
//! Answers: "If I change this event, which services, versions and source files
//! are affected?" so an AI agent (or a human) only has to load the relevant
//! slice of the repository instead of everything.

use crate::scan::{collect_sources, contains_ident, ident_lines};
use crate::schema::{parse_schema_file, ParsedSchema};
use crate::validator::discover_files;
use anyhow::{bail, Context, Result};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AffectedFile {
    pub path: String,
    pub role: String,
    pub generated: bool,
    pub lines: Vec<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImpactReport {
    pub event_type: String,
    pub title: String,
    pub version: Option<String>,
    pub schema_file: String,
    pub fields: Vec<String>,
    pub publishers: Vec<String>,
    pub subscribers: Vec<String>,
    pub sibling_versions: Vec<String>,
    pub affected_files: Vec<AffectedFile>,
}

impl ImpactReport {
    pub fn format_text(&self) -> String {
        let mut out = String::new();
        out.push_str(&format!(
            "🎯 Impact of event '{}' ({})\n  schema: {}\n",
            self.event_type, self.title, self.schema_file
        ));
        out.push_str(&format!("  publishers : {}\n", list_or_none(&self.publishers)));
        out.push_str(&format!("  subscribers: {}\n", list_or_none(&self.subscribers)));
        out.push_str(&format!("  versions   : {}\n", list_or_none(&self.sibling_versions)));
        out.push_str(&format!("  fields     : {}\n", list_or_none(&self.fields)));
        out.push_str(&format!("\n  Affected files ({}):\n", self.affected_files.len()));
        for f in &self.affected_files {
            let tag = if f.generated { "generated" } else { "hand-written" };
            out.push_str(&format!("   - [{}|{}] {}\n", f.role, tag, f.path));
        }
        out
    }
}

fn list_or_none(items: &[String]) -> String {
    if items.is_empty() {
        "(none)".to_string()
    } else {
        items.join(", ")
    }
}

fn strip_version(title: &str) -> String {
    if let Some(pos) = title.rfind('V') {
        if pos > 0 && title[pos + 1..].chars().all(|c| c.is_ascii_digit()) {
            return title[..pos].to_string();
        }
    }
    title.to_string()
}

fn classify(schema: &ParsedSchema, content: &str, generated: bool) -> &'static str {
    let t = &schema.title;
    let s = &schema.file_stem;
    let publisher = [format!("publish{t}"), format!("publish_{s}"), format!("{t}Publisher")];
    let subscriber = [
        format!("{t}Handler"),
        format!("I{t}Handler"),
        format!("handle{t}Event"),
        format!("handle_{s}_event"),
        format!("{t}Subscriber"),
    ];
    let uses_pub = publisher.iter().any(|i| contains_ident(content, i));
    let uses_sub = subscriber.iter().any(|i| contains_ident(content, i));
    match (generated, uses_pub, uses_sub) {
        (true, true, _) => "publisher",
        (true, _, true) => "subscriber",
        (true, _, _) => "model",
        (false, _, true) => "handler",
        (false, true, _) => "publish-call-site",
        _ => "reference",
    }
}

/// `target` may be an event type (`billing.invoice.issued.v1`) or a schema title.
pub fn analyze(schema_dir: &Path, target: &str, source_roots: &[PathBuf]) -> Result<ImpactReport> {
    let mut schemas: Vec<(PathBuf, ParsedSchema)> = Vec::new();
    for file in discover_files(schema_dir, &["json"])
        .with_context(|| format!("Reading schemas from {}", schema_dir.display()))?
    {
        let parsed = parse_schema_file(&file)?;
        schemas.push((file, parsed));
    }

    let Some((schema_path, schema)) = schemas
        .iter()
        .find(|(_, s)| s.event_type.as_deref() == Some(target) || s.title == target)
    else {
        bail!("No event schema matches '{target}' in {}", schema_dir.display());
    };

    let family = strip_version(&schema.title);
    let mut siblings: Vec<String> = schemas
        .iter()
        .filter(|(_, s)| s.event_type.is_some() && strip_version(&s.title) == family && s.title != schema.title)
        .filter_map(|(_, s)| s.event_type.clone())
        .collect();
    siblings.sort();

    let event_type = schema.event_type.clone().unwrap_or_else(|| schema.title.clone());
    let mut affected = Vec::new();
    for root in source_roots {
        for file in collect_sources(root)? {
            let mut lines = ident_lines(&file.content, &schema.title);
            if let Some(et) = &schema.event_type {
                for (i, line) in file.content.lines().enumerate() {
                    if line.contains(et.as_str()) && !lines.contains(&(i + 1)) {
                        lines.push(i + 1);
                    }
                }
            }
            lines.sort_unstable();
            if lines.is_empty() {
                continue;
            }
            affected.push(AffectedFile {
                path: file.path.display().to_string(),
                role: classify(schema, &file.content, file.generated).to_string(),
                generated: file.generated,
                lines,
            });
        }
    }

    Ok(ImpactReport {
        event_type,
        title: schema.title.clone(),
        version: schema.version.clone(),
        schema_file: schema_path.display().to_string(),
        fields: schema.fields.iter().map(|f| f.name.clone()).collect(),
        publishers: schema.published_by.clone(),
        subscribers: schema.subscribed_by.clone(),
        sibling_versions: siblings,
        affected_files: affected,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn strip_version_handles_suffix() {
        assert_eq!(strip_version("OrderPlacedV2"), "OrderPlaced");
        assert_eq!(strip_version("OrderPlaced"), "OrderPlaced");
    }
}
