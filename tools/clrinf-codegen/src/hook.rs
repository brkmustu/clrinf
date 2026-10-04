//! Agent hooks: make the guardrails run on their own.
//!
//! * `hook install --agent claude` registers a Claude Code `PostToolUse` hook
//!   so every file the agent writes is checked incrementally; violations are
//!   returned to the agent on stderr (exit code 2) and it can self-correct.
//! * `hook install --agent git` installs a `pre-commit` hook running
//!   `verify --changed`.
//! * `hook install --agent cursor` registers a Cursor `postToolUse` hook
//!   (`.cursor/hooks.json`); violations return as `additional_context`.
//! * `hook install --agent antigravity` registers a named Antigravity
//!   `PostToolUse` hook (`.agents/hooks.json`, absolute command path).
//! * `hook run` is the entry point the agent hook calls (payload on stdin);
//!   `--format claude|cursor|antigravity` selects how feedback is returned.

use crate::verify;
use anyhow::{bail, Context, Result};
use serde_json::{json, Value};
use std::path::{Path, PathBuf};
use std::process::Command;

const GIT_MARKER: &str = "clrinf-managed pre-commit hook";

fn is_ours(command: &str) -> bool {
    command.contains("clrinf") && command.contains(" hook run")
}

fn claude_settings_path(root: &Path) -> PathBuf {
    root.join(".claude").join("settings.json")
}

fn load_json(path: &Path) -> Result<Value> {
    if !path.is_file() {
        return Ok(json!({}));
    }
    let text = std::fs::read_to_string(path)?;
    if text.trim().is_empty() {
        return Ok(json!({}));
    }
    serde_json::from_str(&text).with_context(|| format!("{} is not valid JSON", path.display()))
}

/// Returns true when the settings file was modified.
pub fn install_claude(root: &Path, command: &str) -> Result<bool> {
    let path = claude_settings_path(root);
    let mut settings = load_json(&path)?;
    let Some(obj) = settings.as_object_mut() else { bail!("{} must contain a JSON object", path.display()) };
    let hooks = obj.entry("hooks").or_insert_with(|| json!({}));
    let Some(hooks) = hooks.as_object_mut() else { bail!("'hooks' must be an object") };
    let post = hooks.entry("PostToolUse").or_insert_with(|| json!([]));
    let Some(post) = post.as_array_mut() else { bail!("'hooks.PostToolUse' must be an array") };

    let already = post.iter().any(|entry| {
        entry["hooks"]
            .as_array()
            .map_or(false, |hs| hs.iter().any(|h| h["command"].as_str().map_or(false, is_ours)))
    });
    if already {
        return Ok(false);
    }
    post.push(json!({
        "matcher": "Write|Edit|MultiEdit",
        "hooks": [{
            "type": "command",
            "command": format!("{command} hook run"),
            "timeout": 15
        }]
    }));
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(&path, serde_json::to_string_pretty(&settings)? + "\n")?;
    Ok(true)
}

pub fn uninstall_claude(root: &Path) -> Result<bool> {
    let path = claude_settings_path(root);
    if !path.is_file() {
        return Ok(false);
    }
    let mut settings = load_json(&path)?;
    let mut changed = false;
    if let Some(post) = settings["hooks"]["PostToolUse"].as_array_mut() {
        let before = post.len();
        post.retain(|entry| {
            !entry["hooks"]
                .as_array()
                .map_or(false, |hs| hs.iter().any(|h| h["command"].as_str().map_or(false, is_ours)))
        });
        changed = post.len() != before;
    }
    if changed {
        std::fs::write(&path, serde_json::to_string_pretty(&settings)? + "\n")?;
    }
    Ok(changed)
}

const CURSOR_MATCHER: &str = "Write|Edit|StrReplace|MultiEdit|EditNotebook";
const ANTIGRAVITY_MATCHER: &str = "write_to_file|replace_file_content|multi_replace_file_content";
const ANTIGRAVITY_KEY: &str = "clrinf";

fn write_json(path: &Path, value: &Value) -> Result<()> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(path, serde_json::to_string_pretty(value)? + "\n")?;
    Ok(())
}

/// Quote a command part when it contains whitespace.
fn quote(s: &str) -> String {
    if s.contains(' ') { format!("\"{s}\"") } else { s.to_string() }
}

/// Cursor: `.cursor/hooks.json` (`version: 1`, `hooks.postToolUse[]`).
/// Returns true when the file was modified.
pub fn install_cursor(root: &Path, command: &str) -> Result<bool> {
    let path = root.join(".cursor").join("hooks.json");
    let mut cfg = load_json(&path)?;
    let Some(obj) = cfg.as_object_mut() else { bail!("{} must contain a JSON object", path.display()) };
    obj.entry("version").or_insert(json!(1));
    let hooks = obj.entry("hooks").or_insert_with(|| json!({}));
    let Some(hooks) = hooks.as_object_mut() else { bail!("'hooks' must be an object") };
    let post = hooks.entry("postToolUse").or_insert_with(|| json!([]));
    let Some(post) = post.as_array_mut() else { bail!("'hooks.postToolUse' must be an array") };
    if post.iter().any(|h| h["command"].as_str().map_or(false, is_ours)) {
        return Ok(false);
    }
    post.push(json!({
        "command": format!("{} hook run --format cursor", quote(command)),
        "matcher": CURSOR_MATCHER,
        "timeout": 15
    }));
    write_json(&path, &cfg)?;
    Ok(true)
}

pub fn uninstall_cursor(root: &Path) -> Result<bool> {
    let path = root.join(".cursor").join("hooks.json");
    if !path.is_file() {
        return Ok(false);
    }
    let mut cfg = load_json(&path)?;
    let mut changed = false;
    if let Some(post) = cfg["hooks"]["postToolUse"].as_array_mut() {
        let before = post.len();
        post.retain(|h| !h["command"].as_str().map_or(false, is_ours));
        changed = post.len() != before;
    }
    if changed {
        write_json(&path, &cfg)?;
    }
    Ok(changed)
}

/// Antigravity: `.agents/hooks.json`, a named hook `clrinf` with a
/// `PostToolUse` matcher. Antigravity requires an absolute command path, so
/// `command` is resolved against the current executable when it is not absolute.
pub fn install_antigravity(root: &Path, command: &str) -> Result<bool> {
    let path = root.join(".agents").join("hooks.json");
    let mut cfg = load_json(&path)?;
    let Some(obj) = cfg.as_object_mut() else { bail!("{} must contain a JSON object", path.display()) };
    let exe = if Path::new(command).is_absolute() {
        command.to_string()
    } else {
        std::env::current_exe()
            .map(|p| p.to_string_lossy().replace('\\', "/"))
            .unwrap_or_else(|_| command.to_string())
    };
    let abs_root = std::fs::canonicalize(root).unwrap_or_else(|_| root.to_path_buf());
    let abs_root = abs_root.to_string_lossy().trim_start_matches(r"\\?\").replace('\\', "/");
    let entry = json!({
        "PostToolUse": [{
            "matcher": ANTIGRAVITY_MATCHER,
            "hooks": [{
                "type": "command",
                "command": format!("{} hook run --format antigravity --path {}", quote(&exe), quote(&abs_root)),
                "timeout": 15
            }]
        }]
    });
    if obj.get(ANTIGRAVITY_KEY) == Some(&entry) {
        return Ok(false);
    }
    obj.insert(ANTIGRAVITY_KEY.to_string(), entry);
    write_json(&path, &cfg)?;
    Ok(true)
}

pub fn uninstall_antigravity(root: &Path) -> Result<bool> {
    let path = root.join(".agents").join("hooks.json");
    if !path.is_file() {
        return Ok(false);
    }
    let mut cfg = load_json(&path)?;
    if !cfg[ANTIGRAVITY_KEY].to_string().contains(" hook run") {
        return Ok(false);
    }
    if let Some(obj) = cfg.as_object_mut() {
        obj.remove(ANTIGRAVITY_KEY);
    }
    write_json(&path, &cfg)?;
    Ok(true)
}

/// How feedback is handed back to the calling agent.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Format {
    /// stderr + exit code 2 (Claude Code).
    Claude,
    /// stdout `{"additional_context": ...}`, exit 0 (Cursor postToolUse).
    Cursor,
    /// stdout `{"decision":"block","reason":...}`, exit 0 (Antigravity).
    Antigravity,
}

impl Format {
    pub fn parse(s: &str) -> Result<Self> {
        match s.to_ascii_lowercase().as_str() {
            "claude" => Ok(Self::Claude),
            "cursor" => Ok(Self::Cursor),
            "antigravity" => Ok(Self::Antigravity),
            other => bail!("Unknown --format '{other}'; use claude, cursor or antigravity"),
        }
    }
}

/// Result of a hook invocation: what to print where, and the exit code.
#[derive(Debug, PartialEq, Eq)]
pub struct Outcome {
    pub code: i32,
    pub stdout: String,
    pub stderr: String,
}

/// Run the check and render the feedback in the agent's dialect.
pub fn run_formatted(root: &Path, stdin: &str, format: Format) -> Outcome {
    let (code, message) = run(root, stdin);
    if message.is_empty() {
        return Outcome { code, stdout: String::new(), stderr: String::new() };
    }
    match (format, code) {
        (Format::Claude, _) | (_, 1) => Outcome { code, stdout: String::new(), stderr: message },
        (Format::Cursor, _) => Outcome {
            code: 0,
            stdout: json!({ "additional_context": message }).to_string(),
            stderr: String::new(),
        },
        (Format::Antigravity, _) => Outcome {
            code: 0,
            stdout: json!({ "decision": "block", "reason": message }).to_string(),
            stderr: String::new(),
        },
    }
}

/// Pull the edited file path out of any supported agent payload.
fn payload_file(payload: &Value) -> Option<String> {
    const CONTAINERS: [&str; 6] = ["tool_input", "toolInput", "tool_args", "toolArgs", "args", "arguments"];
    const KEYS: [&str; 6] = ["file_path", "path", "TargetFile", "AbsolutePath", "filePath", "target_file"];
    for key in KEYS {
        if let Some(s) = payload[key].as_str() {
            return Some(s.to_string());
        }
    }
    for container in CONTAINERS {
        for key in KEYS {
            if let Some(s) = payload[container][key].as_str() {
                return Some(s.to_string());
            }
        }
    }
    None
}

pub fn install_git(root: &Path, command: &str, force: bool) -> Result<PathBuf> {
    let git_dir = root.join(".git");
    if !git_dir.is_dir() {
        bail!("{} is not a git repository root (.git directory not found)", root.display());
    }
    let hooks_dir = git_dir.join("hooks");
    std::fs::create_dir_all(&hooks_dir)?;
    let path = hooks_dir.join("pre-commit");
    if path.exists() && !force {
        let existing = std::fs::read_to_string(&path).unwrap_or_default();
        if !existing.contains(GIT_MARKER) {
            bail!("A different pre-commit hook already exists at {}; use --force to replace it", path.display());
        }
    }
    let script = format!("#!/bin/sh\n# {GIT_MARKER}\n{command} verify --changed || exit 1\n");
    std::fs::write(&path, script)?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o755))?;
    }
    Ok(path)
}

pub fn uninstall_git(root: &Path) -> Result<bool> {
    let path = root.join(".git").join("hooks").join("pre-commit");
    if path.is_file() && std::fs::read_to_string(&path).unwrap_or_default().contains(GIT_MARKER) {
        std::fs::remove_file(&path)?;
        return Ok(true);
    }
    Ok(false)
}

/// Files changed relative to HEAD plus untracked files (absolute paths).
pub fn changed_files(root: &Path) -> Vec<PathBuf> {
    let mut names = Vec::new();
    for args in [
        vec!["diff", "--name-only", "--relative", "HEAD"],
        vec!["ls-files", "--others", "--exclude-standard"],
    ] {
        if let Ok(out) = Command::new("git").args(&args).current_dir(root).output() {
            if out.status.success() {
                names.extend(String::from_utf8_lossy(&out.stdout).lines().map(str::to_string));
            }
        }
    }
    names.sort();
    names.dedup();
    names.into_iter().map(|n| root.join(n)).filter(|p| p.is_file()).collect()
}

/// Handle one agent hook invocation. Returns (exit code, stderr text).
/// Exit code 2 blocks/feeds back to the agent; 0 stays silent.
pub fn run(root: &Path, stdin: &str) -> (i32, String) {
    let payload: Value = serde_json::from_str(stdin).unwrap_or(Value::Null);
    let Some(raw) = payload_file(&payload) else { return (0, String::new()) };
    let file = if Path::new(&raw).is_absolute() { PathBuf::from(&raw) } else { root.join(&raw) };
    if !file.is_file() {
        return (0, String::new());
    }
    match verify::run(root, Some(&[file.clone()])) {
        Ok(report) if report.errors_count() > 0 => (
            2,
            format!(
                "clrinf architecture check failed after editing {}:\n{}Fix these before continuing (rules: clrinf.rules.toml; details: `clrinf verify --changed`).\n",
                file.strip_prefix(root).unwrap_or(&file).display(),
                report.format_text(true)
            ),
        ),
        Ok(_) => (0, String::new()),
        Err(e) => (1, format!("clrinf hook could not run checks: {e:#}\n")),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tmp(name: &str) -> PathBuf {
        let d = std::env::temp_dir().join(format!("clrinf-hook-{name}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&d);
        std::fs::create_dir_all(&d).unwrap();
        d
    }

    #[test]
    fn claude_install_is_idempotent_and_preserves_existing_hooks() {
        let root = tmp("claude");
        std::fs::create_dir_all(root.join(".claude")).unwrap();
        std::fs::write(
            root.join(".claude/settings.json"),
            r#"{"permissions":{"allow":["x"]},"hooks":{"PostToolUse":[{"matcher":"Bash","hooks":[{"type":"command","command":"other"}]}]}}"#,
        )
        .unwrap();
        assert!(install_claude(&root, "clrinf").unwrap());
        assert!(!install_claude(&root, "clrinf").unwrap());
        let v = load_json(&root.join(".claude/settings.json")).unwrap();
        assert_eq!(v["hooks"]["PostToolUse"].as_array().unwrap().len(), 2);
        assert_eq!(v["permissions"]["allow"][0], "x");
        assert!(uninstall_claude(&root).unwrap());
        let v = load_json(&root.join(".claude/settings.json")).unwrap();
        assert_eq!(v["hooks"]["PostToolUse"].as_array().unwrap().len(), 1);
        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn cursor_install_is_idempotent_and_preserves_existing_hooks() {
        let root = tmp("cursor");
        std::fs::create_dir_all(root.join(".cursor")).unwrap();
        std::fs::write(
            root.join(".cursor/hooks.json"),
            r#"{"version":1,"hooks":{"afterFileEdit":[{"command":"fmt.sh"}],"postToolUse":[{"command":"other"}]}}"#,
        )
        .unwrap();
        assert!(install_cursor(&root, "clrinf").unwrap());
        assert!(!install_cursor(&root, "clrinf").unwrap());
        let v = load_json(&root.join(".cursor/hooks.json")).unwrap();
        assert_eq!(v["hooks"]["postToolUse"].as_array().unwrap().len(), 2);
        assert_eq!(v["hooks"]["afterFileEdit"][0]["command"], "fmt.sh");
        assert!(uninstall_cursor(&root).unwrap());
        let v = load_json(&root.join(".cursor/hooks.json")).unwrap();
        assert_eq!(v["hooks"]["postToolUse"].as_array().unwrap().len(), 1);
        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn antigravity_install_uses_absolute_command_and_preserves_other_hooks() {
        let root = tmp("agy");
        std::fs::create_dir_all(root.join(".agents")).unwrap();
        std::fs::write(root.join(".agents/hooks.json"), r#"{"mine":{"Stop":[]}}"#).unwrap();
        assert!(install_antigravity(&root, "clrinf").unwrap());
        assert!(!install_antigravity(&root, "clrinf").unwrap());
        let v = load_json(&root.join(".agents/hooks.json")).unwrap();
        let cmd = v["clrinf"]["PostToolUse"][0]["hooks"][0]["command"].as_str().unwrap().trim_matches('"').to_string();
        assert!(Path::new(cmd.split(" hook run").next().unwrap().trim_matches('"')).is_absolute(), "{cmd}");
        assert!(v["mine"].is_object());
        assert!(uninstall_antigravity(&root).unwrap());
        let v = load_json(&root.join(".agents/hooks.json")).unwrap();
        assert!(v.get("clrinf").is_none() && v["mine"].is_object());
        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn payload_file_understands_every_agent_dialect() {
        for (payload, want) in [
            (r#"{"tool_input":{"file_path":"a.ts"}}"#, "a.ts"),
            (r#"{"file_path":"b.ts","edits":[]}"#, "b.ts"),
            (r#"{"toolArgs":{"TargetFile":"c.rs"}}"#, "c.rs"),
        ] {
            let v: Value = serde_json::from_str(payload).unwrap();
            assert_eq!(payload_file(&v).as_deref(), Some(want));
        }
    }

    #[test]
    fn format_parse_rejects_unknown() {
        assert_eq!(Format::parse("Cursor").unwrap(), Format::Cursor);
        assert!(Format::parse("vim").is_err());
    }

    #[test]
    fn hook_run_ignores_irrelevant_payloads() {
        let root = tmp("run");
        assert_eq!(run(&root, "not json").0, 0);
        assert_eq!(run(&root, r#"{"tool_input":{"file_path":"missing.ts"}}"#).0, 0);
        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn git_hook_refuses_to_overwrite_foreign_hook() {
        let root = tmp("git");
        std::fs::create_dir_all(root.join(".git/hooks")).unwrap();
        std::fs::write(root.join(".git/hooks/pre-commit"), "#!/bin/sh\necho mine\n").unwrap();
        assert!(install_git(&root, "clrinf", false).is_err());
        assert!(install_git(&root, "clrinf", true).is_ok());
        assert!(uninstall_git(&root).unwrap());
        let _ = std::fs::remove_dir_all(root);
    }
}
