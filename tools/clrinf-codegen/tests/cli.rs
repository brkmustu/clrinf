use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::sync::atomic::{AtomicUsize, Ordering};

static NEXT: AtomicUsize = AtomicUsize::new(0);
struct Scratch(PathBuf);
impl Scratch {
    fn new() -> Self {
        let path = std::env::temp_dir().join(format!(
            "clrinf-codegen-test-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        std::fs::create_dir(&path).unwrap();
        Self(path)
    }
}
impl Drop for Scratch {
    fn drop(&mut self) {
        std::fs::remove_dir_all(&self.0).unwrap();
    }
}
fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}
fn run(args: &[&str], cwd: &Path) -> Output {
    Command::new(env!("CARGO_BIN_EXE_clrinf-codegen"))
        .args(args)
        .current_dir(cwd)
        .output()
        .unwrap()
}
fn failure(output: Output, text: &str) {
    assert!(!output.status.success(), "Unexpected success: {:?}", output);
    assert!(
        String::from_utf8_lossy(&output.stderr).contains(text),
        "{:?}",
        output
    );
}
fn files(dir: &Path) -> Vec<(PathBuf, Vec<u8>)> {
    let mut result = Vec::new();
    for entry in walkdir::WalkDir::new(dir).sort_by_file_name() {
        let e = entry.unwrap();
        if e.file_type().is_file() {
            let bytes = std::fs::read(e.path()).unwrap();
            let normalized: Vec<u8> = bytes
                .into_iter()
                .filter(|&b| b != b'\r')
                .collect();
            result.push((
                e.path().strip_prefix(dir).unwrap().to_path_buf(),
                normalized,
            ));
        }
    }
    result
}

#[test]
fn all_languages_match_golden_and_are_deterministic_outside_checkout() {
    let scratch = Scratch::new();
    let schema = root().join("tests/fixtures");
    let templates = root().join("templates");
    for name in ["first", "second"] {
        let output = scratch.0.join(name);
        let result = run(
            &[
                "generate",
                "--schema-dir",
                schema.to_str().unwrap(),
                "--templates-dir",
                templates.to_str().unwrap(),
                "--output",
                output.to_str().unwrap(),
            ],
            &scratch.0,
        );
        assert!(result.status.success(), "{:?}", result);
    }
    assert_eq!(
        files(&scratch.0.join("first")),
        files(&scratch.0.join("second"))
    );
    assert_eq!(
        files(&scratch.0.join("first")),
        files(&root().join("tests/golden"))
    );
}

#[test]
fn rejects_missing_inputs_unknown_languages_and_fake_catalogs() {
    let scratch = Scratch::new();
    failure(
        run(&["validate", "--schema-dir", "missing"], &scratch.0),
        "Input path not found",
    );
    failure(run(&["template", "list"], &scratch.0), "--templates-dir");
    failure(
        run(&["new", "project", "--template", "../escape"], &scratch.0),
        "single directory name",
    );
    let templates = root().join("templates");
    failure(
        run(
            &[
                "generate",
                "--lang",
                "unknown",
                "--output",
                "out",
                "--templates-dir",
                templates.to_str().unwrap(),
            ],
            &scratch.0,
        ),
        "Unsupported language",
    );
}

#[test]
fn check_rejects_unsupported_shapes_and_invalid_schemas() {
    let scratch = Scratch::new();
    let path = scratch.0.join("Invalid.schema.json");
    for unsupported in [
        serde_json::json!({"$ref":"#/definitions/Thing"}),
        serde_json::json!({"type":["string","null"]}),
        serde_json::json!({"type":"string","oneOf":[{"const":"a"},{"const":"b"}]}),
        serde_json::json!({"type":"array","prefixItems":[{"type":"string"}]}),
    ] {
        let schema = serde_json::json!({
            "$schema": "https://json-schema.org/draft/2020-12/schema",
            "$id":"https://example.test/Invalid",
            "type":"object",
            "properties":{"value":unsupported}
        });
        std::fs::write(&path, serde_json::to_vec(&schema).unwrap()).unwrap();
        failure(
            run(
                &["check", "--schema-dir", path.to_str().unwrap()],
                &scratch.0,
            ),
            "Unsupported model schema",
        );
    }
    std::fs::write(
        &path,
        r#"{"$id":"https://example.test/Invalid","type":"nonsense"}"#,
    )
    .unwrap();
    failure(
        run(
            &["validate", "--schema-dir", path.to_str().unwrap()],
            &scratch.0,
        ),
        "Invalid JSON Schema",
    );
}

#[test]
fn detects_duplicate_ids_and_normalized_names() {
    let scratch = Scratch::new();
    let fixture = std::fs::read(root().join("tests/fixtures/Record.v1.schema.json")).unwrap();
    std::fs::write(scratch.0.join("One.json"), &fixture).unwrap();
    std::fs::write(scratch.0.join("Two.json"), &fixture).unwrap();
    failure(
        run(
            &["check", "--schema-dir", scratch.0.to_str().unwrap()],
            &scratch.0,
        ),
        "Duplicate schema",
    );
}

#[test]
fn catalog_uses_sorted_manifests_and_scaffold_never_runs_quickstart() {
    let scratch = Scratch::new();
    let catalog = scratch.0.join("catalog");
    for name in ["zeta", "alpha"] {
        let entry = catalog.join(name);
        std::fs::create_dir_all(&entry).unwrap();
        let manifest = serde_json::json!({
            "name":name, "description":format!("{name} starter"), "language":"rust", "kind":"service",
            "capabilities":["in-process"], "quickstart":["touch SHOULD_NOT_EXIST"]
        });
        std::fs::write(
            entry.join("clrinf-template.json"),
            serde_json::to_vec(&manifest).unwrap(),
        )
        .unwrap();
        std::fs::write(entry.join("source.txt"), b"portable source").unwrap();
    }
    let result = run(
        &[
            "template",
            "list",
            "--templates-dir",
            catalog.to_str().unwrap(),
        ],
        &scratch.0,
    );
    assert!(result.status.success(), "{:?}", result);
    let text = String::from_utf8(result.stdout).unwrap();
    assert!(text.find("alpha starter").unwrap() < text.find("zeta starter").unwrap());
    let result = run(
        &[
            "new",
            "new-project",
            "--template",
            "alpha",
            "--templates-dir",
            catalog.to_str().unwrap(),
        ],
        &scratch.0,
    );
    assert!(result.status.success(), "{:?}", result);
    assert_eq!(
        std::fs::read(scratch.0.join("new-project/source.txt")).unwrap(),
        b"portable source"
    );
    assert!(!scratch.0.join("SHOULD_NOT_EXIST").exists());
    assert!(!scratch.0.join("new-project/SHOULD_NOT_EXIST").exists());
    failure(
        run(
            &[
                "new",
                "new-project",
                "--template",
                "alpha",
                "--templates-dir",
                catalog.to_str().unwrap(),
            ],
            &scratch.0,
        ),
        "empty directory",
    );
    failure(
        run(
            &[
                "new",
                catalog.join("alpha/nested").to_str().unwrap(),
                "--template",
                "alpha",
                "--templates-dir",
                catalog.to_str().unwrap(),
            ],
            &scratch.0,
        ),
        "outside source",
    );
    assert!(!catalog.join("alpha/nested").exists());
}

#[test]
fn canonical_schemas_accept_shared_valid_and_reject_all_invalid_fixtures() {
    let repo = root().join("../..");
    let read_json = |path: PathBuf| -> serde_json::Value {
        serde_json::from_slice(&std::fs::read(path).unwrap()).unwrap()
    };
    let valid = read_json(repo.join("tests/conformance/fixtures/valid.json"));
    let invalid = read_json(repo.join("tests/conformance/fixtures/invalid.json"));
    for (category, file) in [
        ("context", "_context"),
        ("error", "_error"),
        ("event", "_envelope"),
    ] {
        let schema = read_json(root().join(format!("schemas/{file}.schema.json")));
        let validator = jsonschema::options()
            .should_validate_formats(true)
            .build(&schema)
            .unwrap();
        assert!(
            validator.is_valid(valid.get(category).expect("Missing valid fixture")),
            "{category}"
        );
        let cases = invalid
            .get(category)
            .expect("Missing invalid category")
            .as_array()
            .unwrap();
        assert!(!cases.is_empty());
        for case in cases {
            let value = case.get("value").expect("Missing invalid value");
            assert!(
                !validator.is_valid(value),
                "{category}: {}",
                case.get("name").unwrap()
            );
        }
    }
}

#[cfg(unix)]
#[test]
fn rejects_symlinked_inputs_and_templates() {
    use std::os::unix::fs::symlink;
    let scratch = Scratch::new();
    symlink(root().join("tests/fixtures"), scratch.0.join("schemas")).unwrap();
    failure(
        run(
            &[
                "check",
                "--schema-dir",
                scratch.0.join("schemas").to_str().unwrap(),
            ],
            &scratch.0,
        ),
        "Symlinked schema",
    );
    symlink(root(), scratch.0.join("escape")).unwrap();
    failure(
        run(
            &[
                "new",
                "out",
                "--template",
                "escape",
                "--templates-dir",
                scratch.0.to_str().unwrap(),
            ],
            &scratch.0,
        ),
        "symlink",
    );
    let output = scratch.0.join("output");
    std::fs::create_dir(&output).unwrap();
    let protected = scratch.0.join("protected.txt");
    std::fs::write(&protected, "unchanged").unwrap();
    symlink(&protected, output.join("record_v1.rs")).unwrap();
    failure(
        run(
            &[
                "generate",
                "--schema-dir",
                root().join("tests/fixtures").to_str().unwrap(),
                "--templates-dir",
                root().join("templates").to_str().unwrap(),
                "--lang",
                "rust",
                "--output",
                output.to_str().unwrap(),
            ],
            &scratch.0,
        ),
        "symlinked generation output",
    );
    assert_eq!(std::fs::read_to_string(protected).unwrap(), "unchanged");
}
