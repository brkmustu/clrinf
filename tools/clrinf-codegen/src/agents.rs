//! Keep AI-agent instruction files in sync with `clrinf.rules.toml`.
//!
//! A managed block (between `<!-- clrinf:start -->` / `<!-- clrinf:end -->`)
//! is written into AGENTS.md / CLAUDE.md / GEMINI.md (when present) and a
//! Cursor rule file. Everything outside the block is left untouched.

use crate::rules::{render_agent_block, Rules, BLOCK_END, BLOCK_START};
use anyhow::Result;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Status {
    Created,
    Updated,
    Unchanged,
}

fn upsert_block(existing: &str, block: &str) -> String {
    if let (Some(start), Some(end)) = (existing.find(BLOCK_START), existing.find(BLOCK_END)) {
        if end > start {
            let after = &existing[end + BLOCK_END.len()..];
            let after = after.strip_prefix('\n').unwrap_or(after);
            return format!("{}{}{}", &existing[..start], block, after);
        }
    }
    if existing.trim().is_empty() {
        return block.to_string();
    }
    let sep = if existing.ends_with('\n') { "\n" } else { "\n\n" };
    format!("{existing}{sep}{block}")
}

fn write_if_changed(path: &Path, content: &str, check: bool) -> Result<Status> {
    let existing = std::fs::read_to_string(path).ok();
    let status = match &existing {
        None => Status::Created,
        Some(old) if old == content => return Ok(Status::Unchanged),
        Some(_) => Status::Updated,
    };
    if !check {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::write(path, content)?;
    }
    Ok(status)
}

/// Sync all instruction files. With `check`, nothing is written.
pub fn sync(root: &Path, rules: Option<&Rules>, check: bool) -> Result<Vec<(PathBuf, Status)>> {
    let block = render_agent_block(rules);
    let mut results = Vec::new();

    let docs = ["AGENTS.md", "CLAUDE.md", "GEMINI.md"];
    let existing: Vec<&str> = docs.iter().copied().filter(|d| root.join(d).is_file()).collect();
    let targets: Vec<&str> = if existing.is_empty() { vec!["AGENTS.md"] } else { existing };
    for name in targets {
        let path = root.join(name);
        let current = std::fs::read_to_string(&path).unwrap_or_default();
        let updated = upsert_block(&current, &block);
        let status = if !path.is_file() {
            if !check {
                std::fs::write(&path, &updated)?;
            }
            Status::Created
        } else {
            write_if_changed(&path, &updated, check)?
        };
        results.push((path, status));
    }

    let cursor = root.join(".cursor").join("rules").join("clrinf.mdc");
    let cursor_content = format!(
        "---\ndescription: clrinf architecture guardrails (generated from clrinf.rules.toml)\nalwaysApply: true\n---\n{block}"
    );
    results.push((cursor.clone(), write_if_changed(&cursor, &cursor_content, check)?));
    Ok(results)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn upsert_appends_then_replaces_in_place() {
        let block1 = format!("{BLOCK_START}\none\n{BLOCK_END}\n");
        let block2 = format!("{BLOCK_START}\ntwo\n{BLOCK_END}\n");
        let first = upsert_block("# Title\nkeep me\n", &block1);
        assert!(first.starts_with("# Title") && first.contains("one"));
        let second = upsert_block(&first, &block2);
        assert!(second.contains("two") && !second.contains("one") && second.contains("keep me"));
        assert_eq!(upsert_block(&second, &block2), second);
    }

    #[test]
    fn sync_creates_files_and_check_mode_does_not_write() {
        let root = std::env::temp_dir().join(format!("clrinf-agents-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(&root).unwrap();
        let dry = sync(&root, None, true).unwrap();
        assert!(dry.iter().all(|(_, s)| *s == Status::Created));
        assert!(!root.join("AGENTS.md").exists());
        sync(&root, None, false).unwrap();
        assert!(root.join("AGENTS.md").exists() && root.join(".cursor/rules/clrinf.mdc").exists());
        let again = sync(&root, None, true).unwrap();
        assert!(again.iter().all(|(_, s)| *s == Status::Unchanged));
        let _ = std::fs::remove_dir_all(root);
    }
}
