//! Single source of truth for project architecture rules (`clrinf.rules.toml`).
//!
//! The same file drives (a) dependency-rule enforcement in `verify` and the
//! agent hooks, (b) the event/service topology mapping used by drift checks,
//! (c) the generated instruction blocks for AI agents (`agents sync`), and
//! (d) the default MCP tool profile. Instructions given to the agent and rules
//! enforced by tooling therefore cannot diverge.

use crate::scan::{collect_sources, SourceFile};
use crate::topology::IssueSeverity;
use anyhow::{bail, Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

pub const RULES_FILE: &str = "clrinf.rules.toml";
pub const DEFAULT_SCHEMA_DIR: &str = "tools/clrinf-codegen/schemas";

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EventsConfig {
    pub schema_dir: Option<String>,
    /// Service name -> source directory (relative to the project root).
    #[serde(default)]
    pub services: BTreeMap<String, String>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ForbidRule {
    pub id: String,
    /// Path prefixes (relative to the project root) the rule applies to.
    pub paths: Vec<String>,
    /// Literal text that must not appear in files under `paths`.
    pub pattern: String,
    pub reason: String,
    /// Optional extension filter (e.g. ["cs", "ts"]); empty means all supported.
    #[serde(default)]
    pub extensions: Vec<String>,
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AgentsConfig {
    #[serde(default)]
    pub extra: Vec<String>,
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct McpConfig {
    pub profile: Option<String>,
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RulesFile {
    #[serde(default)]
    pub events: EventsConfig,
    #[serde(default)]
    pub forbid: Vec<ForbidRule>,
    #[serde(default)]
    pub agents: AgentsConfig,
    #[serde(default)]
    pub mcp: McpConfig,
}

#[derive(Debug, Clone)]
pub struct Rules {
    pub root: PathBuf,
    pub file: RulesFile,
}

impl Rules {
    /// Load `<root>/clrinf.rules.toml`; `Ok(None)` when it does not exist.
    pub fn load(root: &Path) -> Result<Option<Self>> {
        let path = root.join(RULES_FILE);
        if !path.is_file() {
            return Ok(None);
        }
        let text = std::fs::read_to_string(&path).with_context(|| format!("Reading {}", path.display()))?;
        let file: RulesFile = toml::from_str(&text).with_context(|| format!("Invalid {}", path.display()))?;
        for rule in &file.forbid {
            if rule.id.trim().is_empty() || rule.pattern.is_empty() || rule.paths.is_empty() {
                bail!("{}: forbid rule '{}' needs id, pattern and at least one path", RULES_FILE, rule.id);
            }
        }
        Ok(Some(Self { root: root.to_path_buf(), file }))
    }

    pub fn schema_dir(&self) -> PathBuf {
        self.root.join(self.file.events.schema_dir.as_deref().unwrap_or(DEFAULT_SCHEMA_DIR))
    }

    pub fn service_dirs(&self) -> Vec<(String, PathBuf)> {
        self.file
            .events
            .services
            .iter()
            .map(|(name, dir)| (name.clone(), self.root.join(dir)))
            .collect()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuleIssue {
    pub severity: IssueSeverity,
    pub code: String,
    pub rule_id: String,
    pub message: String,
    pub file: String,
    pub line: usize,
}

fn is_comment(line: &str) -> bool {
    let t = line.trim_start();
    t.starts_with("//") || t.starts_with("/*") || t.starts_with('*')
}

fn same_file(a: &Path, b: &Path) -> bool {
    match (a.canonicalize(), b.canonicalize()) {
        (Ok(a), Ok(b)) => a == b,
        _ => a == b,
    }
}

fn check_file(rule: &ForbidRule, root: &Path, file: &SourceFile, out: &mut Vec<RuleIssue>) {
    if file.generated {
        return;
    }
    if !rule.extensions.is_empty() {
        let ext = file.path.extension().and_then(|e| e.to_str()).unwrap_or("");
        if !rule.extensions.iter().any(|e| e.eq_ignore_ascii_case(ext)) {
            return;
        }
    }
    for (i, line) in file.content.lines().enumerate() {
        if !is_comment(line) && line.contains(&rule.pattern) {
            out.push(RuleIssue {
                severity: IssueSeverity::Error,
                code: "RULE_FORBIDDEN_DEPENDENCY".into(),
                rule_id: rule.id.clone(),
                message: format!("[{}] {}", rule.id, rule.reason),
                file: file.path.strip_prefix(root).unwrap_or(&file.path).display().to_string(),
                line: i + 1,
            });
        }
    }
}

/// Enforce `[[forbid]]` rules. When `only` is given, restrict to those files.
pub fn check_forbid(rules: &Rules, only: Option<&[PathBuf]>) -> Result<Vec<RuleIssue>> {
    let mut issues = Vec::new();
    for rule in &rules.file.forbid {
        for prefix in &rule.paths {
            let dir = rules.root.join(prefix);
            for file in collect_sources(&dir)? {
                if let Some(only) = only {
                    if !only.iter().any(|o| same_file(o, &file.path)) {
                        continue;
                    }
                }
                check_file(rule, &rules.root, &file, &mut issues);
            }
        }
    }
    issues.sort_by(|a, b| (&a.file, a.line, &a.rule_id).cmp(&(&b.file, b.line, &b.rule_id)));
    issues.dedup_by(|a, b| a.file == b.file && a.line == b.line && a.rule_id == b.rule_id);
    Ok(issues)
}

/// Constraint sentences for agents derived from the rules (used by plans and `agents sync`).
pub fn constraint_lines(rules: Option<&Rules>) -> Vec<String> {
    let mut lines = vec![
        "Never edit files marked `Code generated by clrinf-codegen`; change the schema and regenerate.".to_string(),
        "Never write to the outbox directly; publish through the generated publisher.".to_string(),
    ];
    if let Some(r) = rules {
        for f in &r.file.forbid {
            lines.push(format!(
                "[{}] `{}` must not contain `{}`: {}",
                f.id,
                f.paths.join("`, `"),
                f.pattern,
                f.reason
            ));
        }
        lines.extend(r.file.agents.extra.iter().cloned());
    }
    lines
}

pub const BLOCK_START: &str = "<!-- clrinf:start -->";
pub const BLOCK_END: &str = "<!-- clrinf:end -->";

/// Managed instruction block for AGENTS.md / CLAUDE.md / Cursor rules.
pub fn render_agent_block(rules: Option<&Rules>) -> String {
    let mut out = String::new();
    out.push_str(BLOCK_START);
    out.push_str("\n## clrinf — architecture guardrails (generated)\n\n");
    out.push_str(&format!(
        "Source of truth: `{RULES_FILE}`. Regenerate with `clrinf agents sync`; do not edit this block by hand.\n\n"
    ));
    out.push_str("### Workflow\n");
    out.push_str("- Before changing or adding an event, call `clrinf_plan_change` (or `clrinf topology impact <event>`) and load only the files it lists.\n");
    out.push_str("- Register publishers/subscribers with `clrinf event subscribe|publish`; implement only the handler body or publish call site.\n");
    out.push_str("- Verify with `clrinf verify --changed` (runs automatically after edits when `clrinf hook install` was used).\n\n");
    out.push_str("### Rules\n");
    for line in constraint_lines(rules) {
        out.push_str(&format!("- {line}\n"));
    }
    if let Some(r) = rules {
        let services = r.service_dirs();
        out.push_str("\n### Event topology\n");
        out.push_str(&format!(
            "- Schemas: `{}`\n",
            r.file.events.schema_dir.as_deref().unwrap_or(DEFAULT_SCHEMA_DIR)
        ));
        for (name, _) in &services {
            let dir = &r.file.events.services[name];
            out.push_str(&format!("- Service `{name}` lives in `{dir}`\n"));
        }
    }
    out.push_str(BLOCK_END);
    out.push('\n');
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rules_with(toml_text: &str, root: &Path) -> Rules {
        Rules { root: root.to_path_buf(), file: toml::from_str(toml_text).unwrap() }
    }

    #[test]
    fn parses_and_rejects_unknown_keys() {
        assert!(toml::from_str::<RulesFile>("[events]\nschema_dir = \"s\"\n[events.services]\nbilling = \"b\"").is_ok());
        assert!(toml::from_str::<RulesFile>("bogus = 1").is_err());
    }

    #[test]
    fn forbid_rule_flags_code_but_not_comments_or_generated() {
        let dir = std::env::temp_dir().join(format!("clrinf-rules-test-{}", std::process::id()));
        let domain = dir.join("src/domain");
        std::fs::create_dir_all(&domain).unwrap();
        std::fs::write(domain.join("a.ts"), "import x from \"../Infrastructure/db\";\n// Infrastructure note\n").unwrap();
        std::fs::write(
            domain.join("gen.ts"),
            "// Code generated by clrinf-codegen. DO NOT EDIT.\nimport Infrastructure from \"x\";\n",
        )
        .unwrap();
        let rules = rules_with(
            "[[forbid]]\nid=\"domain-no-infra\"\npaths=[\"src/domain\"]\npattern=\"Infrastructure\"\nreason=\"no infra\"\n",
            &dir,
        );
        let issues = check_forbid(&rules, None).unwrap();
        assert_eq!(issues.len(), 1);
        assert_eq!(issues[0].line, 1);
        let block = render_agent_block(Some(&rules));
        assert!(block.contains("domain-no-infra") && block.starts_with(BLOCK_START));
        let _ = std::fs::remove_dir_all(dir);
    }
}
