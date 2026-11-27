//! Brownfield onboarding wizard for `clrinf init --guardrails`.
//!
//! Scans an existing repository for schemas, domain folders, and sibling
//! service/module containers, generates `clrinf.rules.toml`, syncs AI agent
//! instruction files, installs edit hooks, optionally scaffolds a GitHub
//! Actions workflow, and runs `clrinf verify` to summarize baseline status.

use crate::agents::{self, Status};
use crate::hook;
use crate::rules::Rules;
use crate::verify;
use anyhow::Result;
use serde::Serialize;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

const SKIPPED_DIRS: &[&str] = &[
    ".git",
    "node_modules",
    "target",
    "bin",
    "obj",
    "dist",
    "_build",
    "deps",
    ".venv",
    "graft",
];

#[derive(Debug, Clone, Serialize)]
pub struct GuardrailsInitReport {
    pub rules_path: PathBuf,
    pub rules_created: bool,
    pub detected_schema_dir: Option<String>,
    pub detected_services: BTreeMap<String, String>,
    pub detected_domains: Vec<String>,
    pub detected_modules: Vec<String>,
    pub synced_agent_files: Vec<String>,
    pub hooks_installed: Vec<String>,
    pub hook_notes: Vec<String>,
    pub ci_workflow_path: Option<PathBuf>,
    pub verify_errors: usize,
}

fn rel_slash(root: &Path, path: &Path) -> String {
    path.strip_prefix(root)
        .unwrap_or(path)
        .to_string_lossy()
        .replace('\\', "/")
}

fn is_skipped_dir(name: &str) -> bool {
    name.starts_with('.') || SKIPPED_DIRS.contains(&name)
}

fn detect_schema_dir(root: &Path) -> Option<String> {
    for candidate in ["contracts/schemas", "schemas", "tools/clrinf-codegen/schemas"] {
        if root.join(candidate).is_dir() {
            return Some(candidate.to_string());
        }
    }
    None
}

fn detect_sibling_modules(root: &Path) -> (BTreeMap<String, String>, Vec<String>) {
    let mut services = BTreeMap::new();
    let mut modules = Vec::new();

    for container in ["services", "src/Modules", "Modules", "crates", "packages"] {
        let dir = root.join(container);
        if !dir.is_dir() {
            continue;
        }
        let mut children: Vec<(String, String)> = Vec::new();
        if let Ok(entries) = std::fs::read_dir(&dir) {
            for entry in entries.flatten() {
                let p = entry.path();
                if !p.is_dir() {
                    continue;
                }
                let Some(name) = p.file_name().and_then(|n| n.to_str()) else {
                    continue;
                };
                if is_skipped_dir(name) {
                    continue;
                }
                children.push((name.to_string(), rel_slash(root, &p)));
            }
        }
        children.sort_by(|a, b| a.0.cmp(&b.0));
        if children.len() >= 2 {
            for (name, rel) in children {
                if container == "services" {
                    services.insert(name, rel.clone());
                }
                modules.push(rel);
            }
            break;
        }
    }

    (services, modules)
}

fn detect_domain_dirs(root: &Path) -> Vec<String> {
    let mut found = Vec::new();
    let walker = WalkDir::new(root)
        .max_depth(4)
        .follow_links(false)
        .into_iter()
        .filter_entry(|entry| {
            if entry.depth() == 0 {
                return true;
            }
            if !entry.file_type().is_dir() {
                return false;
            }
            entry
                .file_name()
                .to_str()
                .is_none_or(|name| !is_skipped_dir(name))
        });

    for entry in walker.flatten() {
        if entry.depth() == 0 || !entry.file_type().is_dir() {
            continue;
        }
        let name = entry.file_name().to_str().unwrap_or("");
        if name.eq_ignore_ascii_case("domain") {
            found.push(rel_slash(root, entry.path()));
        }
    }
    found.sort();
    found.dedup();
    found
}

fn toml_string_array(items: &[String]) -> String {
    let quoted: Vec<String> = items
        .iter()
        .map(|s| format!("\"{}\"", s.replace('"', "\\\"")))
        .collect();
    format!("[{}]", quoted.join(", "))
}

fn render_rules_toml(
    schema_dir: Option<&str>,
    services: &BTreeMap<String, String>,
    domains: &[String],
    modules: &[String],
) -> String {
    let mut out = String::new();
    out.push_str("# clrinf architecture & event contract guardrails (single source of truth)\n\n");

    if schema_dir.is_some() || !services.is_empty() {
        out.push_str("[events]\n");
        if let Some(sd) = schema_dir {
            out.push_str(&format!("schema_dir = \"{sd}\"\n"));
        }
        if !services.is_empty() {
            out.push_str("\n[events.services]\n");
            for (name, dir) in services {
                out.push_str(&format!("{name} = \"{dir}\"\n"));
            }
        }
        out.push('\n');
    }

    out.push_str("[generated]\n");
    out.push_str("protect = true\n\n");

    out.push_str("[mcp]\n");
    out.push_str("profile = \"lean\"\n\n");

    if !domains.is_empty() {
        out.push_str("[[forbid]]\n");
        out.push_str("id      = \"domain-no-infra\"\n");
        out.push_str("scope   = \"imports\"\n");
        out.push_str(&format!("paths   = {}\n", toml_string_array(domains)));
        out.push_str("pattern = \"Infrastructure\"\n");
        out.push_str("reason  = \"Domain layers must remain free of infrastructure dependencies\"\n\n");
    } else {
        out.push_str("# [[forbid]]\n");
        out.push_str("# id      = \"domain-no-infra\"\n");
        out.push_str("# scope   = \"imports\"\n");
        out.push_str("# paths   = [\"src/domain\"]\n");
        out.push_str("# pattern = \"Infrastructure\"\n");
        out.push_str("# reason  = \"Domain layers must remain free of infrastructure dependencies\"\n\n");
    }

    if modules.len() >= 2 {
        out.push_str("[[boundaries]]\n");
        out.push_str("id      = \"module-isolation\"\n");
        out.push_str(&format!("modules = {}\n", toml_string_array(modules)));
        out.push_str("public  = [\"contracts\", \"Contracts\", \"public\", \"Public\"]\n");
        out.push_str("reason  = \"Cross-module imports must go through public contracts\"\n");
    }

    out
}

fn render_ci_workflow(has_schemas: bool) -> String {
    let mut out = String::from(
        "name: clrinf Architecture Guardrails\n\non:\n  pull_request:\n  push:\n    branches: [main, master]\n\njobs:\n  guardrails:\n    runs-on: ubuntu-latest\n    steps:\n      - uses: actions/checkout@v4\n      - uses: brkmustu/clrinf@v0.1.0\n",
    );
    if has_schemas {
        out.push_str("        # with:\n        #   report-base-ref: origin/main\n");
    }
    out
}

pub fn init_guardrails(
    root: &Path,
    agent: &str,
    ci: bool,
    force: bool,
) -> Result<GuardrailsInitReport> {
    if !root.exists() {
        std::fs::create_dir_all(root)?;
    }

    let detected_schema_dir = detect_schema_dir(root);
    let (detected_services, detected_modules) = detect_sibling_modules(root);
    let detected_domains = detect_domain_dirs(root);

    let rules_path = root.join(crate::rules::RULES_FILE);
    let rules_created = if !rules_path.exists() || force {
        let content = render_rules_toml(
            detected_schema_dir.as_deref(),
            &detected_services,
            &detected_domains,
            &detected_modules,
        );
        std::fs::write(&rules_path, content)?;
        true
    } else {
        false
    };

    let rules = Rules::load(root)?;
    let synced = agents::sync(root, rules.as_ref(), false)?;
    let synced_agent_files = synced
        .into_iter()
        .filter(|(_, s)| *s != Status::Unchanged)
        .map(|(p, _)| rel_slash(root, &p))
        .collect();

    let mut hooks_installed = Vec::new();
    let mut hook_notes = Vec::new();
    let agent_lower = agent.to_ascii_lowercase();
    let all = agent_lower == "all";
    if !all && !["claude", "cursor", "antigravity", "git", "none"].contains(&agent_lower.as_str()) {
        anyhow::bail!("Unknown --agent '{agent}'; use claude, cursor, antigravity, git, all or none");
    }

    if (all || agent_lower == "claude") && hook::install_claude(root, "clrinf")? {
        hooks_installed.push(".claude/settings.json".to_string());
    }
    if (all || agent_lower == "cursor") && hook::install_cursor(root, "clrinf")? {
        hooks_installed.push(".cursor/hooks.json".to_string());
    }
    if (all || agent_lower == "antigravity") && hook::install_antigravity(root, "clrinf")? {
        hooks_installed.push(".agents/hooks.json".to_string());
    }
    if (all && root.join(".git").is_dir()) || agent_lower == "git" {
        match hook::install_git(root, "clrinf", force) {
            Ok(_) => hooks_installed.push(".git/hooks/pre-commit".to_string()),
            Err(e) if all => hook_notes.push(format!("Skipped git pre-commit hook: {e}")),
            Err(e) => return Err(e),
        }
    }

    let ci_workflow_path = if ci {
        let wf = root.join(".github/workflows/clrinf-guardrails.yml");
        if !wf.exists() || force {
            if let Some(parent) = wf.parent() {
                std::fs::create_dir_all(parent)?;
            }
            std::fs::write(&wf, render_ci_workflow(detected_schema_dir.is_some()))?;
            Some(wf)
        } else {
            None
        }
    } else {
        None
    };

    let report = verify::run(root, None)?;

    Ok(GuardrailsInitReport {
        rules_path,
        rules_created,
        detected_schema_dir,
        detected_services,
        detected_domains,
        detected_modules,
        synced_agent_files,
        hooks_installed,
        hook_notes,
        ci_workflow_path,
        verify_errors: report.errors_count(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn guardrails_wizard_detects_services_domains_and_passes_verify() {
        let dir = std::env::temp_dir().join(format!("clrinf-guardrails-init-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(dir.join("services/orders/src/domain")).unwrap();
        std::fs::create_dir_all(dir.join("services/billing/src/domain")).unwrap();
        std::fs::write(
            dir.join("services/orders/src/domain/order.rs"),
            "pub struct Order { pub id: String }\n",
        )
        .unwrap();
        std::fs::write(
            dir.join("services/billing/src/domain/invoice.ts"),
            "export interface Invoice { id: string; }\n",
        )
        .unwrap();

        let report = init_guardrails(&dir, "claude", true, false).unwrap();
        assert!(report.rules_created);
        assert_eq!(report.detected_domains.len(), 2);
        assert_eq!(report.detected_modules.len(), 2);
        assert_eq!(report.verify_errors, 0);
        assert!(dir.join("clrinf.rules.toml").is_file());
        assert!(dir.join("AGENTS.md").is_file());
        assert!(dir.join(".cursor/rules/clrinf.mdc").is_file());
        assert!(dir.join(".claude/settings.json").is_file());
        assert!(dir.join(".github/workflows/clrinf-guardrails.yml").is_file());

        let rules_text = std::fs::read_to_string(dir.join("clrinf.rules.toml")).unwrap();
        assert!(rules_text.contains("[[forbid]]"));
        assert!(rules_text.contains("[[boundaries]]"));
        assert!(rules_text.contains("protect = true"));

        let _ = std::fs::remove_dir_all(dir);
    }
}
