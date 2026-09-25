use anyhow::{bail, ensure, Context, Result};
use heck::{ToSnakeCase, ToUpperCamelCase};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FieldModel {
    pub name: String,
    pub rust_name: String,
    pub csharp_name: String,
    pub ts_name: String,
    pub elixir_name: String,
    pub rust_type: String,
    pub csharp_type: String,
    pub ts_type: String,
    pub elixir_type: String,
    pub description: Option<String>,
    pub is_required: bool,
    pub is_optional: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParsedSchema {
    pub title: String,
    pub description: Option<String>,
    pub domain: String,
    pub file_stem: String,
    pub event_type: Option<String>,
    pub command_type: Option<String>,
    pub query_type: Option<String>,
    pub published_by: Vec<String>,
    pub subscribed_by: Vec<String>,
    pub version: Option<String>,
    pub fields: Vec<FieldModel>,
}

pub fn parse_schema_file(path: &Path) -> Result<ParsedSchema> {
    let content = std::fs::read_to_string(path)
        .with_context(|| format!("Failed to read schema file: {}", path.display()))?;
    let json: Value = serde_json::from_str(&content)
        .with_context(|| format!("Invalid JSON in schema file: {}", path.display()))?;
    check_supported(&json, "$")
        .with_context(|| format!("Unsupported model schema: {}", path.display()))?;
    ensure!(
        json.get("type").and_then(Value::as_str) == Some("object"),
        "Model root must have type object: {}",
        path.display()
    );

    let file_stem = path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("Schema")
        .replace(".schema", "")
        .replace('.', "_")
        .to_snake_case();

    let title = json
        .get("title")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string())
        .unwrap_or_else(|| file_stem.to_upper_camel_case());
    ensure!(
        valid_identifier(&title) && valid_identifier(&file_stem),
        "Schema title and file stem must be identifiers: {}",
        path.display()
    );
    ensure!(title.chars().next().is_some_and(char::is_uppercase)
        && !matches!(file_stem.as_str(), "mod" | "index" | "self" | "super" | "crate")
        && title != "ICloudEvent",
        "Schema title must start uppercase and names must not collide with generated support files: {}", path.display());

    let description = json
        .get("description")
        .and_then(|v| v.as_str())
        .map(clean_description);

    let event_type = json
        .get("x-event-type")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());

    let command_type = json
        .get("x-command-type")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());

    let query_type = json
        .get("x-query-type")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());
    ensure!(
        [&event_type, &command_type, &query_type]
            .into_iter()
            .filter(|value| value.is_some())
            .count()
            <= 1,
        "Schema must declare at most one event, command or query type"
    );

    let domain = match json.get("x-domain") {
        Some(value) => value
            .as_str()
            .context("x-domain must be a string")?
            .to_owned(),
        None => "common".to_string(),
    };
    ensure!(
        valid_identifier(&domain),
        "x-domain must be an identifier, not a path: {}",
        path.display()
    );
    for name in [&event_type, &command_type, &query_type]
        .into_iter()
        .flatten()
    {
        ensure!(
            !name.is_empty()
                && name
                    .chars()
                    .all(|c| c.is_ascii_alphanumeric() || "._-".contains(c)),
            "Message type contains unsupported characters: {name}"
        );
    }

    let required_fields: Vec<String> = json
        .get("required")
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|v| v.as_str().map(|s| s.to_string()))
                .collect()
        })
        .unwrap_or_default();

    let mut fields = Vec::new();

    if let Some(properties) = json.get("properties").and_then(|v| v.as_object()) {
        for (prop_name, prop_val) in properties {
            ensure!(!prop_name.is_empty() && prop_name.chars().all(|c| c.is_alphanumeric() || "_-.".contains(c)),
                "Unsupported wire property name: {prop_name:?}; use letters, digits, underscores, hyphens or dots");
            let is_required = required_fields.contains(prop_name);
            let prop_desc = prop_val
                .get("description")
                .and_then(|v| v.as_str())
                .map(clean_description);

            let (rust_type, csharp_type, ts_type, elixir_type) = map_types(prop_val)?;

            let rust_name = sanitize_rust_keyword(&prop_name.to_snake_case());
            let csharp_name = prop_name.to_upper_camel_case();
            let ts_name = serde_json::to_string(prop_name)?;
            let elixir_name = prop_name.to_snake_case();
            ensure!(
                valid_identifier(&csharp_name) && valid_identifier(&elixir_name),
                "Property cannot be represented as an identifier: {prop_name}"
            );
            ensure!(
                !matches!(
                    elixir_name.as_str(),
                    "self" | "super" | "crate" | "true" | "false" | "nil" | "__struct__"
                ) && csharp_name != title
                    && !matches!(
                        csharp_name.as_str(),
                        "Clone"
                            | "EqualityContract"
                            | "PrintMembers"
                            | "ToString"
                            | "Equals"
                            | "GetHashCode"
                            | "Deconstruct"
                    )
                    && !(event_type.is_some()
                        && matches!(csharp_name.as_str(), "EventType" | "EventTypeName"))
                    && !(command_type.is_some()
                        && matches!(csharp_name.as_str(), "CommandType" | "CommandTypeName"))
                    && !(query_type.is_some()
                        && matches!(csharp_name.as_str(), "QueryType" | "QueryTypeName")),
                "Reserved property name: {prop_name}"
            );
            ensure!(
                !fields.iter().any(|f: &FieldModel| f.rust_name == rust_name
                    || f.csharp_name == csharp_name
                    || f.elixir_name == elixir_name),
                "Properties collide after normalization: {prop_name}"
            );

            fields.push(FieldModel {
                name: prop_name.clone(),
                rust_name,
                csharp_name,
                ts_name,
                elixir_name,
                rust_type,
                csharp_type,
                ts_type,
                elixir_type,
                description: prop_desc,
                is_required,
                is_optional: !is_required,
            });
        }
    }

    let published_by: Vec<String> = match json.get("x-published-by") {
        Some(Value::Array(arr)) => arr.iter().filter_map(|v| v.as_str().map(|s| s.to_string())).collect(),
        Some(Value::String(s)) => vec![s.clone()],
        _ => Vec::new(),
    };

    let subscribed_by: Vec<String> = match json.get("x-subscribed-by") {
        Some(Value::Array(arr)) => arr.iter().filter_map(|v| v.as_str().map(|s| s.to_string())).collect(),
        Some(Value::String(s)) => vec![s.clone()],
        _ => Vec::new(),
    };

    let version: Option<String> = json
        .get("x-version")
        .and_then(Value::as_str)
        .map(|s| s.to_string())
        .or_else(|| {
            event_type.as_ref().and_then(|et| {
                et.split('.').last().filter(|part| part.starts_with('v') && part[1..].chars().all(|c| c.is_ascii_digit())).map(|s| s.to_string())
            })
        });

    // Sort fields: required first, then optional (retaining stable name order within each group)
    fields.sort_by_key(|f| if f.is_required { 0 } else { 1 });

    Ok(ParsedSchema {
        title,
        description,
        domain,
        file_stem,
        event_type,
        command_type,
        query_type,
        published_by,
        subscribed_by,
        version,
        fields,
    })
}

fn map_types(prop_val: &Value) -> Result<(String, String, String, String)> {
    let prop_type = prop_val
        .get("type")
        .and_then(|v| v.as_str())
        .context("Property requires an explicit type")?;
    let prop_format = prop_val.get("format").and_then(|v| v.as_str());

    let (default_rust, default_csharp, default_ts, default_elixir) = match prop_type {
        "integer" => match prop_format {
            Some("int32") | Some("int") => (
                "i32".to_string(),
                "int".to_string(),
                "number".to_string(),
                "integer()".to_string(),
            ),
            Some("uint32") | Some("uint") => (
                "u32".to_string(),
                "uint".to_string(),
                "number".to_string(),
                "non_neg_integer()".to_string(),
            ),
            Some("int64") => (
                "i64".to_string(),
                "long".to_string(),
                "number".to_string(),
                "integer()".to_string(),
            ),
            Some("uint64") => (
                "u64".to_string(),
                "ulong".to_string(),
                "number".to_string(),
                "non_neg_integer()".to_string(),
            ),
            _ => (
                "i64".to_string(),
                "long".to_string(),
                "number".to_string(),
                "integer()".to_string(),
            ),
        },
        "number" => match prop_format {
            Some("float") => (
                "f32".to_string(),
                "float".to_string(),
                "number".to_string(),
                "float()".to_string(),
            ),
            Some("decimal") => (
                "f64".to_string(),
                "decimal".to_string(),
                "number".to_string(),
                "float()".to_string(),
            ),
            _ => (
                "f64".to_string(),
                "double".to_string(),
                "number".to_string(),
                "float()".to_string(),
            ),
        },
        "boolean" => (
            "bool".to_string(),
            "bool".to_string(),
            "boolean".to_string(),
            "boolean()".to_string(),
        ),
        "array" => {
            let (r, c, t, e) = map_types(prop_val.get("items").context("Array requires items")?)?;
            (
                format!("Vec<{r}>"),
                format!("IReadOnlyList<{c}>"),
                format!("ReadonlyArray<{t}>"),
                format!("list({e})"),
            )
        }
        "object" => (
            "serde_json::Value".to_string(),
            "object".to_string(),
            "Record<string, unknown>".to_string(),
            "map()".to_string(),
        ),
        "string" => match prop_format {
            Some("date-time") => (
                "String".to_string(),
                "System.DateTimeOffset".to_string(),
                "string".to_string(),
                "String.t()".to_string(),
            ),
            Some("uuid") => (
                "String".to_string(),
                "System.Guid".to_string(),
                "string".to_string(),
                "String.t()".to_string(),
            ),
            _ => (
                "String".to_string(),
                "string".to_string(),
                "string".to_string(),
                "String.t()".to_string(),
            ),
        },
        other => bail!("Unsupported type {other}"),
    };

    Ok((default_rust, default_csharp, default_ts, default_elixir))
}

fn valid_identifier(name: &str) -> bool {
    let mut chars = name.chars();
    chars.next().is_some_and(|c| c.is_alphabetic() || c == '_')
        && chars.all(|c| c.is_alphanumeric() || c == '_')
}

fn clean_description(value: &str) -> String {
    value
        .replace(['\r', '\n'], " ")
        .replace("*/", "* /")
        .replace('\\', "\\\\")
        .replace("\"\"\"", "\\\"\\\"\\\"")
        .replace("#{", "\\#{")
}

/// Generated DTOs represent shapes; JSON Schema remains the runtime validation authority.
fn check_supported(schema: &Value, location: &str) -> Result<()> {
    let object = schema.as_object().with_context(|| {
        format!("{location}: boolean schemas are not supported by model generation")
    })?;
    for key in object.keys() {
        ensure!(matches!(key.as_str(), "$schema" | "$id" | "title" | "description" | "type"
            | "properties" | "required" | "items" | "additionalProperties" | "format" | "minimum"
            | "maximum" | "exclusiveMinimum" | "exclusiveMaximum" | "multipleOf" | "minLength"
            | "maxLength" | "pattern" | "minItems" | "maxItems" | "uniqueItems" | "minProperties"
            | "maxProperties" | "enum" | "const" | "default" | "examples" | "deprecated"
            | "readOnly" | "writeOnly" | "$comment" | "x-domain" | "x-event-type" | "x-command-type"
            | "x-query-type" | "x-published-by" | "x-subscribed-by" | "x-version"),
            "{location}: unsupported keyword '{key}' (references, composition, unions and custom code type overrides are not supported)");
    }
    let kind = schema
        .get("type")
        .and_then(Value::as_str)
        .with_context(|| {
            format!(
                "{location}: explicit scalar type required; nullable/type unions are unsupported"
            )
        })?;
    ensure!(
        matches!(
            kind,
            "object" | "array" | "string" | "integer" | "number" | "boolean"
        ),
        "{location}: unsupported type '{kind}'"
    );
    if let Some(properties) = schema.get("properties") {
        for (name, value) in properties
            .as_object()
            .context("properties must be an object")?
        {
            check_supported(value, &format!("{location}.{name}"))?;
        }
    }
    if let Some(required) = schema.get("required") {
        for name in required.as_array().context("required must be an array")? {
            let name = name.as_str().context("required names must be strings")?;
            ensure!(
                schema.get("properties").and_then(|p| p.get(name)).is_some(),
                "{location}: required field '{name}' is not declared in properties"
            );
        }
    }
    if kind == "array" {
        check_supported(
            schema
                .get("items")
                .context("array requires homogeneous items")?,
            &format!("{location}[]"),
        )?;
    }
    if let Some(additional) = schema.get("additionalProperties").filter(|v| v.is_object()) {
        check_supported(additional, &format!("{location}.*"))?;
    }
    Ok(())
}

fn sanitize_rust_keyword(name: &str) -> String {
    match name {
        "type" | "match" | "enum" | "struct" | "fn" | "let" | "ref" | "move" | "pub" | "crate"
        | "mod" | "use" | "self" | "super" | "where" | "for" | "while" | "loop" | "if" | "else"
        | "return" | "break" | "continue" | "as" | "const" | "trait" | "impl" | "dyn" | "yield"
        | "async" | "await" | "try" | "in" | "extern" | "unsafe" | "static" | "abstract"
        | "become" | "box" | "do" | "final" | "macro" | "override" | "priv" | "typeof"
        | "unsized" | "virtual" | "gen" => format!("r#{}", name),
        _ => name.to_string(),
    }
}
