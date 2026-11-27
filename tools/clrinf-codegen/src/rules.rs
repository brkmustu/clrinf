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

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum ForbidScope {
    #[default]
    #[serde(alias = "any")]
    Raw,
    Code,
    Imports,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ForbidRule {
    pub id: String,
    /// Path prefixes (relative to the project root) the rule applies to.
    pub paths: Vec<String>,
    /// Text pattern that must not appear in the selected scope under `paths`.
    pub pattern: String,
    pub reason: String,
    /// Optional extension filter (e.g. ["cs", "ts"]); empty means all supported.
    #[serde(default)]
    pub extensions: Vec<String>,
    /// Matching scope: "raw" (default, strips comments only), "code" (strips comments & strings), or "imports".
    #[serde(default)]
    pub scope: ForbidScope,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BoundaryRule {
    pub id: String,
    /// Module root directories (relative to project root) isolated from each other.
    pub modules: Vec<String>,
    /// Optional allowed public segments (e.g. ["contracts", "Contracts", "public", "Public"]).
    #[serde(default)]
    pub public: Vec<String>,
    pub reason: String,
    #[serde(default)]
    pub extensions: Vec<String>,
}

fn default_true() -> bool {
    true
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GeneratedConfig {
    #[serde(default = "default_true")]
    pub protect: bool,
}

impl Default for GeneratedConfig {
    fn default() -> Self {
        Self { protect: true }
    }
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
    pub boundaries: Vec<BoundaryRule>,
    #[serde(default)]
    pub generated: GeneratedConfig,
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
        for rule in &file.boundaries {
            if rule.id.trim().is_empty() || rule.modules.len() < 2 {
                bail!(
                    "{}: boundary rule '{}' needs id and at least two modules",
                    RULES_FILE,
                    rule.id
                );
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
    let rel_file = file
        .path
        .strip_prefix(root)
        .unwrap_or(&file.path)
        .display()
        .to_string();

    match rule.scope {
        ForbidScope::Imports => {
            for stmt in file.imports() {
                if stmt.target.contains(&rule.pattern) || stmt.raw.contains(&rule.pattern) {
                    out.push(RuleIssue {
                        severity: IssueSeverity::Error,
                        code: "RULE_FORBIDDEN_DEPENDENCY".into(),
                        rule_id: rule.id.clone(),
                        message: format!("[{}] {}", rule.id, rule.reason),
                        file: rel_file.clone(),
                        line: stmt.line,
                    });
                }
            }
        }
        ForbidScope::Code => {
            let code = file.code_only();
            for (i, line) in code.lines().enumerate() {
                if line.contains(&rule.pattern) {
                    out.push(RuleIssue {
                        severity: IssueSeverity::Error,
                        code: "RULE_FORBIDDEN_DEPENDENCY".into(),
                        rule_id: rule.id.clone(),
                        message: format!("[{}] {}", rule.id, rule.reason),
                        file: rel_file.clone(),
                        line: i + 1,
                    });
                }
            }
        }
        ForbidScope::Raw => {
            let no_comments = file.without_comments();
            for (i, line) in no_comments.lines().enumerate() {
                if line.contains(&rule.pattern) {
                    out.push(RuleIssue {
                        severity: IssueSeverity::Error,
                        code: "RULE_FORBIDDEN_DEPENDENCY".into(),
                        rule_id: rule.id.clone(),
                        message: format!("[{}] {}", rule.id, rule.reason),
                        file: rel_file.clone(),
                        line: i + 1,
                    });
                }
            }
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

fn module_segment_name(module_dir: &str) -> String {
    let trimmed = module_dir.trim_end_matches(['/', '\\']);
    Path::new(trimmed)
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or(trimmed)
        .to_ascii_lowercase()
}

fn target_segments(target: &str) -> Vec<String> {
    target
        .split(['/', '\\', '.', ':', ';', ' ', '{', '}', ','])
        .filter(|s| !s.is_empty())
        .map(str::to_ascii_lowercase)
        .collect()
}

/// Enforce `[[boundaries]]` module isolation rules. When `only` is given, restrict to those files.
pub fn check_boundaries(rules: &Rules, only: Option<&[PathBuf]>) -> Result<Vec<RuleIssue>> {
    let mut issues = Vec::new();
    for rule in &rules.file.boundaries {
        let entries: Vec<(&str, String)> = rule
            .modules
            .iter()
            .map(|m| (m.as_str(), module_segment_name(m)))
            .collect();
        let public_lower: Vec<String> = rule
            .public
            .iter()
            .map(|p| p.to_ascii_lowercase())
            .collect();

        for (mod_dir, self_mod) in &entries {
            let dir = rules.root.join(mod_dir);
            for file in collect_sources(&dir)? {
                if file.generated {
                    continue;
                }
                if let Some(only) = only {
                    if !only.iter().any(|o| same_file(o, &file.path)) {
                        continue;
                    }
                }
                if !rule.extensions.is_empty() {
                    let ext = file.path.extension().and_then(|e| e.to_str()).unwrap_or("");
                    if !rule.extensions.iter().any(|e| e.eq_ignore_ascii_case(ext)) {
                        continue;
                    }
                }

                let rel_file = file
                    .path
                    .strip_prefix(&rules.root)
                    .unwrap_or(&file.path)
                    .display()
                    .to_string();

                for stmt in file.imports() {
                    let segs = target_segments(&stmt.target);
                    for (_, other_mod) in &entries {
                        if other_mod == self_mod {
                            continue;
                        }
                        if segs.iter().any(|s| s == other_mod) {
                            let is_public = !public_lower.is_empty()
                                && segs.iter().any(|s| public_lower.iter().any(|p| s == p));
                            if !is_public {
                                issues.push(RuleIssue {
                                    severity: IssueSeverity::Error,
                                    code: "RULE_MODULE_BOUNDARY_VIOLATION".into(),
                                    rule_id: rule.id.clone(),
                                    message: format!(
                                        "[{}] Module '{}' imports internal target '{}' from module '{}': {}",
                                        rule.id, self_mod, stmt.target, other_mod, rule.reason
                                    ),
                                    file: rel_file.clone(),
                                    line: stmt.line,
                                });
                            }
                        }
                    }
                }
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
            let scope_note = match f.scope {
                ForbidScope::Imports => " (imports only)",
                ForbidScope::Code => " (code only, strings excluded)",
                ForbidScope::Raw => "",
            };
            lines.push(format!(
                "[{}] `{}` must not contain `{}`{}: {}",
                f.id,
                f.paths.join("`, `"),
                f.pattern,
                scope_note,
                f.reason
            ));
        }
        for b in &r.file.boundaries {
            let pub_note = if b.public.is_empty() {
                String::new()
            } else {
                format!(" (allowed public surfaces: `{}`)", b.public.join("`, `"))
            };
            lines.push(format!(
                "[{}] Modules `{}` are isolated{}: {}",
                b.id,
                b.modules.join("`, `"),
                pub_note,
                b.reason
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
    out.push_str("- C# CRUD: never read generated files under `obj/`. Change behavior with a hook rule (`clrinf rule new --on before-create|before-update|before-delete`) or eject one operation (`clrinfcs eject <Entity> --operation <Op>`).\n");
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

    #[test]
    fn forbid_imports_scope_ignores_non_import_mentions_and_strings() {
        let dir = std::env::temp_dir().join(format!("clrinf-rules-imports-{}", std::process::id()));
        let domain = dir.join("src/domain");
        std::fs::create_dir_all(&domain).unwrap();
        std::fs::write(
            domain.join("order.rs"),
            "/* /* Infrastructure */ */\nuse crate::Infrastructure::Repo;\npub fn label() -> &'static str { \"Infrastructure\" }\npub struct InfrastructureDoc;\n",
        )
        .unwrap();
        std::fs::write(
            domain.join("item.cs"),
            "using System;\nusing var conn = OpenInfrastructure();\nusing App.Infrastructure.Db;\n",
        )
        .unwrap();
        let rules = rules_with(
            "[[forbid]]\nid=\"domain-no-infra\"\nscope=\"imports\"\npaths=[\"src/domain\"]\npattern=\"Infrastructure\"\nreason=\"no infra imports\"\n",
            &dir,
        );
        let issues = check_forbid(&rules, None).unwrap();
        assert_eq!(issues.len(), 2);
        assert_eq!(issues[0].file.replace('\\', "/"), "src/domain/item.cs");
        assert_eq!(issues[0].line, 3);
        assert_eq!(issues[1].file.replace('\\', "/"), "src/domain/order.rs");
        assert_eq!(issues[1].line, 2);
        let _ = std::fs::remove_dir_all(dir);
    }

    #[test]
    fn forbid_code_scope_ignores_string_literals_while_raw_keeps_them() {
        let dir = std::env::temp_dir().join(format!("clrinf-rules-scope-{}", std::process::id()));
        let domain = dir.join("src/domain");
        std::fs::create_dir_all(&domain).unwrap();
        std::fs::write(
            domain.join("msg.cs"),
            "var a = @\"Infrastructure\";\n// Infrastructure\n",
        )
        .unwrap();
        let code_rules = rules_with(
            "[[forbid]]\nid=\"no-infra-code\"\nscope=\"code\"\npaths=[\"src/domain\"]\npattern=\"Infrastructure\"\nreason=\"no infra\"\n",
            &dir,
        );
        assert!(check_forbid(&code_rules, None).unwrap().is_empty());

        let raw_rules = rules_with(
            "[[forbid]]\nid=\"no-infra-raw\"\npaths=[\"src/domain\"]\npattern=\"Infrastructure\"\nreason=\"no infra\"\n",
            &dir,
        );
        let issues = check_forbid(&raw_rules, None).unwrap();
        assert_eq!(issues.len(), 1);
        assert_eq!(issues[0].line, 1);
        let _ = std::fs::remove_dir_all(dir);
    }

    #[test]
    fn boundary_rules_block_internal_cross_module_imports_and_allow_public_contracts() {
        let dir = std::env::temp_dir().join(format!("clrinf-rules-boundary-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(dir.join("services/orders/src")).unwrap();
        std::fs::create_dir_all(dir.join("services/billing/src")).unwrap();

        std::fs::write(
            dir.join("services/orders/src/handler.ts"),
            "import { Local } from './domain';\nimport { BillingEvent } from '../../billing/contracts/events';\nimport { Ledger } from '../../billing/src/internal/ledger';\n// import { Fake } from '../../billing/src/secret';\n",
        )
        .unwrap();
        std::fs::write(
            dir.join("services/billing/src/app.rs"),
            "use crate::billing_internal::Calc;\nuse orders::internal::OrderRepo;\n",
        )
        .unwrap();

        let rules = rules_with(
            "[[boundaries]]\nid=\"module-isolation\"\nmodules=[\"services/orders\", \"services/billing\"]\npublic=[\"contracts\", \"Contracts\"]\nreason=\"Import only public contracts across modules\"\n",
            &dir,
        );
        let issues = check_boundaries(&rules, None).unwrap();
        assert_eq!(issues.len(), 2, "{issues:?}");
        assert_eq!(issues[0].code, "RULE_MODULE_BOUNDARY_VIOLATION");
        assert_eq!(issues[0].file.replace('\\', "/"), "services/billing/src/app.rs");
        assert_eq!(issues[0].line, 2);
        assert_eq!(issues[1].file.replace('\\', "/"), "services/orders/src/handler.ts");
        assert_eq!(issues[1].line, 3);

        let lines = constraint_lines(Some(&rules));
        assert!(lines.iter().any(|l| l.contains("module-isolation") && l.contains("contracts")));
        let _ = std::fs::remove_dir_all(dir);
    }
}
