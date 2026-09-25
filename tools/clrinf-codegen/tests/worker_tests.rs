use std::io::Write;
use std::path::PathBuf;
use std::process::{Command, Stdio};
use serde_json::{json, Value};

fn bin_path() -> PathBuf {
    PathBuf::from(env!("CARGO_BIN_EXE_clrinf-codegen"))
}

fn root_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

#[test]
fn worker_check_detects_all_federated_languages() {
    let output = Command::new(bin_path())
        .arg("worker")
        .arg("check")
        .current_dir(root_dir())
        .output()
        .expect("Failed to run clrinf-codegen worker check");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Federated clrinf Language Workers Status"));
    assert!(stdout.contains("csharp"));
    assert!(stdout.contains("rust"));
    assert!(stdout.contains("typescript"));
    assert!(stdout.contains("elixir"));
}

#[test]
fn lint_command_rejects_unsupported_language() {
    let output = Command::new(bin_path())
        .arg("lint")
        .arg("--lang")
        .arg("nonexistent")
        .current_dir(root_dir())
        .output()
        .expect("Failed to run clrinf-codegen lint");

    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("Desteklenmeyen dil"));
}

#[test]
fn mcp_stdio_jsonrpc_lifecycle_and_tools() {
    let mut child = Command::new(bin_path())
        .arg("mcp")
        .current_dir(root_dir())
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("Failed to spawn clrinf-codegen mcp");

    let mut stdin = child.stdin.take().expect("Failed to open stdin");

    // 1. Initialize
    let init_req = json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "initialize"
    });
    writeln!(stdin, "{}", serde_json::to_string(&init_req).unwrap()).unwrap();

    // 2. Tools list
    let list_req = json!({
        "jsonrpc": "2.0",
        "id": 2,
        "method": "tools/list"
    });
    writeln!(stdin, "{}", serde_json::to_string(&list_req).unwrap()).unwrap();

    // 3. Inspect ecosystem tool call
    let call_req = json!({
        "jsonrpc": "2.0",
        "id": 3,
        "method": "tools/call",
        "params": {
            "name": "clrinf_inspect_ecosystem",
            "arguments": {}
        }
    });
    writeln!(stdin, "{}", serde_json::to_string(&call_req).unwrap()).unwrap();

    // 4. Validate schemas tool call
    let val_req = json!({
        "jsonrpc": "2.0",
        "id": 4,
        "method": "tools/call",
        "params": {
            "name": "clrinf_validate_schemas",
            "arguments": {
                "schema_dir": "./tools/clrinf-codegen/schemas"
            }
        }
    });
    writeln!(stdin, "{}", serde_json::to_string(&val_req).unwrap()).unwrap();

    drop(stdin); // Close stdin to end process

    let output = child.wait_with_output().expect("Failed to read output");
    let stdout = String::from_utf8_lossy(&output.stdout);
    let lines: Vec<&str> = stdout.lines().filter(|l| !l.trim().is_empty()).collect();

    assert!(lines.len() >= 4, "Expected at least 4 responses, got: {}", stdout);

    // Verify Initialize response
    let init_res: Value = serde_json::from_str(lines[0]).unwrap();
    assert_eq!(init_res.get("id").unwrap().as_i64(), Some(1));
    assert_eq!(
        init_res["result"]["serverInfo"]["name"].as_str(),
        Some("clrinf-meta-mcp")
    );

    // Verify Tools List response
    let list_res: Value = serde_json::from_str(lines[1]).unwrap();
    assert_eq!(list_res.get("id").unwrap().as_i64(), Some(2));
    let tools = list_res["result"]["tools"].as_array().unwrap();
    assert!(tools.len() >= 6);
    let names: Vec<&str> = tools.iter().map(|t| t["name"].as_str().unwrap()).collect();
    assert!(names.contains(&"clrinf_inspect_ecosystem"));
    assert!(names.contains(&"clrinf_lint_architecture"));
    assert!(names.contains(&"clrinf_scaffold_rule"));
    assert!(names.contains(&"clrinf_generate_contracts"));
    assert!(names.contains(&"clrinf_validate_schemas"));
    assert!(names.contains(&"clrinf_new_project"));
    assert!(names.contains(&"clrinf_project_status"));
    assert!(names.contains(&"clrinf_module_manage"));
    assert!(names.contains(&"clrinf_add_domain_module"));
    assert!(names.contains(&"clrinf_add_entity"));

    // Verify Inspect Ecosystem response
    let inspect_res: Value = serde_json::from_str(lines[2]).unwrap();
    assert_eq!(inspect_res.get("id").unwrap().as_i64(), Some(3));
    let content_text = inspect_res["result"]["content"][0]["text"].as_str().unwrap();
    assert!(content_text.contains("clrinf-codegen Meta Orchestrator"));
    assert!(content_text.contains("Polyglot CloudEvents"));

    // Verify Validate Schemas response
    let val_res: Value = serde_json::from_str(lines[3]).unwrap();
    assert_eq!(val_res.get("id").unwrap().as_i64(), Some(4));
    let val_text = val_res["result"]["content"][0]["text"].as_str().unwrap();
    assert!(val_text.contains("\"status\": \"PASS\""));
}

#[test]
fn test_mcp_docs_and_graft_tools() {
    let mut cmd = Command::new(bin_path());
    cmd.arg("mcp");
    cmd.current_dir(root_dir());
    cmd.stdin(Stdio::piped());
    cmd.stdout(Stdio::piped());

    let mut child = cmd.spawn().expect("Failed to start mcp");
    let mut stdin = child.stdin.take().expect("Failed to open stdin");

    // 1. Initialize
    let init_req = json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "initialize"
    });
    writeln!(stdin, "{}", serde_json::to_string(&init_req).unwrap()).unwrap();

    // 2. Tools list
    let list_req = json!({
        "jsonrpc": "2.0",
        "id": 2,
        "method": "tools/list"
    });
    writeln!(stdin, "{}", serde_json::to_string(&list_req).unwrap()).unwrap();

    // 3. Get docs for C# (token-isolated)
    let docs_req = json!({
        "jsonrpc": "2.0",
        "id": 3,
        "method": "tools/call",
        "params": {
            "name": "clrinf_get_docs",
            "arguments": {
                "lang": "csharp"
            }
        }
    });
    writeln!(stdin, "{}", serde_json::to_string(&docs_req).unwrap()).unwrap();

    // 4. Graft ask tool call
    let graft_req = json!({
        "jsonrpc": "2.0",
        "id": 4,
        "method": "tools/call",
        "params": {
            "name": "clrinf_graft_ask",
            "arguments": {
                "query": "IBusinessRule"
            }
        }
    });
    writeln!(stdin, "{}", serde_json::to_string(&graft_req).unwrap()).unwrap();

    drop(stdin);

    let output = child.wait_with_output().expect("Failed to read output");
    let stdout = String::from_utf8_lossy(&output.stdout);
    let lines: Vec<&str> = stdout.lines().filter(|l| !l.trim().is_empty()).collect();

    assert!(lines.len() >= 4, "Expected at least 4 responses, got: {}", stdout);

    // Verify tools list includes new tools
    let list_res: Value = serde_json::from_str(lines[1]).unwrap();
    let tools = list_res["result"]["tools"].as_array().unwrap();
    let names: Vec<&str> = tools.iter().map(|t| t["name"].as_str().unwrap()).collect();
    assert!(names.contains(&"clrinf_get_docs"));
    assert!(names.contains(&"clrinf_graft_ask"));
    assert!(names.contains(&"clrinf_graft_skeleton"));
    assert!(names.contains(&"clrinf_scaffold_otp"));

    // Verify docs call returns C# specific instructions
    let docs_res: Value = serde_json::from_str(lines[2]).unwrap();
    let docs_text = docs_res["result"]["content"][0]["text"].as_str().unwrap();
    assert!(docs_text.contains("ARCH001"));
    assert!(docs_text.contains("IBusinessRule"));
    assert!(!docs_text.contains("RUST_ARCH001")); // Verifies language isolation!

    // Verify graft response
    let graft_res: Value = serde_json::from_str(lines[3]).unwrap();
    let graft_text = graft_res["result"]["content"][0]["text"].as_str().unwrap();
    assert!(graft_text.contains("success") || graft_text.contains("result"));
}

#[test]
fn test_cli_docs_command() {
    let output = Command::new(bin_path())
        .args(["docs", "--lang", "rust"])
        .current_dir(root_dir())
        .output()
        .expect("Failed to run clrinf-codegen docs");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("RUST_ARCH001"));
    assert!(stdout.contains("Zero-Panic Policy"));
    assert!(!stdout.contains("ARCH001 (Zero Leakage)")); // Verifies language isolation!
}

#[test]
fn test_scaffold_otp_cli() {
    let tmp = tempfile::tempdir().unwrap();
    let output = Command::new(bin_path())
        .args([
            "scaffold",
            "otp",
            "OrderProcessor",
            "--app",
            "my_store",
            "--path",
            tmp.path().to_str().unwrap(),
            "--templates-dir",
            root_dir().join("tools/clrinf-codegen/templates").to_str().unwrap(),
        ])
        .current_dir(root_dir())
        .output()
        .expect("Failed to run clrinf-codegen scaffold otp");

    assert!(output.status.success(), "stderr: {}", String::from_utf8_lossy(&output.stderr));
    let worker = tmp.path().join("order_processor/worker.ex");
    let supervisor = tmp.path().join("order_processor/supervisor.ex");
    let handler = tmp.path().join("order_processor/handler.ex");

    assert!(worker.exists());
    assert!(supervisor.exists());
    assert!(handler.exists());

    let worker_code = std::fs::read_to_string(&worker).unwrap();
    assert!(worker_code.contains("defmodule MyStore.OrderProcessor.Worker do"));
    assert!(worker_code.contains("use GenServer"));
    assert!(worker_code.contains("@impl true"));

    let supervisor_code = std::fs::read_to_string(&supervisor).unwrap();
    assert!(supervisor_code.contains("defmodule MyStore.OrderProcessor.Supervisor do"));
    assert!(supervisor_code.contains("use Supervisor"));

    let handler_code = std::fs::read_to_string(&handler).unwrap();
    assert!(handler_code.contains("defmodule MyStore.OrderProcessor.Handler do"));
    assert!(handler_code.contains("execute(params)"));
}

#[test]
fn test_scaffold_elixir_rule_cli() {
    let tmp = tempfile::tempdir().unwrap();
    let output = Command::new(bin_path())
        .args([
            "rule",
            "new",
            "MaxDiscountRule",
            "--lang",
            "elixir",
            "--entity",
            "Order",
            "--error-code",
            "MAX_DISCOUNT_EXCEEDED",
            "--target-dir",
            tmp.path().to_str().unwrap(),
        ])
        .current_dir(root_dir())
        .output()
        .expect("Failed to run clrinf-codegen rule new");

    assert!(output.status.success(), "stderr: {}", String::from_utf8_lossy(&output.stderr));
    let rule = tmp.path().join("max_discount_rule.ex");
    let rule_test = tmp.path().join("max_discount_rule_test.exs");

    assert!(rule.exists());
    assert!(rule_test.exists());

    let rule_code = std::fs::read_to_string(&rule).unwrap();
    assert!(rule_code.contains("MAX_DISCOUNT_EXCEEDED"));
    assert!(rule_code.contains("check(subject)"));
}

#[test]
fn test_cli_docs_elixir() {
    let output = Command::new(bin_path())
        .args(["docs", "--lang", "elixir"])
        .current_dir(root_dir())
        .output()
        .expect("Failed to run clrinf-codegen docs");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("ARCH_EX_001"));
    assert!(stdout.contains("Pure Domain Result Monad"));
    assert!(!stdout.contains("ARCH001 (Zero Leakage)"));
}


