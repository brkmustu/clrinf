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
use std::collections::HashSet;
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

    /// Emit a SARIF 2.1.0 report suitable for GitHub Code Scanning (`upload-sarif`).
    pub fn to_sarif(&self) -> serde_json::Value {
        let mut seen_rules = HashSet::new();
        let mut rules = Vec::new();
        let mut results = Vec::new();

        for issue in &self.issues {
            let level = match issue.severity {
                IssueSeverity::Error => "error",
                IssueSeverity::Warning => "warning",
            };
            if seen_rules.insert(issue.code.as_str()) {
                rules.push(serde_json::json!({
                    "id": issue.code,
                    "name": issue.code,
                    "shortDescription": { "text": sarif_rule_description(&issue.code) },
                    "defaultConfiguration": { "level": level }
                }));
            }

            let mut result = serde_json::json!({
                "ruleId": issue.code,
                "level": level,
                "message": { "text": issue.message }
            });
            if let Some(file) = &issue.file {
                let uri = file.replace('\\', "/");
                let start_line = issue.line.unwrap_or(1).max(1);
                result["locations"] = serde_json::json!([{
                    "physicalLocation": {
                        "artifactLocation": { "uri": uri },
                        "region": { "startLine": start_line }
                    }
                }]);
            }
            results.push(result);
        }

        serde_json::json!({
            "$schema": "https://json.schemastore.org/sarif-2.1.0.json",
            "version": "2.1.0",
            "runs": [{
                "tool": {
                    "driver": {
                        "name": "clrinf",
                        "version": env!("CARGO_PKG_VERSION"),
                        "informationUri": "https://github.com/brkmustu/clrinf",
                        "rules": rules
                    }
                },
                "results": results
            }]
        })
    }
}

fn sarif_rule_description(code: &str) -> &str {
    match code {
        "RULE_FORBIDDEN_DEPENDENCY" => "Architectural dependency rule violation",
        "RULE_MODULE_BOUNDARY_VIOLATION" => "Cross-module boundary violation",
        "RULE_GENERATED_FILE_MODIFIED" => "Manual edit to generated file",
        "DRIFT_UNDECLARED_PUBLISH" => "Service publishes an event not declared in x-published-by",
        "DRIFT_UNDECLARED_SUBSCRIBE" => "Service handles an event not declared in x-subscribed-by",
        "DRIFT_MISSING_PUBLISHER_USAGE" => "Declared publisher has no matching code usage",
        "DRIFT_MISSING_SUBSCRIBER_IMPL" => "Declared subscriber has no matching handler usage",
        "DRIFT_RAW_OUTBOX_WRITE" => "Direct outbox write bypassing generated publisher",
        "DRIFT_UNKNOWN_ANNOTATED_EVENT" => "Source annotation references an unknown event schema",
        "TOPOLOGY_DEAD_EVENT" => "Event is missing a publisher or subscriber",
        "TOPOLOGY_ORPHAN_SUBSCRIBER" => "Subscriber listens to an event with no publisher",
        "TOPOLOGY_UPCASTER_MISSING" => "Multiple schema versions exist without a registered upcaster",
        "VERIFY_SCHEMA_ERROR" => "Schema loading or topology validation error",
        other => other,
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
    let relevant = |dir: &Path| only.is_none_or(|files| files.iter().any(|f| inside(f, dir)));
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
        if !r.file.boundaries.is_empty() {
            checks.push("boundaries".to_string());
            for i in rules::check_boundaries(r, only)? {
                issues.push(VerifyIssue {
                    source: "boundaries".into(),
                    severity: i.severity,
                    code: i.code,
                    message: i.message,
                    file: Some(i.file),
                    line: Some(i.line),
                });
            }
        }
    }

    // Protect generated files from hand edits on incremental checks (`--changed` / agent edit hook).
    let protect_generated = rules.as_ref().is_none_or(|r| r.file.generated.protect);
    if let (Some(files), true) = (only, protect_generated) {
        checks.push("generated-files".to_string());
        for f in files {
            if !f.is_file() {
                continue;
            }
            if let Ok(content) = std::fs::read_to_string(f) {
                if content.contains("Code generated by clrinf-codegen") {
                    issues.push(VerifyIssue {
                        source: "generated-files".into(),
                        severity: IssueSeverity::Error,
                        code: "RULE_GENERATED_FILE_MODIFIED".into(),
                        message: "Do not edit generated files by hand; update the schema or manifest and re-run clrinf generate.".into(),
                        file: Some(f.strip_prefix(root).unwrap_or(f).display().to_string()),
                        line: Some(1),
                    });
                }
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generated_file_protection_flags_incremental_edits_only_when_enabled() {
        let dir = std::env::temp_dir().join(format!("clrinf-verify-gen-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(dir.join("src")).unwrap();
        let gen_file = dir.join("src/order_placed.ts");
        std::fs::write(
            &gen_file,
            "// Code generated by clrinf-codegen. DO NOT EDIT.\nexport interface OrderPlaced {}\n",
        )
        .unwrap();

        // Full repo verify allows committed generated files on disk
        let full = run(&dir, None).unwrap();
        assert!(full.is_valid);

        // Incremental verify / hook flags hand edit of generated file
        let inc = run(&dir, Some(std::slice::from_ref(&gen_file))).unwrap();
        assert!(!inc.is_valid);
        assert!(inc.issues.iter().any(|i| i.code == "RULE_GENERATED_FILE_MODIFIED"));

        // Opt-out via clrinf.rules.toml [generated] protect = false
        std::fs::write(dir.join("clrinf.rules.toml"), "[generated]\nprotect = false\n").unwrap();
        let disabled = run(&dir, Some(std::slice::from_ref(&gen_file))).unwrap();
        assert!(disabled.is_valid);

        let _ = std::fs::remove_dir_all(dir);
    }

    #[test]
    fn sarif_output_normalizes_paths_deduplicates_rules_and_handles_empty() {
        let empty = VerifyReport {
            is_valid: true,
            checks: vec!["forbid".into()],
            issues: vec![],
        };
        let empty_sarif = empty.to_sarif();
        assert_eq!(empty_sarif["version"], "2.1.0");
        assert_eq!(empty_sarif["$schema"], "https://json.schemastore.org/sarif-2.1.0.json");
        assert_eq!(empty_sarif["runs"][0]["results"].as_array().unwrap().len(), 0);

        let rep = VerifyReport {
            is_valid: false,
            checks: vec!["forbid".into(), "topology".into()],
            issues: vec![
                VerifyIssue {
                    source: "forbid".into(),
                    severity: IssueSeverity::Error,
                    code: "RULE_FORBIDDEN_DEPENDENCY".into(),
                    message: "No Infrastructure in Domain".into(),
                    file: Some(r"src\domain\order.rs".into()),
                    line: Some(12),
                },
                VerifyIssue {
                    source: "forbid".into(),
                    severity: IssueSeverity::Error,
                    code: "RULE_FORBIDDEN_DEPENDENCY".into(),
                    message: "No Infrastructure in Domain 2".into(),
                    file: Some(r"src\domain\user.rs".into()),
                    line: None,
                },
                VerifyIssue {
                    source: "topology".into(),
                    severity: IssueSeverity::Warning,
                    code: "TOPOLOGY_DEAD_EVENT".into(),
                    message: "Dead event".into(),
                    file: None,
                    line: None,
                },
            ],
        };
        let sarif = rep.to_sarif();
        let rules = sarif["runs"][0]["tool"]["driver"]["rules"].as_array().unwrap();
        assert_eq!(rules.len(), 2);
        let results = sarif["runs"][0]["results"].as_array().unwrap();
        assert_eq!(results.len(), 3);
        assert_eq!(
            results[0]["locations"][0]["physicalLocation"]["artifactLocation"]["uri"],
            "src/domain/order.rs"
        );
        assert_eq!(
            results[1]["locations"][0]["physicalLocation"]["region"]["startLine"],
            1
        );
        assert!(results[2].get("locations").is_none());
    }
}
