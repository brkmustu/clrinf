//! Code <-> contract drift detection.
//!
//! The topology graph is built from schema declarations (`x-published-by`,
//! `x-subscribed-by`). This module checks those declarations against what the
//! hand-written source code of each service actually does, so the declared
//! topology cannot silently diverge from reality.

use crate::schema::{parse_schema_file, ParsedSchema};
use crate::scan::{
    collect_sources, contains_ident, event_annotations, ident_lines, AnnotationKind, SourceFile,
};
use crate::topology::IssueSeverity;
use crate::validator::discover_files;
use anyhow::{bail, Context, Result};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub struct ServiceSource {
    pub name: String,
    pub dir: PathBuf,
}

impl ServiceSource {
    /// Parse `name=path` (the first `=` splits name from path).
    pub fn parse(spec: &str) -> Result<Self> {
        match spec.split_once('=') {
            Some((name, dir)) if !name.trim().is_empty() && !dir.trim().is_empty() => Ok(Self {
                name: name.trim().to_string(),
                dir: PathBuf::from(dir.trim()),
            }),
            _ => bail!("Invalid --service '{spec}'; expected NAME=PATH"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DriftIssue {
    pub severity: IssueSeverity,
    pub code: String,
    pub message: String,
    pub event_type: Option<String>,
    pub service: String,
    pub file: Option<String>,
    pub line: Option<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DriftReport {
    pub is_valid: bool,
    pub issues: Vec<DriftIssue>,
}

impl DriftReport {
    pub fn errors_count(&self) -> usize {
        self.issues.iter().filter(|i| i.severity == IssueSeverity::Error).count()
    }

    pub fn warnings_count(&self) -> usize {
        self.issues.iter().filter(|i| i.severity == IssueSeverity::Warning).count()
    }

    pub fn format_diagnostics(&self) -> String {
        let mut out = String::new();
        for issue in &self.issues {
            let icon = match issue.severity {
                IssueSeverity::Warning => "⚠️  [WARN]",
                IssueSeverity::Error => "❌ [FAIL]",
            };
            let location = match (&issue.file, issue.line) {
                (Some(f), Some(l)) => format!(" ({f}:{l})"),
                (Some(f), None) => format!(" ({f})"),
                _ => String::new(),
            };
            out.push_str(&format!("{icon} [{}] {}{location}\n", issue.code, issue.message));
        }
        if self.is_valid {
            out.push_str("\n✅ Drift check passed: code matches declared topology.\n");
        } else {
            out.push_str("\n❌ Drift check failed: code diverges from declared topology.\n");
        }
        out
    }
}

fn publish_idents(s: &ParsedSchema) -> Vec<String> {
    vec![
        format!("publish{}", s.title),
        format!("publish_{}", s.file_stem),
        format!("{}Publisher", s.title),
    ]
}

fn subscribe_idents(s: &ParsedSchema) -> Vec<String> {
    vec![
        format!("{}Handler", s.title),
        format!("I{}Handler", s.title),
        format!("handle{}Event", s.title),
        format!("handle_{}_event", s.file_stem),
        format!("{}Subscriber", s.title),
    ]
}

/// First non-generated file/line that references any of `idents` in actual code.
fn first_usage<'a>(sources: &'a [SourceFile], idents: &[String]) -> Option<(&'a SourceFile, usize)> {
    for file in sources.iter().filter(|f| !f.generated) {
        let code = file.code_only();
        for ident in idents {
            if contains_ident(&code, ident) {
                let line = ident_lines(&code, ident).first().copied().unwrap_or(1);
                return Some((file, line));
            }
        }
    }
    None
}

fn annotation_matches(target: &str, schema: &ParsedSchema) -> bool {
    schema.event_type.as_deref() == Some(target) || schema.title == target
}

fn first_annotated_usage<'a>(
    sources: &'a [SourceFile],
    kind: AnnotationKind,
    schema: &ParsedSchema,
) -> Option<(&'a SourceFile, usize)> {
    for file in sources.iter().filter(|f| !f.generated) {
        for ann in event_annotations(&file.content) {
            if ann.kind == kind && annotation_matches(&ann.target, schema) {
                return Some((file, ann.line));
            }
        }
    }
    None
}

pub fn raw_outbox_writes(file: &SourceFile) -> Vec<usize> {
    let code = file.code_only();
    code.lines()
        .enumerate()
        .filter(|(_, line)| {
            let lower = line.to_ascii_lowercase();
            lower.contains("outbox")
                && (lower.contains(".enqueue(")
                    || lower.contains(".enqueueasync(")
                    || lower.contains(".append(")
                    || lower.contains(".appendasync("))
        })
        .map(|(i, _)| i + 1)
        .collect()
}

pub fn detect(schema_dir: &Path, services: &[ServiceSource]) -> Result<DriftReport> {
    let mut schemas = Vec::new();
    for file in discover_files(schema_dir, &["json"])
        .with_context(|| format!("Reading schemas from {}", schema_dir.display()))?
    {
        schemas.push(parse_schema_file(&file)?);
    }
    let events: Vec<&ParsedSchema> = schemas.iter().filter(|s| s.event_type.is_some()).collect();

    let mut issues = Vec::new();
    for service in services {
        let sources = collect_sources(&service.dir)?;
        let rel = |f: &SourceFile| {
            f.path
                .strip_prefix(&service.dir)
                .unwrap_or(&f.path)
                .display()
                .to_string()
        };

        for schema in &events {
            let event_type = schema.event_type.clone().unwrap();
            let declared_pub = schema.published_by.iter().any(|s| s == &service.name);
            let declared_sub = schema.subscribed_by.iter().any(|s| s == &service.name);
            let pub_use = first_usage(&sources, &publish_idents(schema))
                .or_else(|| first_annotated_usage(&sources, AnnotationKind::Publishes, schema));
            let sub_use = first_usage(&sources, &subscribe_idents(schema))
                .or_else(|| first_annotated_usage(&sources, AnnotationKind::Handles, schema));

            if let (Some((file, line)), false) = (pub_use, declared_pub) {
                issues.push(DriftIssue {
                    severity: IssueSeverity::Error,
                    code: "DRIFT_UNDECLARED_PUBLISH".into(),
                    message: format!(
                        "Service '{}' publishes '{}' but is not listed in x-published-by.",
                        service.name, event_type
                    ),
                    event_type: Some(event_type.clone()),
                    service: service.name.clone(),
                    file: Some(rel(file)),
                    line: Some(line),
                });
            }
            if let (Some((file, line)), false) = (sub_use, declared_sub) {
                issues.push(DriftIssue {
                    severity: IssueSeverity::Error,
                    code: "DRIFT_UNDECLARED_SUBSCRIBE".into(),
                    message: format!(
                        "Service '{}' handles '{}' but is not listed in x-subscribed-by.",
                        service.name, event_type
                    ),
                    event_type: Some(event_type.clone()),
                    service: service.name.clone(),
                    file: Some(rel(file)),
                    line: Some(line),
                });
            }
            if declared_pub && pub_use.is_none() {
                issues.push(DriftIssue {
                    severity: IssueSeverity::Warning,
                    code: "DRIFT_MISSING_PUBLISHER_USAGE".into(),
                    message: format!(
                        "Service '{}' is declared as publisher of '{}' but no code calls the generated publisher.",
                        service.name, event_type
                    ),
                    event_type: Some(event_type.clone()),
                    service: service.name.clone(),
                    file: None,
                    line: None,
                });
            }
            if declared_sub && sub_use.is_none() {
                issues.push(DriftIssue {
                    severity: IssueSeverity::Warning,
                    code: "DRIFT_MISSING_SUBSCRIBER_IMPL".into(),
                    message: format!(
                        "Service '{}' is declared as subscriber of '{}' but no handler implementation was found.",
                        service.name, event_type
                    ),
                    event_type: Some(event_type.clone()),
                    service: service.name.clone(),
                    file: None,
                    line: None,
                });
            }
        }

        for file in sources.iter().filter(|f| !f.generated) {
            for line in raw_outbox_writes(file) {
                issues.push(DriftIssue {
                    severity: IssueSeverity::Error,
                    code: "DRIFT_RAW_OUTBOX_WRITE".into(),
                    message: format!(
                        "Service '{}' writes to the outbox directly; use the generated publisher so the CloudEvent envelope stays canonical.",
                        service.name
                    ),
                    event_type: None,
                    service: service.name.clone(),
                    file: Some(rel(file)),
                    line: Some(line),
                });
            }
            for ann in event_annotations(&file.content) {
                if !events.iter().any(|s| annotation_matches(&ann.target, s)) {
                    issues.push(DriftIssue {
                        severity: IssueSeverity::Error,
                        code: "DRIFT_UNKNOWN_ANNOTATED_EVENT".into(),
                        message: format!(
                            "Service '{}' references unknown annotated event '{}' (no schema with x-event-type or title matches).",
                            service.name, ann.target
                        ),
                        event_type: Some(ann.target),
                        service: service.name.clone(),
                        file: Some(rel(file)),
                        line: Some(ann.line),
                    });
                }
            }
        }
    }

    let is_valid = !issues.iter().any(|i| i.severity == IssueSeverity::Error);
    Ok(DriftReport { is_valid, issues })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn service_spec_parses() {
        let s = ServiceSource::parse("billing=./svc/billing").unwrap();
        assert_eq!(s.name, "billing");
        assert!(ServiceSource::parse("nope").is_err());
        assert!(ServiceSource::parse("=x").is_err());
    }

    #[test]
    fn ignores_comments_and_strings_in_outbox_and_usage_scans() {
        use crate::ast::SourceLang;

        let file = SourceFile {
            path: PathBuf::from("svc/handler.rs"),
            lang: SourceLang::Rust,
            generated: false,
            content: "/* outbox.enqueue(x); publishOrderPlaced(); */\nlet msg = \"outbox.enqueue(y)\";\noutbox.enqueue(ev);\n".into(),
        };
        assert_eq!(raw_outbox_writes(&file), vec![3]);
        let files = [file];
        let usage = first_usage(&files, &["publishOrderPlaced".into()]);
        assert!(usage.is_none());
    }

    #[test]
    fn brownfield_annotations_participate_in_drift_and_reject_unknown_events() {
        let dir = std::env::temp_dir().join(format!("clrinf-drift-ann-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        let schemas = dir.join("schemas");
        let orders = dir.join("svc/orders");
        let billing = dir.join("svc/billing");
        let shipping = dir.join("svc/shipping");
        std::fs::create_dir_all(&schemas).unwrap();
        std::fs::create_dir_all(&orders).unwrap();
        std::fs::create_dir_all(&billing).unwrap();
        std::fs::create_dir_all(&shipping).unwrap();

        std::fs::write(
            schemas.join("order-placed.v1.schema.json"),
            r#"{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "$id": "https://example.test/OrderPlaced",
  "title": "OrderPlaced",
  "type": "object",
  "x-event-type": "orders.placed.v1",
  "x-published-by": ["orders"],
  "x-subscribed-by": ["billing"],
  "properties": { "orderId": { "type": "string" } }
}"#,
        )
        .unwrap();

        std::fs::write(
            orders.join("pub.rs"),
            "// @clrinf:publishes orders.placed.v1\npub fn create_order() {}\n",
        )
        .unwrap();
        std::fs::write(
            billing.join("sub.ex"),
            "# @clrinf:handles orders.placed.v1\ndefmodule Billing.Sub do\nend\n",
        )
        .unwrap();

        let ok_report = detect(
            &schemas,
            &[
                ServiceSource { name: "orders".into(), dir: orders.clone() },
                ServiceSource { name: "billing".into(), dir: billing.clone() },
            ],
        )
        .unwrap();
        assert!(ok_report.is_valid, "{:?}", ok_report.issues);
        assert!(ok_report.issues.is_empty(), "{:?}", ok_report.issues);

        std::fs::write(
            shipping.join("extra.ts"),
            "// @clrinf:handles orders.placed.v1\n// @clrinf:publishes orders.unknown.v1\nexport function run() {}\n",
        )
        .unwrap();
        let bad_report = detect(
            &schemas,
            &[ServiceSource { name: "shipping".into(), dir: shipping }],
        )
        .unwrap();
        assert!(!bad_report.is_valid);
        assert!(bad_report.issues.iter().any(|i| i.code == "DRIFT_UNDECLARED_SUBSCRIBE"));
        assert!(bad_report.issues.iter().any(|i| i.code == "DRIFT_UNKNOWN_ANNOTATED_EVENT"));

        let _ = std::fs::remove_dir_all(dir);
    }
}
