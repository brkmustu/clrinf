//! Schema evolution compatibility analysis.
//!
//! Compares two versions of an event schema from the perspective of an
//! upcaster that must turn an *old* payload into the *new* shape, and
//! classifies every difference as breaking or non-breaking.

use crate::schema::{FieldModel, ParsedSchema};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ChangeKind {
    FieldAdded,
    FieldRemoved,
    TypeChanged,
    BecameRequired,
    BecameOptional,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SchemaChange {
    pub kind: ChangeKind,
    pub field: String,
    pub breaking: bool,
    pub detail: String,
}

fn type_signature(f: &FieldModel) -> String {
    format!("{}|{}|{}", f.rust_type, f.csharp_type, f.ts_type)
}

/// Diff `old` -> `new`. Breaking means an upcaster must supply or convert data
/// that is not derivable from the old payload alone.
pub fn diff_schemas(old: &ParsedSchema, new: &ParsedSchema) -> Vec<SchemaChange> {
    let old_fields: BTreeMap<&str, &FieldModel> = old.fields.iter().map(|f| (f.name.as_str(), f)).collect();
    let new_fields: BTreeMap<&str, &FieldModel> = new.fields.iter().map(|f| (f.name.as_str(), f)).collect();
    let mut changes = Vec::new();

    for (name, nf) in &new_fields {
        match old_fields.get(name) {
            None => changes.push(SchemaChange {
                kind: ChangeKind::FieldAdded,
                field: (*name).to_string(),
                breaking: nf.is_required,
                detail: if nf.is_required {
                    format!("required field '{}' ({}) added; old payloads do not carry it", name, nf.ts_type)
                } else {
                    format!("optional field '{}' ({}) added", name, nf.ts_type)
                },
            }),
            Some(of) => {
                if type_signature(of) != type_signature(nf) {
                    changes.push(SchemaChange {
                        kind: ChangeKind::TypeChanged,
                        field: (*name).to_string(),
                        breaking: true,
                        detail: format!("field '{}' type changed {} -> {}", name, of.ts_type, nf.ts_type),
                    });
                }
                if !of.is_required && nf.is_required {
                    changes.push(SchemaChange {
                        kind: ChangeKind::BecameRequired,
                        field: (*name).to_string(),
                        breaking: true,
                        detail: format!("field '{}' optional -> required", name),
                    });
                } else if of.is_required && !nf.is_required {
                    changes.push(SchemaChange {
                        kind: ChangeKind::BecameOptional,
                        field: (*name).to_string(),
                        breaking: false,
                        detail: format!("field '{}' required -> optional", name),
                    });
                }
            }
        }
    }

    for (name, of) in &old_fields {
        if !new_fields.contains_key(name) {
            changes.push(SchemaChange {
                kind: ChangeKind::FieldRemoved,
                field: (*name).to_string(),
                breaking: of.is_required,
                detail: if of.is_required {
                    format!("required field '{}' removed; consumers of the new shape lose data", name)
                } else {
                    format!("optional field '{}' removed", name)
                },
            });
        }
    }

    changes
}

pub fn has_breaking(changes: &[SchemaChange]) -> bool {
    changes.iter().any(|c| c.breaking)
}

/// Human readable one-line summary used inside diagnostics.
pub fn summarize(changes: &[SchemaChange]) -> String {
    if changes.is_empty() {
        return "no structural field differences".to_string();
    }
    let breaking: Vec<&str> = changes.iter().filter(|c| c.breaking).map(|c| c.detail.as_str()).collect();
    if breaking.is_empty() {
        format!("{} non-breaking change(s) only", changes.len())
    } else {
        format!("{} breaking change(s): {}", breaking.len(), breaking.join("; "))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn field(name: &str, ty: &str, required: bool) -> FieldModel {
        FieldModel {
            name: name.into(),
            rust_name: name.into(),
            csharp_name: name.into(),
            ts_name: name.into(),
            elixir_name: name.into(),
            rust_type: ty.into(),
            csharp_type: ty.into(),
            ts_type: ty.into(),
            elixir_type: ty.into(),
            description: None,
            is_required: required,
            is_optional: !required,
        }
    }

    fn schema(fields: Vec<FieldModel>) -> ParsedSchema {
        ParsedSchema {
            title: "E".into(),
            description: None,
            domain: "d".into(),
            file_stem: "e".into(),
            event_type: Some("d.e.v1".into()),
            command_type: None,
            query_type: None,
            published_by: vec![],
            subscribed_by: vec![],
            version: Some("v1".into()),
            fields,
        }
    }

    #[test]
    fn additive_optional_is_not_breaking() {
        let old = schema(vec![field("id", "string", true)]);
        let new = schema(vec![field("id", "string", true), field("note", "string", false)]);
        let changes = diff_schemas(&old, &new);
        assert_eq!(changes.len(), 1);
        assert!(!has_breaking(&changes));
    }

    #[test]
    fn required_addition_type_change_and_removal_are_breaking() {
        let old = schema(vec![field("id", "string", true), field("qty", "int", true)]);
        let new = schema(vec![field("id", "number", true), field("sku", "string", true)]);
        let changes = diff_schemas(&old, &new);
        assert!(changes.iter().any(|c| c.kind == ChangeKind::TypeChanged && c.breaking));
        assert!(changes.iter().any(|c| c.kind == ChangeKind::FieldAdded && c.breaking));
        assert!(changes.iter().any(|c| c.kind == ChangeKind::FieldRemoved && c.breaking));
        assert!(summarize(&changes).contains("breaking"));
    }
}
