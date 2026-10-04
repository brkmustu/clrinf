//! Unified project verification: topology, drift, dependency rules.
//!
//! Used by `clrinf verify` (CI / pre-commit), by the agent edit hook (single
//! changed file) and by the MCP `clrinf_verify` tool. With `only` set, just
//! the checks relevant to those files run, which keeps it fast enough to run
//! after every agent edit.

use crate::drift::{self, ServiceSource};
use crate::rules::{self, Rules, DEFAULT_SCHEMA_DIR};
use crate::scan::load_source;
use crate::topology::{IssueSeverity, TopologyValidator};
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerifyIssue {
    pub source: String,
    pub severity: IssueSeverity,
    pub code: String,
    pub message: String,
    pub file: Option<String>,
    pub line: Option<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerifyReport {
    pub is_valid: bool,
    pub checks: Vec<String>,
    pub issues: Vec<VerifyIssue>,
}

impl VerifyReport {
    pub fn errors_count(&self) -> usize {
        self.issues.iter().filter(|i| i.severity == IssueSeverity::Error).count()
    }

    pub fn format_text(&self, errors_only: bool) -> String {
        let mut out = String::new();
        for i in &self.issues {
            if errors_only && i.severity != IssueSeverity::Error {
                continue;
            }
            let icon = match i.severity {
                IssueSeverity::Warning => "WARN",
                IssueSeverity::Error => "FAIL",
            };
            let loc = match (&i.file, i.line) {
                (Some(f), Some(l)) => format!(" ({f}:{l})"),
                (Some(f), None) => format!(" ({f})"),
                _ => String::new(),
            };
            out.push_str(&format!("[{icon}] {} {}{loc}\n", i.code, i.message));
        }
        out
    }
}

fn inside(file: &Path, dir: &Path) -> bool {
    match (file.canonicalize(), dir.canonicalize()) {
        (Ok(f), Ok(d)) => f.starts_with(d),
        _ => file.starts_with(dir),
    }
}

pub fn run(root: &Path, only: Option<&[PathBuf]>) -> Result<VerifyReport> {
    let rules = Rules::load(root)?;
    let schema_dir = rules.as_ref().map(Rules::schema_dir).unwrap_or_else(|| root.join(DEFAULT_SCHEMA_DIR));
    let relevant = |dir: &Path| only.map_or(true, |files| files.iter().any(|f| inside(f, dir)));
    let mut issues = Vec::new();
    let mut checks = Vec::new();

    if schema_dir.is_dir() && relevant(&schema_dir) {
        checks.push("topology".to_string());
        match TopologyValidator::validate(&schema_dir, Some(&schema_dir)) {
            Ok(report) => {
                for i in report.issues {
                    issues.push(VerifyIssue {
                        source: "topology".into(),
                        severity: i.severity,
                        code: i.code,
                        message: i.message,
                        file: None,
                        line: None,
                    });
                }
            }
            Err(e) => issues.push(VerifyIssue {
                source: "topology".into(),
                severity: IssueSeverity::Error,
                code: "VERIFY_SCHEMA_ERROR".into(),
                message: format!("{e:#}"),
                file: None,
                line: None,
            }),
        }
    }

    let mut covered_dirs = Vec::new();
    if let Some(r) = &rules {
        if schema_dir.is_dir() {
            for (name, dir) in r.service_dirs() {
                covered_dirs.push(dir.clone());
                if !relevant(&dir) {
                    continue;
                }
                checks.push(format!("drift:{name}"));
                let report = drift::detect(&schema_dir, &[ServiceSource { name: name.clone(), dir: dir.clone() }])?;
                for i in report.issues {
                    issues.push(VerifyIssue {
                        source: format!("drift:{name}"),
                        severity: i.severity,
                        code: i.code,
                        message: i.message,
                        file: i.file.map(|f| format!("{}/{}", r.file.events.services[&name], f)),
                        line: i.line,
                    });
                }
            }
        }
        if !r.file.forbid.is_empty() {
            checks.push("forbid".to_string());
            for i in rules::check_forbid(r, only)? {
                issues.push(VerifyIssue {
                    source: "forbid".into(),
                    severity: i.severity,
                    code: i.code,
                    message: i.message,
                    file: Some(i.file),
                    line: Some(i.line),
                });
            }
        }
    }

    // Raw outbox writes in changed files that no configured service covers.
    if let Some(files) = only {
        for f in files {
            if covered_dirs.iter().any(|d| inside(f, d)) {
                continue;
            }
            if let Some(src) = load_source(f) {
                if src.generated {
                    continue;
                }
                for line in drift::raw_outbox_writes(&src) {
                    issues.push(VerifyIssue {
                        source: "outbox".into(),
                        severity: IssueSeverity::Error,
                        code: "DRIFT_RAW_OUTBOX_WRITE".into(),
                        message: "Direct outbox write; use the generated publisher so the CloudEvent envelope stays canonical.".into(),
                        file: Some(f.strip_prefix(root).unwrap_or(f).display().to_string()),
                        line: Some(line),
                    });
                }
            }
        }
    }

    let is_valid = !issues.iter().any(|i| i.severity == IssueSeverity::Error);
    Ok(VerifyReport { is_valid, checks, issues })
}
