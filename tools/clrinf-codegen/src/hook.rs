//! Agent hooks: make the guardrails run on their own.
//!
//! * `hook install --agent claude` registers a Claude Code `PostToolUse` hook
//!   so every file the agent writes is checked incrementally; violations are
//!   returned to the agent on stderr (exit code 2) and it can self-correct.
//! * `hook install --agent git` installs a `pre-commit` hook running
//!   `verify --changed`.
//! * `hook run` is the entry point the agent hook calls (payload on stdin).

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
            "timeout": 15000
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
    let raw = payload["tool_input"]["file_path"]
        .as_str()
        .or_else(|| payload["tool_input"]["path"].as_str())
        .or_else(|| payload["file_path"].as_str());
    let Some(raw) = raw else { return (0, String::new()) };
    let file = if Path::new(raw).is_absolute() { PathBuf::from(raw) } else { root.join(raw) };
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
