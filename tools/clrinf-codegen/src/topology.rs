use crate::schema::{parse_schema_file, ParsedSchema};
use crate::validator::discover_files;
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum IssueSeverity {
    Warning,
    Error,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TopologyIssue {
    pub severity: IssueSeverity,
    pub code: String,
    pub message: String,
    pub event_type: String,
    pub service: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ServiceNode {
    pub publishes: Vec<String>,
    pub subscribes: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventNode {
    pub title: String,
    pub version: Option<String>,
    pub publishers: Vec<String>,
    pub subscribers: Vec<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TopologyGraph {
    pub services: BTreeMap<String, ServiceNode>,
    pub events: BTreeMap<String, EventNode>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TopologyValidationReport {
    pub is_valid: bool,
    pub issues: Vec<TopologyIssue>,
    pub graph: TopologyGraph,
}

impl TopologyValidationReport {
    pub fn errors_count(&self) -> usize {
        self.issues.iter().filter(|i| i.severity == IssueSeverity::Error).count()
    }

    pub fn warnings_count(&self) -> usize {
        self.issues.iter().filter(|i| i.severity == IssueSeverity::Warning).count()
    }

    pub fn format_ascii_graph(&self) -> String {
        TopologyValidator::format_ascii_report(self)
    }

    pub fn format_diagnostics(&self) -> String {
        let mut out = String::new();
        for issue in &self.issues {
            let icon = match issue.severity {
                IssueSeverity::Warning => "⚠️  [WARN]",
                IssueSeverity::Error => "❌ [FAIL]",
            };
            out.push_str(&format!("{} [{}] {}\n", icon, issue.code, issue.message));
        }
        out
    }
}

pub struct TopologyValidator;

impl TopologyValidator {
    pub fn build_graph(schemas: &[ParsedSchema]) -> TopologyGraph {
        let mut services: BTreeMap<String, ServiceNode> = BTreeMap::new();
        let mut events: BTreeMap<String, EventNode> = BTreeMap::new();

        for schema in schemas {
            if let Some(ref event_type) = schema.event_type {
                let event_node = events.entry(event_type.clone()).or_insert_with(|| EventNode {
                    title: schema.title.clone(),
                    version: schema.version.clone(),
                    publishers: Vec::new(),
                    subscribers: Vec::new(),
                });

                for pub_srv in &schema.published_by {
                    if !event_node.publishers.contains(pub_srv) {
                        event_node.publishers.push(pub_srv.clone());
                    }
                    services
                        .entry(pub_srv.clone())
                        .or_default()
                        .publishes
                        .push(event_type.clone());
                }

                for sub_srv in &schema.subscribed_by {
                    if !event_node.subscribers.contains(sub_srv) {
                        event_node.subscribers.push(sub_srv.clone());
                    }
                    services
                        .entry(sub_srv.clone())
                        .or_default()
                        .subscribes
                        .push(event_type.clone());
                }
            }
        }

        // Deduplicate lists inside services
        for node in services.values_mut() {
            node.publishes.sort();
            node.publishes.dedup();
            node.subscribes.sort();
            node.subscribes.dedup();
        }

        TopologyGraph { services, events }
    }

    pub fn validate(
        schema_dir: &Path,
        upcasters_dir: Option<&Path>,
    ) -> Result<TopologyValidationReport> {
        let files = discover_files(schema_dir, &["json"])
            .with_context(|| format!("Reading schemas from {}", schema_dir.display()))?;

        let mut schemas = Vec::new();
        for file in &files {
            let s = parse_schema_file(file)?;
            schemas.push(s);
        }

        let graph = Self::build_graph(&schemas);
        let mut issues = Vec::new();

        // Check 1: Dead events (Warning)
        for (event_type, event_node) in &graph.events {
            if !event_node.publishers.is_empty() && event_node.subscribers.is_empty() {
                issues.push(TopologyIssue {
                    severity: IssueSeverity::Warning,
                    code: "TOPOLOGY_DEAD_EVENT".to_string(),
                    message: format!(
                        "Event '{}' is published by [{}] but has no active subscribers in the topology.",
                        event_type,
                        event_node.publishers.join(", ")
                    ),
                    event_type: event_type.clone(),
                    service: event_node.publishers.first().cloned(),
                });
            }
        }

        // Check 2: Orphan subscribers (Error)
        for (event_type, event_node) in &graph.events {
            if !event_node.subscribers.is_empty() && event_node.publishers.is_empty() {
                issues.push(TopologyIssue {
                    severity: IssueSeverity::Error,
                    code: "TOPOLOGY_ORPHAN_SUBSCRIBER".to_string(),
                    message: format!(
                        "Event '{}' is subscribed by [{}] but no service publishes it.",
                        event_type,
                        event_node.subscribers.join(", ")
                    ),
                    event_type: event_type.clone(),
                    service: event_node.subscribers.first().cloned(),
                });
            }
        }

        // Check 3: Upcaster chain validation between versions (Error)
        let upcaster_pairs = if let Some(up_dir) = upcasters_dir {
            Self::discover_upcaster_pairs(up_dir)?
        } else {
            BTreeSet::new()
        };

        // Group events by base family (everything before .vX)
        let mut event_families: BTreeMap<String, Vec<&EventNode>> = BTreeMap::new();
        for event_node in graph.events.values() {
            let base_name = Self::strip_version(&event_node.title);
            event_families.entry(base_name).or_default().push(event_node);
        }

        for (family, versions) in event_families {
            if versions.len() > 1 {
                // Multiple versions of the same event exist in the ecosystem
                for v1 in &versions {
                    for v2 in &versions {
                        if v1.title != v2.title && !v1.publishers.is_empty() && !v2.subscribers.is_empty() {
                            let pair_key = (v1.title.clone(), v2.title.clone());
                            if !upcaster_pairs.contains(&pair_key) {
                                issues.push(TopologyIssue {
                                    severity: IssueSeverity::Error,
                                    code: "TOPOLOGY_UPCASTER_MISSING".to_string(),
                                    message: format!(
                                        "Version gap in event '{}': published as '{}' by [{}] while [{}] subscribes to '{}', but no upcaster chain ('{}' -> '{}') exists.",
                                        family,
                                        v1.title,
                                        v1.publishers.join(", "),
                                        v2.subscribers.join(", "),
                                        v2.title,
                                        v1.title,
                                        v2.title
                                    ),
                                    event_type: family.clone(),
                                    service: v1.publishers.first().cloned(),
                                });
                            }
                        }
                    }
                }
            }
        }

        let is_valid = !issues.iter().any(|i| i.severity == IssueSeverity::Error);

        Ok(TopologyValidationReport {
            is_valid,
            issues,
            graph,
        })
    }

    fn strip_version(title: &str) -> String {
        if let Some(pos) = title.rfind('V') {
            if pos > 0 && title[pos + 1..].chars().all(|c| c.is_ascii_digit()) {
                return title[..pos].to_string();
            }
        }
        title.to_string()
    }

    fn discover_upcaster_pairs(upcasters_dir: &Path) -> Result<BTreeSet<(String, String)>> {
        let mut pairs = BTreeSet::new();
        if !upcasters_dir.is_dir() {
            return Ok(pairs);
        }

        let files = discover_files(upcasters_dir, &["yaml", "yml"])?;
        for file in files {
            let content = std::fs::read_to_string(&file)?;
            if let Ok(yaml) = serde_yaml::from_str::<serde_yaml::Value>(&content) {
                if let (Some(from), Some(to)) = (
                    yaml.get("from").and_then(|v| v.as_str()),
                    yaml.get("to").and_then(|v| v.as_str()),
                ) {
                    pairs.insert((from.to_string(), to.to_string()));
                }
            }
        }

        Ok(pairs)
    }

    pub fn format_ascii_report(report: &TopologyValidationReport) -> String {
        let mut out = String::new();
        out.push_str("🌐 Cross-Service Pub/Sub Topology Map:\n\n");

        if report.graph.services.is_empty() {
            out.push_str("  (No services declared in schema metadata with x-published-by / x-subscribed-by)\n\n");
        } else {
            for (service, node) in &report.graph.services {
                out.push_str(&format!("  📦 Service: \x1b[1m{}\x1b[0m\n", service));
                if node.publishes.is_empty() {
                    out.push_str("     Publishes: (none)\n");
                } else {
                    out.push_str("     Publishes:\n");
                    for p in &node.publishes {
                        out.push_str(&format!("       ➔ \x1b[32m{}\x1b[0m\n", p));
                    }
                }
                if node.subscribes.is_empty() {
                    out.push_str("     Subscribes: (none)\n");
                } else {
                    out.push_str("     Subscribes:\n");
                    for s in &node.subscribes {
                        out.push_str(&format!("       📥 \x1b[34m{}\x1b[0m\n", s));
                    }
                }
                out.push('\n');
            }
        }

        out.push_str("──────────────────────────────────────────\n");
        out.push_str(&format!("Diagnostics: {} issue(s) detected.\n\n", report.issues.len()));

        for issue in &report.issues {
            let icon = match issue.severity {
                IssueSeverity::Warning => "⚠️  [WARN]",
                IssueSeverity::Error => "❌ [FAIL]",
            };
            out.push_str(&format!("{} [{}] {}\n", icon, issue.code, issue.message));
        }

        if report.is_valid {
            out.push_str("\n✅ Topology verification passed: No orphan subscribers or missing upcaster chains.\n");
        } else {
            out.push_str("\n❌ Topology verification failed with fatal cross-service mismatches.\n");
        }

        out
    }
}
