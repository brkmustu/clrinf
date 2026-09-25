use anyhow::{ensure, Context, Result};
use heck::{ToKebabCase, ToSnakeCase, ToUpperCamelCase};
use serde::Deserialize;
use serde_json::{json, Value};
use std::collections::BTreeSet;
use std::path::Path;
use tera::{Context as TeraContext, Tera};

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Definition {
    from: String,
    to: String,
    #[serde(default, rename = "description")]
    _description: Option<String>,
    mappings: Vec<Mapping>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Mapping {
    target_field: String,
    #[serde(default)]
    source_field: Option<String>,
    #[serde(default, deserialize_with = "explicit_default")]
    default_value: Option<Value>,
    r#type: String,
}

fn explicit_default<'de, D: serde::Deserializer<'de>>(
    deserializer: D,
) -> std::result::Result<Option<Value>, D::Error> {
    Value::deserialize(deserializer).map(Some)
}

fn identifier(value: &str) -> bool {
    let mut chars = value.chars();
    chars
        .next()
        .is_some_and(|c| c.is_ascii_alphabetic() || c == '_')
        && chars.all(|c| c.is_ascii_alphanumeric() || c == '_')
}

fn event_identifier(value: &str) -> bool {
    !value.is_empty() && value.split('.').all(identifier)
}

fn matches_type(value: &Value, kind: &str) -> bool {
    match kind {
        "boolean" => value.is_boolean(),
        "string" => value.is_string(),
        "integer" => value.is_i64() || value.is_u64(),
        "number" => value.is_number(),
        "array" => value.is_array(),
        "object" => value.is_object(),
        "null" => value.is_null(),
        _ => false,
    }
}

// JavaScript cannot represent integers outside this range faithfully, even nested in defaults.
fn portable_numbers(value: &Value) -> bool {
    match value {
        Value::Number(n) if n.is_i64() || n.is_u64() => n
            .as_i64()
            .is_some_and(|v| (-9_007_199_254_740_991..=9_007_199_254_740_991).contains(&v)),
        Value::Array(values) => values.iter().all(portable_numbers),
        Value::Object(values) => values.values().all(portable_numbers),
        _ => true,
    }
}

fn validate(def: &mut Definition) -> Result<()> {
    ensure!(
        event_identifier(&def.from),
        "Invalid source identifier: {}",
        def.from
    );
    ensure!(
        event_identifier(&def.to),
        "Invalid target identifier: {}",
        def.to
    );
    ensure!(
        def.from != def.to,
        "Source and target identifiers must differ"
    );
    let mut targets = BTreeSet::new();
    for mapping in &def.mappings {
        ensure!(
            identifier(&mapping.target_field),
            "Invalid target field identifier: {}",
            mapping.target_field
        );
        ensure!(
            targets.insert(&mapping.target_field),
            "Duplicate target field: {}",
            mapping.target_field
        );
        ensure!(
            matches!(
                mapping.r#type.as_str(),
                "boolean" | "string" | "integer" | "number" | "array" | "object" | "null"
            ),
            "Unsupported mapping type: {}",
            mapping.r#type
        );
        match (&mapping.source_field, &mapping.default_value) {
            (Some(source), None) => ensure!(identifier(source), "Invalid source field identifier: {source}"),
            (None, Some(value)) => {
                ensure!(matches_type(value, &mapping.r#type), "Default for {} must have type {}", mapping.target_field, mapping.r#type);
                ensure!(portable_numbers(value), "Default for {} contains an integer outside the portable JSON safe-integer range", mapping.target_field);
            }
            _ => anyhow::bail!("Mapping {} requires exactly one of source_field or an explicit default_value; transforms and source/default fallbacks are unsupported", mapping.target_field),
        }
    }
    def.mappings
        .sort_by(|a, b| a.target_field.cmp(&b.target_field));
    Ok(())
}

fn json_literal(value: &str) -> Result<String> {
    Ok(serde_json::to_string(value)?
        .replace('\u{85}', "\\u0085")
        .replace('\u{2028}', "\\u2028")
        .replace('\u{2029}', "\\u2029"))
}

fn context(def: &Definition, name: &str) -> Result<TeraContext> {
    let mut context = TeraContext::new();
    context.insert("from", &def.from);
    context.insert("to", &def.to);
    context.insert("upcaster_name", name);
    let mappings = def.mappings.iter().map(|mapping| -> Result<Value> {
        let source = mapping.source_field.as_deref().unwrap_or("");
        let default_json = serde_json::to_string(&mapping.default_value)?;
        Ok(json!({
            "target_rust": format!("{:?}", mapping.target_field),
            "target_literal": json_literal(&mapping.target_field)?,
            "source_rust": format!("{source:?}"),
            "source_literal": json_literal(source)?,
            "rename": mapping.source_field.as_ref().is_some_and(|source| source != &mapping.target_field),
            "has_default": mapping.default_value.is_some(),
            "default_rust": format!("{default_json:?}"),
            "default_literal": json_literal(&default_json)?,
        }))
    }).collect::<Result<Vec<_>>>()?;
    context.insert("field_mappings", &mappings);
    Ok(context)
}

/// Generates JSON-object upcasters, not DTO conversions or schema migrations.
///
/// Unmapped fields are retained. Explicit source mappings read the original input,
/// remove renamed source keys, and overwrite targets; absent sources are errors.
/// Defaults apply only to absent keys (explicit null is preserved). Mapping types
/// validate defaults, not incoming data. Nested paths and executable transforms
/// are unsupported. A single definition retains the legacy version-based name;
/// multiple definitions use full event identifiers and reject naming collisions.
pub fn generate(tera: &Tera, schema_dir: &Path, lang: &str, output_dir: &Path) -> Result<usize> {
    let languages: &[&str] = match lang.to_ascii_lowercase().as_str() {
        "all" => &["rust", "csharp", "typescript"],
        "rust" => &["rust"],
        "csharp" | "cs" | "dotnet" => &["csharp"],
        "typescript" | "ts" => &["typescript"],
        _ => anyhow::bail!("Unsupported upcaster language: {lang}"),
    };
    let mut files = crate::validator::discover_files(schema_dir, &["yaml", "yml"])?;
    files.sort();
    let mut definitions = Vec::new();
    let mut pairs = BTreeSet::new();
    for file in files {
        let mut def: Definition = serde_yaml::from_str(&std::fs::read_to_string(&file)?)
            .with_context(|| format!("Invalid upcaster YAML: {}", file.display()))?;
        validate(&mut def).with_context(|| format!("Invalid upcaster: {}", file.display()))?;
        ensure!(
            pairs.insert((def.from.clone(), def.to.clone())),
            "Duplicate upcaster mapping: {} -> {}",
            def.from,
            def.to
        );
        definitions.push(def);
    }
    let mut names = BTreeSet::new();
    let mut paths = BTreeSet::new();
    let mut outputs = Vec::new();
    for def in &definitions {
        let component = |id: &str| {
            if definitions.len() == 1 {
                id.rsplit('.').next().unwrap_or(id).to_upper_camel_case()
            } else {
                id.to_upper_camel_case()
            }
        };
        let name = format!("{}To{}Upcaster", component(&def.from), component(&def.to));
        ensure!(identifier(&name), "Invalid generated upcaster name: {name}");
        ensure!(
            names.insert(name.to_ascii_lowercase()),
            "Generated upcaster name collision: {name}"
        );
        let context = context(def, &name)?;
        for language in languages {
            let filename = match *language {
                "rust" => format!("{}.rs", name.to_snake_case()),
                "csharp" => format!("{name}.cs"),
                _ => format!("{}.ts", name.to_kebab_case()),
            };
            ensure!(
                paths.insert(filename.to_ascii_lowercase()),
                "Generated upcaster filename collision: {filename}"
            );
            let rendered = tera
                .render(&format!("{language}/upcaster.tera"), &context)
                .with_context(|| {
                    format!(
                        "Rendering upcaster {} -> {} for {language}",
                        def.from, def.to
                    )
                })?;
            outputs.push((filename, rendered));
        }
    }
    if outputs.is_empty() {
        return Ok(0);
    }
    // Check ancestors as well as existing entries to avoid writing through a symlink.
    for ancestor in output_dir.ancestors() {
        ensure!(
            !ancestor.is_symlink(),
            "Symlinked upcaster output: {}",
            ancestor.display()
        );
    }
    crate::generator::ensure_safe_output(output_dir)?;
    std::fs::create_dir_all(output_dir)?;
    for (filename, rendered) in &outputs {
        let path = output_dir.join(filename);
        std::fs::write(&path, rendered).with_context(|| format!("Writing {}", path.display()))?;
        println!("  Generated upcaster: {}", path.display());
    }
    Ok(outputs.len())
}

pub fn templates() -> Result<Tera> {
    let mut tera = Tera::default();
    tera.add_raw_templates([
        (
            "rust/upcaster.tera",
            include_str!("../templates/rust/upcaster.tera"),
        ),
        (
            "csharp/upcaster.tera",
            include_str!("../templates/csharp/upcaster.tera"),
        ),
        (
            "typescript/upcaster.tera",
            include_str!("../templates/typescript/upcaster.tera"),
        ),
    ])?;
    Ok(tera)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn definition(mappings: &str) -> Result<Definition> {
        let mut def = serde_yaml::from_str(&format!(
            "from: com.example.Event.v1\nto: com.example.Event.v2\nmappings:\n{mappings}"
        ))?;
        validate(&mut def)?;
        Ok(def)
    }

    #[test]
    fn copied_renamed_and_default_fields_render_for_all_languages() -> Result<()> {
        let def = definition(
            "  - {target_field: newName, source_field: oldName, type: string}\n  - {target_field: retained, source_field: retained, type: object}\n  - {target_field: added, default_value: [1, {nested: true}], type: array}"
        )?;
        let context = context(&def, "V1ToV2Upcaster")?;
        let tera = templates()?;
        let rust = tera.render("rust/upcaster.tera", &context)?;
        assert!(rust.contains("let mut target = source.clone()"));
        assert!(rust.contains("target.remove(\"oldName\")"));
        assert!(rust.contains("source.get(\"oldName\")"));
        assert!(!rust.contains("target.remove(\"retained\")"));
        assert!(rust.contains("!target.contains_key(\"added\")"));
        let cs = tera.render("csharp/upcaster.tera", &context)?;
        assert!(cs.contains("source.DeepClone()"));
        assert!(cs.contains("source.TryGetPropertyValue(\"oldName\""));
        let ts = tera.render("typescript/upcaster.tera", &context)?;
        assert!(ts.contains("Record<string, unknown>"));
        assert!(!ts.contains("any"));
        assert!(ts.contains("...source"));
        assert!(ts.contains("delete target[\"oldName\"]"));
        Ok(())
    }

    #[test]
    fn rejects_invalid_or_ambiguous_mappings() {
        for mapping in [
            "{target_field: x, type: string}",
            "{target_field: x, type: boolean, default_value: 'false'}",
            "{target_field: x, type: string, default_value: null}",
            "{target_field: x, type: date, default_value: '2020-01-01'}",
            "{target_field: x, source_field: y, default_value: 1, type: integer}",
            "{target_field: x, source_field: y, transform: trim, type: string}",
            "{target_field: '../x', default_value: 1, type: integer}",
            "{target_field: x, default_value: 9007199254740992, type: integer}",
            "{target_field: x, default_value: {nested: [9007199254740992]}, type: object}",
        ] {
            assert!(definition(&format!("  - {mapping}")).is_err(), "{mapping}");
        }
        assert!(definition("  - {target_field: x, default_value: 1, type: integer}\n  - {target_field: x, default_value: 2, type: integer}").is_err());
    }

    #[test]
    fn defaults_preserve_json_and_escape_literals() -> Result<()> {
        let def = definition("  - {target_field: empty, default_value: null, type: 'null'}\n  - {target_field: text, default_value: 'quotes \" and \\ slash', type: string}\n  - {target_field: object, default_value: {__proto__: {safe: true}}, type: object}")?;
        let ctx = context(&def, "V1ToV2Upcaster")?;
        let values = ctx
            .get("field_mappings")
            .context("Missing mappings")?
            .as_array()
            .context("Mappings not array")?;
        assert_eq!(values[0]["has_default"], true);
        assert_eq!(values[0]["default_literal"], "\"null\"");
        for lang in ["rust", "csharp", "typescript"] {
            templates()?.render(&format!("{lang}/upcaster.tera"), &ctx)?;
        }
        Ok(())
    }
}
