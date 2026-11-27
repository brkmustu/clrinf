//! Event change impact analysis.
//!
//! Answers: "If I change this event, which services, versions and source files
//! are affected?" so an AI agent (or a human) only has to load the relevant
//! slice of the repository instead of everything.

use crate::scan::{collect_sources, contains_ident, event_annotations, ident_lines, AnnotationKind};
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

fn classify(
    schema: &ParsedSchema,
    content: &str,
    generated: bool,
    ann_pub: bool,
    ann_sub: bool,
) -> &'static str {
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
    let uses_pub = ann_pub || publisher.iter().any(|i| contains_ident(content, i));
    let uses_sub = ann_sub || subscriber.iter().any(|i| contains_ident(content, i));
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
            let code = file.code_only();
            let no_comments = file.without_comments();
            let mut lines = ident_lines(&code, &schema.title);
            if let Some(et) = &schema.event_type {
                for (i, line) in no_comments.lines().enumerate() {
                    if line.contains(et.as_str()) && !lines.contains(&(i + 1)) {
                        lines.push(i + 1);
                    }
                }
            }
            let matching_anns: Vec<_> = event_annotations(&file.content)
                .into_iter()
                .filter(|a| {
                    schema.event_type.as_deref() == Some(a.target.as_str())
                        || schema.title == a.target
                })
                .collect();
            for ann in &matching_anns {
                if !lines.contains(&ann.line) {
                    lines.push(ann.line);
                }
            }
            lines.sort_unstable();
            if lines.is_empty() {
                continue;
            }
            let ann_pub = matching_anns.iter().any(|a| a.kind == AnnotationKind::Publishes);
            let ann_sub = matching_anns.iter().any(|a| a.kind == AnnotationKind::Handles);
            affected.push(AffectedFile {
                path: file.path.display().to_string(),
                role: classify(schema, &code, file.generated, ann_pub, ann_sub).to_string(),
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

    #[test]
    fn impact_discovers_and_classifies_brownfield_annotations() {
        let dir = std::env::temp_dir().join(format!("clrinf-impact-ann-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        let schemas = dir.join("schemas");
        let src = dir.join("src");
        std::fs::create_dir_all(&schemas).unwrap();
        std::fs::create_dir_all(&src).unwrap();

        std::fs::write(
            schemas.join("invoice-issued.v1.schema.json"),
            r#"{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "$id": "https://example.test/InvoiceIssued",
  "title": "InvoiceIssued",
  "type": "object",
  "x-event-type": "billing.invoice.issued.v1",
  "x-published-by": ["billing"],
  "x-subscribed-by": ["ledger"],
  "properties": { "invoiceId": { "type": "string" } }
}"#,
        )
        .unwrap();

        std::fs::write(
            src.join("billing_pub.cs"),
            "// @clrinf:publishes billing.invoice.issued.v1\npublic class BillingWorker { }\n",
        )
        .unwrap();
        std::fs::write(
            src.join("ledger_sub.ex"),
            "# @clrinf:handles billing.invoice.issued.v1\ndefmodule LedgerWorker do\nend\n",
        )
        .unwrap();

        let report = analyze(&schemas, "billing.invoice.issued.v1", &[src]).unwrap();
        assert_eq!(report.affected_files.len(), 2);
        let pub_entry = report
            .affected_files
            .iter()
            .find(|f| f.path.ends_with("billing_pub.cs"))
            .unwrap();
        assert_eq!(pub_entry.role, "publish-call-site");
        assert_eq!(pub_entry.lines, vec![1]);

        let sub_entry = report
            .affected_files
            .iter()
            .find(|f| f.path.ends_with("ledger_sub.ex"))
            .unwrap();
        assert_eq!(sub_entry.role, "handler");
        assert_eq!(sub_entry.lines, vec![1]);

        let _ = std::fs::remove_dir_all(dir);
    }
}
