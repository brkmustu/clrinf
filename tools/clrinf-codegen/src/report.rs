//! Topology change report for pull requests.
//!
//! Compares a *base* schema set (a directory or a git ref) against the *head*
//! schema set and renders a compact summary a human reviewer can scan in
//! seconds: new/removed events, subscriber/publisher changes, breaking field
//! changes, and newly introduced topology diagnostics.

use crate::compat::{diff_schemas, SchemaChange};
use crate::schema::{parse_schema_file, ParsedSchema};
use crate::topology::{IssueSeverity, TopologyIssue, TopologyValidator};
use crate::validator::discover_files;
use anyhow::{bail, Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicUsize, Ordering};

static NEXT: AtomicUsize = AtomicUsize::new(0);

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventChange {
    pub event_type: String,
    pub added_publishers: Vec<String>,
    pub removed_publishers: Vec<String>,
    pub added_subscribers: Vec<String>,
    pub removed_subscribers: Vec<String>,
    pub field_changes: Vec<SchemaChange>,
}

impl EventChange {
    fn is_empty(&self) -> bool {
        self.added_publishers.is_empty()
            && self.removed_publishers.is_empty()
            && self.added_subscribers.is_empty()
            && self.removed_subscribers.is_empty()
            && self.field_changes.is_empty()
    }

    fn has_breaking(&self) -> bool {
        crate::compat::has_breaking(&self.field_changes) || !self.removed_subscribers.is_empty()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TopologyDiff {
    pub added_events: Vec<String>,
    pub removed_events: Vec<String>,
    pub changed_events: Vec<EventChange>,
    pub new_issues: Vec<TopologyIssue>,
    pub resolved_issues: Vec<TopologyIssue>,
}

impl TopologyDiff {
    pub fn has_breaking(&self) -> bool {
        !self.removed_events.is_empty()
            || self.changed_events.iter().any(EventChange::has_breaking)
            || self.new_issues.iter().any(|i| i.severity == IssueSeverity::Error)
    }

    pub fn is_empty(&self) -> bool {
        self.added_events.is_empty()
            && self.removed_events.is_empty()
            && self.changed_events.is_empty()
            && self.new_issues.is_empty()
            && self.resolved_issues.is_empty()
    }

    pub fn to_markdown(&self) -> String {
        let mut out = String::from("## Event topology changes\n\n");
        if self.is_empty() {
            out.push_str("No event topology changes detected.\n");
            return out;
        }
        if self.has_breaking() {
            out.push_str("> ⚠️ **Contains breaking or error-level changes. Review carefully.**\n\n");
        }
        if !self.added_events.is_empty() {
            out.push_str("### Added events\n");
            for e in &self.added_events {
                out.push_str(&format!("- `{e}`\n"));
            }
            out.push('\n');
        }
        if !self.removed_events.is_empty() {
            out.push_str("### Removed events (breaking)\n");
            for e in &self.removed_events {
                out.push_str(&format!("- `{e}`\n"));
            }
            out.push('\n');
        }
        if !self.changed_events.is_empty() {
            out.push_str("### Changed events\n");
            for c in &self.changed_events {
                out.push_str(&format!("- `{}`\n", c.event_type));
                for s in &c.added_publishers {
                    out.push_str(&format!("  - publisher added: {s}\n"));
                }
                for s in &c.removed_publishers {
                    out.push_str(&format!("  - publisher removed: {s}\n"));
                }
                for s in &c.added_subscribers {
                    out.push_str(&format!("  - subscriber added: {s}\n"));
                }
                for s in &c.removed_subscribers {
                    out.push_str(&format!("  - subscriber removed: {s} (breaking)\n"));
                }
                for f in &c.field_changes {
                    let tag = if f.breaking { "BREAKING" } else { "ok" };
                    out.push_str(&format!("  - field [{tag}]: {}\n", f.detail));
                }
            }
            out.push('\n');
        }
        if !self.new_issues.is_empty() {
            out.push_str("### New diagnostics\n");
            for i in &self.new_issues {
                out.push_str(&format!("- **{}** `{}`: {}\n", severity_label(i.severity), i.code, i.message));
            }
            out.push('\n');
        }
        if !self.resolved_issues.is_empty() {
            out.push_str("### Resolved diagnostics\n");
            for i in &self.resolved_issues {
                out.push_str(&format!("- `{}`: {}\n", i.code, i.message));
            }
            out.push('\n');
        }
        out
    }
}

fn severity_label(s: IssueSeverity) -> &'static str {
    match s {
        IssueSeverity::Warning => "warning",
        IssueSeverity::Error => "error",
    }
}

fn load_events(dir: &Path) -> Result<BTreeMap<String, ParsedSchema>> {
    let mut map = BTreeMap::new();
    for file in discover_files(dir, &["json"])? {
        let schema = parse_schema_file(&file)?;
        if let Some(event_type) = schema.event_type.clone() {
            map.insert(event_type, schema);
        }
    }
    Ok(map)
}

fn set_diff(from: &[String], to: &[String]) -> (Vec<String>, Vec<String>) {
    let from: BTreeSet<&String> = from.iter().collect();
    let to: BTreeSet<&String> = to.iter().collect();
    (
        to.difference(&from).map(|s| (*s).clone()).collect(),
        from.difference(&to).map(|s| (*s).clone()).collect(),
    )
}

fn issue_key(i: &TopologyIssue) -> (String, String) {
    (i.code.clone(), i.message.clone())
}

pub fn compare(base_dir: &Path, head_dir: &Path, upcasters_dir: Option<&Path>) -> Result<TopologyDiff> {
    let base = load_events(base_dir).with_context(|| format!("Reading base schemas {}", base_dir.display()))?;
    let head = load_events(head_dir).with_context(|| format!("Reading head schemas {}", head_dir.display()))?;

    let added_events: Vec<String> = head.keys().filter(|k| !base.contains_key(*k)).cloned().collect();
    let removed_events: Vec<String> = base.keys().filter(|k| !head.contains_key(*k)).cloned().collect();

    let mut changed_events = Vec::new();
    for (event_type, new) in &head {
        let Some(old) = base.get(event_type) else { continue };
        let (added_publishers, removed_publishers) = set_diff(&old.published_by, &new.published_by);
        let (added_subscribers, removed_subscribers) = set_diff(&old.subscribed_by, &new.subscribed_by);
        let change = EventChange {
            event_type: event_type.clone(),
            added_publishers,
            removed_publishers,
            added_subscribers,
            removed_subscribers,
            field_changes: diff_schemas(old, new),
        };
        if !change.is_empty() {
            changed_events.push(change);
        }
    }

    let head_report = TopologyValidator::validate(head_dir, upcasters_dir)?;
    let base_report = TopologyValidator::validate(base_dir, upcasters_dir)?;
    let base_keys: BTreeSet<_> = base_report.issues.iter().map(issue_key).collect();
    let head_keys: BTreeSet<_> = head_report.issues.iter().map(issue_key).collect();
    let new_issues = head_report
        .issues
        .iter()
        .filter(|i| !base_keys.contains(&issue_key(i)))
        .cloned()
        .collect();
    let resolved_issues = base_report
        .issues
        .iter()
        .filter(|i| !head_keys.contains(&issue_key(i)))
        .cloned()
        .collect();

    Ok(TopologyDiff {
        added_events,
        removed_events,
        changed_events,
        new_issues,
        resolved_issues,
    })
}

/// Export the contents of `dir` as it exists at git `reference` into a fresh
/// temporary directory. The caller owns (and should delete) the result.
pub fn materialize_git_ref(dir: &Path, reference: &str) -> Result<PathBuf> {
    if reference.starts_with('-') {
        bail!("Invalid git reference: {reference}");
    }
    let listing = Command::new("git")
        .args(["ls-tree", "-r", "--name-only", reference, "--", "."])
        .current_dir(dir)
        .output()
        .context("Failed to run git")?;
    if !listing.status.success() {
        bail!(
            "git ls-tree failed for '{reference}': {}",
            String::from_utf8_lossy(&listing.stderr).trim()
        );
    }
    let target = std::env::temp_dir().join(format!(
        "clrinf-topology-base-{}-{}",
        std::process::id(),
        NEXT.fetch_add(1, Ordering::Relaxed)
    ));
    std::fs::create_dir_all(&target)?;
    for name in String::from_utf8_lossy(&listing.stdout).lines() {
        if name.split('/').any(|part| part == "..") {
            continue;
        }
        let blob = Command::new("git")
            .args(["show", &format!("{reference}:./{name}")])
            .current_dir(dir)
            .output()
            .context("Failed to run git show")?;
        if !blob.status.success() {
            continue;
        }
        let out_path = target.join(name);
        if let Some(parent) = out_path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::write(out_path, blob.stdout)?;
    }
    Ok(target)
}
