//! "Fill-in-the-blanks" change plan for events.
//!
//! Given an intent (this service subscribes to / publishes that event), clrinf
//! performs (or previews) every mechanical step and tells the agent exactly
//! what is left: the handful of symbols to implement, the files it may read,
//! the files it must not edit, and the rules it must respect. The agent then
//! only has to write business logic.

use crate::event_scaffold::{self, Role};
use crate::impact;
use crate::rules::{self, Rules};
use crate::scan::{collect_sources, contains_ident};
use crate::schema::parse_schema_file;
use crate::topology::TopologyValidator;
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FillIn {
    pub language: String,
    pub kind: String,
    pub symbol: String,
    pub file: Option<String>,
    pub note: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SchemaEdit {
    pub file: String,
    pub would_change: bool,
    pub applied: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChangePlan {
    pub event_type: String,
    pub title: String,
    pub role: String,
    pub service: String,
    pub schema_edit: SchemaEdit,
    pub fill_in: Vec<FillIn>,
    pub read_these: Vec<String>,
    pub do_not_edit: Vec<String>,
    pub constraints: Vec<String>,
    pub open_issues: Vec<String>,
    pub next_steps: Vec<String>,
}

fn role_name(role: Role) -> &'static str {
    match role {
        Role::Publisher => "publisher",
        Role::Subscriber => "subscriber",
    }
}

pub fn plan_event(
    schema_dir: &Path,
    event_type: &str,
    service: &str,
    role: Role,
    src_roots: &[PathBuf],
    apply: bool,
    rules: Option<&Rules>,
) -> Result<ChangePlan> {
    let report = impact::analyze(schema_dir, event_type, src_roots)?;
    let schema_path = PathBuf::from(&report.schema_file);
    let schema = parse_schema_file(&schema_path)?;
    let (title, stem) = (schema.title.clone(), schema.file_stem.clone());

    let text = std::fs::read_to_string(&schema_path).context("Reading event schema")?;
    let would_change = event_scaffold::register_in_text(&text, role, service)?.is_some();
    let applied = if apply && would_change {
        event_scaffold::register(schema_dir, &report.event_type, role, service)?;
        true
    } else {
        false
    };

    // Which symbols must the agent implement, per language?
    let candidates: Vec<(&str, &str, String, &str)> = match role {
        Role::Subscriber => vec![
            ("typescript", "handler", format!("{title}Handler"), "implement handle(payload, context); envelope parsing and idempotency are generated"),
            ("rust", "handler", format!("{title}Handler"), "implement the async handle method; envelope parsing and idempotency are generated"),
            ("csharp", "handler", format!("I{title}Handler"), "implement HandleAsync; envelope parsing and idempotency are generated"),
        ],
        Role::Publisher => vec![
            ("typescript", "call-site", format!("publish{title}"), "call from your use case inside the transaction; the outbox write is generated"),
            ("rust", "call-site", format!("publish_{stem}"), "call from your use case inside the transaction; the outbox write is generated"),
            ("csharp", "call-site", format!("{title}Publisher"), "call PublishAsync from your use case inside the transaction"),
        ],
    };

    let mut all_sources = Vec::new();
    for root in src_roots {
        all_sources.extend(collect_sources(root)?);
    }
    let fill_in = candidates
        .into_iter()
        .map(|(language, kind, symbol, note)| {
            let file = all_sources
                .iter()
                .find(|f| f.generated && contains_ident(&f.content, &symbol))
                .map(|f| f.path.display().to_string());
            FillIn { language: language.into(), kind: kind.into(), symbol, file, note: note.into() }
        })
        .collect();

    let do_not_edit: Vec<String> = report
        .affected_files
        .iter()
        .filter(|f| f.generated)
        .map(|f| f.path.clone())
        .chain(std::iter::once(report.schema_file.clone()).filter(|_| !would_change || applied))
        .collect();
    let read_these: Vec<String> = report
        .affected_files
        .iter()
        .filter(|f| !f.generated)
        .map(|f| f.path.clone())
        .collect();

    let topo = TopologyValidator::validate(schema_dir, Some(schema_dir))?;
    let open_issues = topo
        .issues
        .iter()
        .filter(|i| i.event_type == report.event_type || report.event_type.contains(&i.event_type))
        .map(|i| format!("[{}] {}", i.code, i.message))
        .collect();

    let mut next_steps = Vec::new();
    if would_change && !applied {
        next_steps.push(format!(
            "Run `clrinf event {} {} --service {}` (or call clrinf_event_register) to register the service and regenerate shells.",
            if role == Role::Subscriber { "subscribe" } else { "publish" },
            report.event_type,
            service
        ));
    }
    next_steps.push("Implement only the symbols listed in fill_in; do not touch generated files.".to_string());
    next_steps.push("Run `clrinf verify --changed` (or clrinf_verify) before finishing.".to_string());

    Ok(ChangePlan {
        event_type: report.event_type,
        title,
        role: role_name(role).into(),
        service: service.into(),
        schema_edit: SchemaEdit { file: report.schema_file, would_change, applied },
        fill_in,
        read_these,
        do_not_edit,
        constraints: rules::constraint_lines(rules),
        open_issues,
        next_steps,
    })
}
