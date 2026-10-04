//! Event-first scaffolding: register a service as publisher/subscriber of an
//! event by editing the schema's `x-published-by` / `x-subscribed-by`
//! declaration in place (text-level, so formatting and key order survive).
//! The caller then regenerates pub/sub shells so the AI agent only has to fill
//! in the handler body.

use crate::validator::discover_files;
use anyhow::{bail, ensure, Context, Result};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Role {
    Publisher,
    Subscriber,
}

impl Role {
    fn key(self) -> &'static str {
        match self {
            Role::Publisher => "x-published-by",
            Role::Subscriber => "x-subscribed-by",
        }
    }
}

#[derive(Debug)]
pub struct ScaffoldOutcome {
    pub schema_file: PathBuf,
    pub title: String,
    pub changed: bool,
}

fn find_event_schema(schema_dir: &Path, event_type: &str) -> Result<(PathBuf, String, String)> {
    for file in discover_files(schema_dir, &["json"])? {
        let text = std::fs::read_to_string(&file)?;
        let Ok(json) = serde_json::from_str::<serde_json::Value>(&text) else { continue };
        if json.get("x-event-type").and_then(|v| v.as_str()) == Some(event_type) {
            let title = json
                .get("title")
                .and_then(|v| v.as_str())
                .unwrap_or_default()
                .to_string();
            return Ok((file, text, title));
        }
    }
    bail!("No schema declares x-event-type '{event_type}' in {}", schema_dir.display())
}

fn valid_service_name(name: &str) -> bool {
    !name.is_empty() && name.chars().all(|c| c.is_ascii_alphanumeric() || "-_.".contains(c))
}

/// Return the edited schema text, or `None` when the service is already registered.
pub fn register_in_text(text: &str, role: Role, service: &str) -> Result<Option<String>> {
    ensure!(valid_service_name(service), "Invalid service name: {service:?}");
    let key = format!("\"{}\"", role.key());

    if let Some(key_pos) = text.find(&key) {
        let after_key = key_pos + key.len();
        let rest = &text[after_key..];
        let colon = rest.find(':').context("Malformed schema: missing ':' after key")?;
        let value_start = after_key + colon + 1;
        let trimmed_offset = text[value_start..].len() - text[value_start..].trim_start().len();
        let begin = value_start + trimmed_offset;
        let (end, mut services): (usize, Vec<String>) = match text[begin..].chars().next() {
            Some('[') => {
                let close = text[begin..].find(']').context("Malformed schema: unterminated array")? + begin;
                let list: Vec<String> = serde_json::from_str(&text[begin..=close])
                    .context("x-published-by/x-subscribed-by must be an array of strings")?;
                (close + 1, list)
            }
            Some('"') => {
                let (s, len) = {
                    let mut de = serde_json::Deserializer::from_str(&text[begin..]).into_iter::<String>();
                    let value = de.next().context("Malformed string value")??;
                    (value, de.byte_offset())
                };
                (begin + len, vec![s])
            }
            _ => bail!("Unsupported value for {key}"),
        };
        if services.iter().any(|s| s == service) {
            return Ok(None);
        }
        services.push(service.to_string());
        let rendered = format!(
            "[{}]",
            services
                .iter()
                .map(|s| serde_json::to_string(s).unwrap())
                .collect::<Vec<_>>()
                .join(", ")
        );
        return Ok(Some(format!("{}{}{}", &text[..begin], rendered, &text[end..])));
    }

    // Key missing: insert a new line after the x-event-type line.
    let marker = "\"x-event-type\"";
    let pos = text.find(marker).context("Schema has no x-event-type")?;
    let line_start = text[..pos].rfind('\n').map_or(0, |i| i + 1);
    let indent: String = text[line_start..pos].chars().take_while(|c| c.is_whitespace()).collect();
    let line_end = text[pos..].find('\n').map_or(text.len(), |i| pos + i);
    let line = &text[pos..line_end];
    ensure!(
        line.trim_end().trim_end_matches('\r').ends_with(','),
        "x-event-type must not be the last property in the schema; add the declaration manually"
    );
    let newline = if text[..line_end].ends_with('\r') { "\r\n" } else { "\n" };
    let insert = format!(
        "{newline}{indent}\"{}\": [{}],",
        role.key(),
        serde_json::to_string(service)?
    );
    let insert_at = if text[..line_end].ends_with('\r') { line_end - 1 } else { line_end };
    Ok(Some(format!("{}{}{}", &text[..insert_at], insert, &text[insert_at..])))
}

pub fn register(schema_dir: &Path, event_type: &str, role: Role, service: &str) -> Result<ScaffoldOutcome> {
    let (file, text, title) = find_event_schema(schema_dir, event_type)?;
    match register_in_text(&text, role, service)? {
        Some(updated) => {
            serde_json::from_str::<serde_json::Value>(&updated)
                .context("Edit would produce invalid JSON; aborting without writing")?;
            std::fs::write(&file, updated)?;
            Ok(ScaffoldOutcome { schema_file: file, title, changed: true })
        }
        None => Ok(ScaffoldOutcome { schema_file: file, title, changed: false }),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const BASE: &str = "{\n  \"title\": \"E\",\n  \"x-event-type\": \"a.b.v1\",\n  \"x-published-by\": [\"p\"],\n  \"type\": \"object\"\n}";

    #[test]
    fn appends_to_existing_array() {
        let out = register_in_text(BASE, Role::Publisher, "q").unwrap().unwrap();
        assert!(out.contains("[\"p\", \"q\"]"));
        serde_json::from_str::<serde_json::Value>(&out).unwrap();
    }

    #[test]
    fn is_idempotent() {
        assert!(register_in_text(BASE, Role::Publisher, "p").unwrap().is_none());
    }

    #[test]
    fn inserts_missing_key_after_event_type() {
        let out = register_in_text(BASE, Role::Subscriber, "sub").unwrap().unwrap();
        assert!(out.contains("  \"x-subscribed-by\": [\"sub\"],"));
        serde_json::from_str::<serde_json::Value>(&out).unwrap();
    }

    #[test]
    fn converts_string_value() {
        let text = "{\"x-event-type\": \"a.v1\", \"x-subscribed-by\": \"one\"}";
        let out = register_in_text(text, Role::Subscriber, "two").unwrap().unwrap();
        assert!(out.contains("[\"one\", \"two\"]"));
        serde_json::from_str::<serde_json::Value>(&out).unwrap();
    }

    #[test]
    fn rejects_bad_service_names() {
        assert!(register_in_text(BASE, Role::Publisher, "bad\"name").is_err());
    }
}
