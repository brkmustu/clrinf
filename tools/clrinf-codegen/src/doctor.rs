//! Environment and guardrails diagnostic checks for `clrinf doctor`.

use crate::agents::{self, Status};
use crate::hook;
use crate::rules::{Rules, DEFAULT_SCHEMA_DIR, RULES_FILE};
use crate::topology::IssueSeverity;
use crate::validator;
use crate::verify;
use crate::worker::WorkerManager;
use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DoctorCheck {
    pub name: String,
    pub status: String,
    pub summary: String,
    pub hint: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DoctorReport {
    pub ok: bool,
    pub checks: Vec<DoctorCheck>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub fixed: Vec<String>,
}

impl DoctorReport {
    pub fn format_text(&self) -> String {
        let mut out = String::new();
        if !self.fixed.is_empty() {
            out.push_str("Fixed:\n");
            for item in &self.fixed {
                out.push_str(&format!("  * {item}\n"));
            }
            out.push('\n');
        }
        for check in &self.checks {
            let tag = match check.status.as_str() {
                "pass" => "[PASS]",
                "warn" => "[WARN]",
                _ => "[FAIL]",
            };
            out.push_str(&format!("{tag} {}: {}\n", check.name, check.summary));
            if let Some(hint) = &check.hint {
                out.push_str(&format!("       hint: {hint}\n"));
            }
        }
        let fails = self.checks.iter().filter(|c| c.status == "fail").count();
        let warns = self.checks.iter().filter(|c| c.status == "warn").count();
        let icon = if self.ok { "✅" } else { "❌" };
        out.push_str(&format!(
            "\n{icon} clrinf doctor: {} check(s), {fails} failure(s), {warns} warning(s).\n",
            self.checks.len()
        ));
        out
    }
}

pub fn run(root: &Path) -> DoctorReport {
    run_with_fix(root, false)
}

pub fn run_with_fix(root: &Path, fix: bool) -> DoctorReport {
    let mut fixed = Vec::new();
    if fix {
        let rules_pre = Rules::load(root).ok().flatten();
        if let Ok(dry) = agents::sync(root, rules_pre.as_ref(), true) {
            if dry.iter().any(|(_, s)| *s != Status::Unchanged) {
                if let Ok(applied) = agents::sync(root, rules_pre.as_ref(), false) {
                    for (path, status) in applied {
                        if status != Status::Unchanged {
                            fixed.push(format!(
                                "Synced agent instruction file: {}",
                                path.strip_prefix(root).unwrap_or(&path).display()
                            ));
                        }
                    }
                }
            }
        }
        let has_any_hook = [
            ".claude/settings.json",
            ".cursor/hooks.json",
            ".agents/hooks.json",
            ".git/hooks/pre-commit",
        ]
        .iter()
        .any(|rel| {
            let p = root.join(rel);
            p.is_file() && std::fs::read_to_string(&p).is_ok_and(|c| c.contains("clrinf"))
        });
        if !has_any_hook && root.join(RULES_FILE).is_file() {
            if hook::install_claude(root, "clrinf").unwrap_or(false) {
                fixed.push("Installed hook: .claude/settings.json".into());
            }
            if hook::install_cursor(root, "clrinf").unwrap_or(false) {
                fixed.push("Installed hook: .cursor/hooks.json".into());
            }
            if hook::install_antigravity(root, "clrinf").unwrap_or(false) {
                fixed.push("Installed hook: .agents/hooks.json".into());
            }
            if root.join(".git").is_dir() && hook::install_git(root, "clrinf", false).is_ok() {
                fixed.push("Installed hook: .git/hooks/pre-commit".into());
            }
        }
    }

    let mut checks = Vec::new();

    // 1. clrinf.rules.toml
    let rules = match Rules::load(root) {
        Ok(Some(r)) => {
            checks.push(DoctorCheck {
                name: "rules_file".into(),
                status: "pass".into(),
                summary: format!(
                    "clrinf.rules.toml valid ({} forbid, {} boundaries, {} services)",
                    r.file.forbid.len(),
                    r.file.boundaries.len(),
                    r.file.events.services.len()
                ),
                hint: None,
            });
            Some(r)
        }
        Ok(None) => {
            checks.push(DoctorCheck {
                name: "rules_file".into(),
                status: "warn".into(),
                summary: "clrinf.rules.toml not found".into(),
                hint: Some("Run 'clrinf init --guardrails' to bootstrap rules, hooks, and agent instructions.".into()),
            });
            None
        }
        Err(e) => {
            checks.push(DoctorCheck {
                name: "rules_file".into(),
                status: "fail".into(),
                summary: format!("Failed to parse clrinf.rules.toml: {e:#}"),
                hint: Some("Fix syntax or field names in clrinf.rules.toml.".into()),
            });
            None
        }
    };

    // 2. Agent instruction sync status (check-only)
    match agents::sync(root, rules.as_ref(), true) {
        Ok(results) => {
            let stale: Vec<String> = results
                .iter()
                .filter(|(_, s)| *s != Status::Unchanged)
                .map(|(p, _)| p.strip_prefix(root).unwrap_or(p).display().to_string())
                .collect();
            if stale.is_empty() {
                checks.push(DoctorCheck {
                    name: "agents_sync".into(),
                    status: "pass".into(),
                    summary: format!("Agent instruction files are in sync ({} checked)", results.len()),
                    hint: None,
                });
            } else {
                checks.push(DoctorCheck {
                    name: "agents_sync".into(),
                    status: "warn".into(),
                    summary: format!(
                        "{} agent instruction file(s) need sync: {}",
                        stale.len(),
                        stale.join(", ")
                    ),
                    hint: Some("Run 'clrinf agents sync' to refresh managed blocks.".into()),
                });
            }
        }
        Err(e) => checks.push(DoctorCheck {
            name: "agents_sync".into(),
            status: "fail".into(),
            summary: format!("Agent sync check failed: {e:#}"),
            hint: None,
        }),
    }

    // 3. Installed hooks
    let hook_targets = [
        (".claude/settings.json", "claude"),
        (".cursor/hooks.json", "cursor"),
        (".agents/hooks.json", "antigravity"),
        (".git/hooks/pre-commit", "git"),
    ];
    let mut installed = Vec::new();
    for (rel, label) in hook_targets {
        let p = root.join(rel);
        if p.is_file() && std::fs::read_to_string(&p).is_ok_and(|c| c.contains("clrinf")) {
            installed.push(label);
        }
    }
    if installed.is_empty() {
        checks.push(DoctorCheck {
            name: "hooks".into(),
            status: "warn".into(),
            summary: "No agent or git hooks installed".into(),
            hint: Some("Run 'clrinf hook install --agent all' (or claude|cursor|antigravity|git).".into()),
        });
    } else {
        checks.push(DoctorCheck {
            name: "hooks".into(),
            status: "pass".into(),
            summary: format!("Installed hooks: {}", installed.join(", ")),
            hint: None,
        });
    }

    // 4. Schema directory validation
    let schema_dir = rules
        .as_ref()
        .map(Rules::schema_dir)
        .unwrap_or_else(|| root.join(DEFAULT_SCHEMA_DIR));
    if !schema_dir.is_dir() {
        checks.push(DoctorCheck {
            name: "schemas".into(),
            status: "warn".into(),
            summary: format!(
                "Schema directory '{}' not found; event contract checks skipped.",
                schema_dir.display()
            ),
            hint: Some("Create schemas or configure [events] schema_dir in clrinf.rules.toml.".into()),
        });
    } else {
        match validator::check_schemas_quiet(&schema_dir) {
            Ok(n) => checks.push(DoctorCheck {
                name: "schemas".into(),
                status: "pass".into(),
                summary: format!("{n} valid Draft 2020-12 schema(s) in {}", schema_dir.display()),
                hint: None,
            }),
            Err(e) => checks.push(DoctorCheck {
                name: "schemas".into(),
                status: "fail".into(),
                summary: format!("Schema validation failed: {e:#}"),
                hint: Some(format!(
                    "Run 'clrinf check --schema-dir {}' for details.",
                    schema_dir.display()
                )),
            }),
        }
    }

    // 5. Full verify gate
    match verify::run(root, None) {
        Ok(rep) if rep.is_valid => {
            let warns = rep
                .issues
                .iter()
                .filter(|i| i.severity == IssueSeverity::Warning)
                .count();
            checks.push(DoctorCheck {
                name: "verify".into(),
                status: "pass".into(),
                summary: format!("verify passed ({} check(s), {warns} warning(s))", rep.checks.len()),
                hint: None,
            });
        }
        Ok(rep) => checks.push(DoctorCheck {
            name: "verify".into(),
            status: "fail".into(),
            summary: format!(
                "verify failed with {} error(s) across {} check(s)",
                rep.errors_count(),
                rep.checks.len()
            ),
            hint: Some("Run 'clrinf verify' to inspect and fix violations.".into()),
        }),
        Err(e) => checks.push(DoctorCheck {
            name: "verify".into(),
            status: "fail".into(),
            summary: format!("verify failed to execute: {e:#}"),
            hint: None,
        }),
    }

    // 6. Federated workers
    let workers = WorkerManager::new().resolve_all_workers();
    let available: Vec<String> = workers
        .iter()
        .filter(|w| w.is_available)
        .map(|w| w.name.clone())
        .collect();
    let missing: Vec<String> = workers
        .iter()
        .filter(|w| !w.is_available)
        .map(|w| w.name.clone())
        .collect();
    if available.is_empty() {
        checks.push(DoctorCheck {
            name: "workers".into(),
            status: "warn".into(),
            summary: "No federated language workers found on PATH or workspace (guardrails work standalone)".into(),
            hint: Some("Optional: install language workers (clrinfcs, clrinfts, clrinfex, clrinfrs) if using code generation.".into()),
        });
    } else {
        let summary = if missing.is_empty() {
            format!("All federated workers available: {}", available.join(", "))
        } else {
            format!(
                "Available workers: {} (missing: {})",
                available.join(", "),
                missing.join(", ")
            )
        };
        checks.push(DoctorCheck {
            name: "workers".into(),
            status: "pass".into(),
            summary,
            hint: None,
        });
    }

    let ok = !checks.iter().any(|c| c.status == "fail");
    DoctorReport { ok, checks, fixed }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::guardrails_init;

    #[test]
    fn empty_dir_warns_without_failing() {
        let dir = std::env::temp_dir().join(format!("clrinf-doctor-empty-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();

        let report = run(&dir);
        assert!(report.ok, "{:?}", report.checks);
        let status = |name: &str| {
            report
                .checks
                .iter()
                .find(|c| c.name == name)
                .unwrap()
                .status
                .as_str()
        };
        assert_eq!(status("rules_file"), "warn");
        assert_eq!(status("agents_sync"), "warn");
        assert_eq!(status("hooks"), "warn");
        assert_eq!(status("schemas"), "warn");
        assert_eq!(status("verify"), "pass");

        let _ = std::fs::remove_dir_all(dir);
    }

    #[test]
    fn initialized_repo_passes_core_health_checks() {
        let dir = std::env::temp_dir().join(format!("clrinf-doctor-init-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(dir.join("services/orders/src/domain")).unwrap();
        std::fs::write(
            dir.join("services/orders/src/domain/order.rs"),
            "pub struct Order { pub id: String }\n",
        )
        .unwrap();

        guardrails_init::init_guardrails(&dir, "claude", false, false).unwrap();

        let report = run(&dir);
        assert!(report.ok, "{:?}", report.checks);
        for name in ["rules_file", "agents_sync", "hooks", "verify"] {
            let c = report.checks.iter().find(|c| c.name == name).unwrap();
            assert_eq!(c.status, "pass", "expected pass for {name}: {:?}", c);
        }

        let _ = std::fs::remove_dir_all(dir);
    }

    #[test]
    fn invalid_rules_file_causes_doctor_failure() {
        let dir = std::env::temp_dir().join(format!("clrinf-doctor-bad-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join("clrinf.rules.toml"), "not valid = toml [[\n").unwrap();

        let report = run(&dir);
        assert!(!report.ok);
        let rules_check = report.checks.iter().find(|c| c.name == "rules_file").unwrap();
        assert_eq!(rules_check.status, "fail");

        let _ = std::fs::remove_dir_all(dir);
    }

    #[test]
    fn doctor_fix_syncs_agents_and_installs_hooks() {
        let dir = std::env::temp_dir().join(format!("clrinf-doctor-fix-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join("clrinf.rules.toml"), "[generated]\nprotect = true\n").unwrap();

        let before = run(&dir);
        assert_eq!(
            before.checks.iter().find(|c| c.name == "agents_sync").unwrap().status,
            "warn"
        );
        assert_eq!(
            before.checks.iter().find(|c| c.name == "hooks").unwrap().status,
            "warn"
        );

        let after = run_with_fix(&dir, true);
        assert!(after.ok, "{:?}", after.checks);
        assert!(!after.fixed.is_empty());
        assert_eq!(
            after.checks.iter().find(|c| c.name == "agents_sync").unwrap().status,
            "pass"
        );
        assert_eq!(
            after.checks.iter().find(|c| c.name == "hooks").unwrap().status,
            "pass"
        );

        let _ = std::fs::remove_dir_all(dir);
    }
}
