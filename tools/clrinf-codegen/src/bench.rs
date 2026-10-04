//! Deterministic context-cost benchmark.
//!
//! Measures (not guesses) how much context an agent needs for an event change
//! with and without clrinf's targeted slice, and the fixed per-session cost of
//! the MCP tool schemas for each profile. Token counts use the common
//! ~4 characters/token estimate; ratios are what matter, not absolute values.
//! End-to-end agent A/B runs are described in docs/benchmarks/README.md.

use crate::impact;
use crate::mcp::{McpProfile, McpServer};
use crate::rules::{render_agent_block, Rules};
use crate::scan::collect_sources;
use crate::validator::discover_files;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

pub fn estimate_tokens(chars: usize) -> usize {
    (chars + 3) / 4
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpCost {
    pub profile: String,
    pub tools: usize,
    pub tokens: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextBench {
    pub event_type: String,
    pub baseline_files: usize,
    pub baseline_tokens: usize,
    pub slice_files: usize,
    pub slice_tokens: usize,
    pub reduction_pct: f64,
    pub instruction_tokens: usize,
    pub mcp: Vec<McpCost>,
    pub method: String,
}

impl ContextBench {
    pub fn format_text(&self) -> String {
        let mut out = format!(
            "📏 Context cost for event '{}' (estimate, ~4 chars/token)\n\n",
            self.event_type
        );
        out.push_str(&format!(
            "  Baseline (explore sources + schemas): {:>8} tokens in {} files\n",
            self.baseline_tokens, self.baseline_files
        ));
        out.push_str(&format!(
            "  clrinf slice (schema + hand-written affected files + rules): {:>6} tokens in {} files\n",
            self.slice_tokens, self.slice_files
        ));
        out.push_str(&format!("  Reduction: {:.1}%\n\n", self.reduction_pct));
        out.push_str("  Fixed MCP tool-schema cost per session:\n");
        for m in &self.mcp {
            out.push_str(&format!("    profile {:<7} {:>2} tools {:>6} tokens\n", m.profile, m.tools, m.tokens));
        }
        out.push_str(&format!("\n  Method: {}\n", self.method));
        out
    }
}

fn mcp_costs() -> Vec<McpCost> {
    [McpProfile::Full, McpProfile::Events, McpProfile::Lean]
        .into_iter()
        .map(|profile| {
            let server = McpServer::new().with_profile(profile);
            let response = server
                .handle_request(&json!({"jsonrpc": "2.0", "id": 1, "method": "tools/list"}))
                .ok()
                .flatten()
                .unwrap_or_default();
            let tools = response["result"]["tools"].as_array().cloned().unwrap_or_default();
            let chars = serde_json::to_string(&tools).map(|s| s.len()).unwrap_or(0);
            McpCost { profile: profile.name().into(), tools: tools.len(), tokens: estimate_tokens(chars) }
        })
        .collect()
}

pub fn context(schema_dir: &Path, event: &str, src_roots: &[PathBuf], rules: Option<&Rules>) -> Result<ContextBench> {
    let report = impact::analyze(schema_dir, event, src_roots)?;

    let mut baseline_chars = 0usize;
    let mut baseline_files = 0usize;
    for root in src_roots {
        for f in collect_sources(root)? {
            baseline_chars += f.content.len();
            baseline_files += 1;
        }
    }
    for schema in discover_files(schema_dir, &["json"])? {
        baseline_chars += std::fs::read_to_string(&schema).map(|s| s.len()).unwrap_or(0);
        baseline_files += 1;
    }

    let instruction_tokens = estimate_tokens(render_agent_block(rules).len());
    let mut slice_chars = std::fs::read_to_string(&report.schema_file).map(|s| s.len()).unwrap_or(0);
    let mut slice_paths = BTreeSet::new();
    slice_paths.insert(report.schema_file.clone());
    for f in report.affected_files.iter().filter(|f| !f.generated) {
        if slice_paths.insert(f.path.clone()) {
            slice_chars += std::fs::read_to_string(&f.path).map(|s| s.len()).unwrap_or(0);
        }
    }
    let baseline_tokens = estimate_tokens(baseline_chars);
    let slice_tokens = estimate_tokens(slice_chars) + instruction_tokens;
    let reduction_pct = if baseline_tokens == 0 {
        0.0
    } else {
        (1.0 - slice_tokens as f64 / baseline_tokens as f64) * 100.0
    };

    Ok(ContextBench {
        event_type: report.event_type,
        baseline_files,
        baseline_tokens,
        slice_files: slice_paths.len(),
        slice_tokens,
        reduction_pct,
        instruction_tokens,
        mcp: mcp_costs(),
        method: "baseline = every source file under --src plus all schemas; slice = event schema + hand-written files that reference the event + the generated agent rules block (generated shells are not read).".into(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn token_estimate_rounds_up() {
        assert_eq!(estimate_tokens(0), 0);
        assert_eq!(estimate_tokens(1), 1);
        assert_eq!(estimate_tokens(8), 2);
    }

    #[test]
    fn lean_profile_costs_less_than_full() {
        let costs = mcp_costs();
        let by = |n: &str| costs.iter().find(|c| c.profile == n).unwrap().clone();
        assert!(by("lean").tokens < by("events").tokens && by("events").tokens < by("full").tokens);
        assert!(by("lean").tools >= 2);
    }
}
