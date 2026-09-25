use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::sync::atomic::{AtomicUsize, Ordering};

static NEXT: AtomicUsize = AtomicUsize::new(0);

struct Scratch(PathBuf);
impl Scratch {
    fn new() -> Self {
        let path = std::env::temp_dir().join(format!(
            "clrinf-codegen-topology-test-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        std::fs::create_dir_all(&path).unwrap();
        Self(path)
    }
}
impl Drop for Scratch {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
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

#[test]
fn test_topology_detects_dead_event() {
    let scratch = Scratch::new();
    let schema_dir = scratch.0.join("schemas");
    std::fs::create_dir_all(&schema_dir).unwrap();

    let event_json = r#"{
        "$schema": "https://json-schema.org/draft/2020-12/schema",
        "title": "PaymentCompleted",
        "x-domain": "payment",
        "x-event-type": "payment.completed.v1",
        "x-published-by": ["payment-service"],
        "type": "object",
        "properties": {
            "paymentId": { "type": "string" }
        },
        "required": ["paymentId"]
    }"#;

    std::fs::write(schema_dir.join("payment_completed.json"), event_json).unwrap();

    let output = run(
        &[
            "topology",
            "check",
            "--schema-dir",
            schema_dir.to_str().unwrap(),
            "--upcasters-dir",
            schema_dir.to_str().unwrap(),
        ],
        &scratch.0,
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(output.status.success(), "Warnings should not fail validation: {:?}", output);
    assert!(stdout.contains("TOPOLOGY_DEAD_EVENT"), "Expected TOPOLOGY_DEAD_EVENT warning, got: {}", stdout);
    assert!(stdout.contains("payment.completed.v1"), "Expected event name in output, got: {}", stdout);
}

#[test]
fn test_topology_detects_orphan_subscriber() {
    let scratch = Scratch::new();
    let schema_dir = scratch.0.join("schemas");
    std::fs::create_dir_all(&schema_dir).unwrap();

    let event_json = r#"{
        "$schema": "https://json-schema.org/draft/2020-12/schema",
        "title": "InvoiceIssued",
        "x-domain": "billing",
        "x-event-type": "billing.invoice.issued.v1",
        "x-subscribed-by": ["notification-service"],
        "type": "object",
        "properties": {
            "invoiceId": { "type": "string" }
        },
        "required": ["invoiceId"]
    }"#;

    std::fs::write(schema_dir.join("invoice_issued.json"), event_json).unwrap();

    let output = run(
        &[
            "topology",
            "check",
            "--schema-dir",
            schema_dir.to_str().unwrap(),
            "--upcasters-dir",
            schema_dir.to_str().unwrap(),
        ],
        &scratch.0,
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(!output.status.success(), "Orphan subscriber must fail topology check");
    assert!(
        stdout.contains("TOPOLOGY_ORPHAN_SUBSCRIBER") || stderr.contains("TOPOLOGY_ORPHAN_SUBSCRIBER"),
        "Expected TOPOLOGY_ORPHAN_SUBSCRIBER in output:\nStdout: {}\nStderr: {}",
        stdout, stderr
    );
}

#[test]
fn test_topology_detects_missing_upcaster_chain() {
    let scratch = Scratch::new();
    let schema_dir = scratch.0.join("schemas");
    std::fs::create_dir_all(&schema_dir).unwrap();

    // v1 published by order-service
    let v1_json = r#"{
        "$schema": "https://json-schema.org/draft/2020-12/schema",
        "title": "OrderPlacedV1",
        "x-domain": "ordering",
        "x-event-type": "ordering.order.placed.v1",
        "x-published-by": ["order-service"],
        "type": "object",
        "properties": {
            "orderId": { "type": "string" }
        },
        "required": ["orderId"]
    }"#;

    // v2 subscribed by shipping-service
    let v2_json = r#"{
        "$schema": "https://json-schema.org/draft/2020-12/schema",
        "title": "OrderPlacedV2",
        "x-domain": "ordering",
        "x-event-type": "ordering.order.placed.v2",
        "x-subscribed-by": ["shipping-service"],
        "type": "object",
        "properties": {
            "orderId": { "type": "string" },
            "currency": { "type": "string" }
        },
        "required": ["orderId", "currency"]
    }"#;

    std::fs::write(schema_dir.join("order_placed_v1.json"), v1_json).unwrap();
    std::fs::write(schema_dir.join("order_placed_v2.json"), v2_json).unwrap();

    let output = run(
        &[
            "topology",
            "check",
            "--schema-dir",
            schema_dir.to_str().unwrap(),
            "--upcasters-dir",
            schema_dir.to_str().unwrap(),
        ],
        &scratch.0,
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(!output.status.success(), "Missing upcaster chain must fail check");
    assert!(
        stdout.contains("TOPOLOGY_UPCASTER_MISSING") || stderr.contains("TOPOLOGY_UPCASTER_MISSING"),
        "Expected TOPOLOGY_UPCASTER_MISSING:\nStdout: {}\nStderr: {}",
        stdout, stderr
    );

    // Now supply the upcaster YAML and verify the missing upcaster error disappears!
    let upcaster_yaml = r#"
from: OrderPlacedV1
to: OrderPlacedV2
steps:
  - add_field:
      name: currency
      value: "TRY"
"#;
    std::fs::write(schema_dir.join("order_placed_upcaster.yaml"), upcaster_yaml).unwrap();

    let output2 = run(
        &[
            "topology",
            "check",
            "--schema-dir",
            schema_dir.to_str().unwrap(),
            "--upcasters-dir",
            schema_dir.to_str().unwrap(),
        ],
        &scratch.0,
    );

    let stdout2 = String::from_utf8_lossy(&output2.stdout);
    let stderr2 = String::from_utf8_lossy(&output2.stderr);
    assert!(
        !stdout2.contains("TOPOLOGY_UPCASTER_MISSING") && !stderr2.contains("TOPOLOGY_UPCASTER_MISSING"),
        "Expected upcaster error to be resolved with YAML definition:\nStdout: {}\nStderr: {}",
        stdout2, stderr2
    );
}

#[test]
fn test_generate_pubsub_across_all_languages() {
    let scratch = Scratch::new();
    let schema_dir = scratch.0.join("schemas");
    let templates_dir = root().join("templates");
    let out_dir = scratch.0.join("pubsub");
    std::fs::create_dir_all(&schema_dir).unwrap();

    let event_json = r#"{
        "$schema": "https://json-schema.org/draft/2020-12/schema",
        "title": "OrderCreated",
        "x-domain": "ordering",
        "x-event-type": "ordering.order.created.v1",
        "x-published-by": ["order-service"],
        "x-subscribed-by": ["billing-service", "inventory-service"],
        "type": "object",
        "properties": {
            "orderId": { "type": "string" },
            "amount": { "type": "number" }
        },
        "required": ["orderId", "amount"]
    }"#;

    std::fs::write(schema_dir.join("order_created.json"), event_json).unwrap();

    let output = run(
        &[
            "generate-pubsub",
            "--schema-dir",
            schema_dir.to_str().unwrap(),
            "--templates-dir",
            templates_dir.to_str().unwrap(),
            "--lang",
            "all",
            "--output",
            out_dir.to_str().unwrap(),
        ],
        &scratch.0,
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(output.status.success(), "generate-pubsub all failed: stdout={}\nstderr={}", stdout, stderr);

    // Verify Rust artifacts
    let rust_pub = out_dir.join("rust/order_created_publisher.rs");
    let rust_sub = out_dir.join("rust/order_created_subscriber.rs");
    let rust_mod = out_dir.join("rust/mod.rs");
    assert!(rust_pub.exists(), "Rust publisher missing: {}", rust_pub.display());
    assert!(rust_sub.exists(), "Rust subscriber missing: {}", rust_sub.display());
    assert!(rust_mod.exists(), "Rust mod.rs missing");

    let rust_pub_code = std::fs::read_to_string(&rust_pub).unwrap();
    assert!(rust_pub_code.contains("pub async fn publish_order_created"));
    assert!(rust_pub_code.contains("OutboxStore"));
    assert!(rust_pub_code.contains("ordering.order.created.v1"));

    let rust_sub_code = std::fs::read_to_string(&rust_sub).unwrap();
    assert!(rust_sub_code.contains("pub trait OrderCreatedHandler"));
    assert!(rust_sub_code.contains("IdempotencyStore"));
    assert!(rust_sub_code.contains("handle_order_created_event"));

    // Verify C# artifacts
    let cs_pub = out_dir.join("csharp/OrderCreatedPublisher.cs");
    let cs_sub = out_dir.join("csharp/OrderCreatedSubscriber.cs");
    assert!(cs_pub.exists(), "C# publisher missing");
    assert!(cs_sub.exists(), "C# subscriber missing");

    let cs_pub_code = std::fs::read_to_string(&cs_pub).unwrap();
    assert!(cs_pub_code.contains("public static class OrderCreatedPublisher"));
    assert!(cs_pub_code.contains("IOutboxStore outbox"));

    let cs_sub_code = std::fs::read_to_string(&cs_sub).unwrap();
    assert!(cs_sub_code.contains("public interface IOrderCreatedHandler"));
    assert!(cs_sub_code.contains("IIdempotencyStore"));

    // Verify TypeScript artifacts
    let ts_pub = out_dir.join("typescript/order-created-publisher.ts");
    let ts_sub = out_dir.join("typescript/order-created-subscriber.ts");
    let ts_index = out_dir.join("typescript/index.ts");
    assert!(ts_pub.exists(), "TS publisher missing");
    assert!(ts_sub.exists(), "TS subscriber missing");
    assert!(ts_index.exists(), "TS index.ts missing");

    let ts_pub_code = std::fs::read_to_string(&ts_pub).unwrap();
    assert!(ts_pub_code.contains("export async function publishOrderCreated"));
    assert!(ts_pub_code.contains("OutboxStore"));

    let ts_sub_code = std::fs::read_to_string(&ts_sub).unwrap();
    assert!(ts_sub_code.contains("export interface OrderCreatedHandler"));
    assert!(ts_sub_code.contains("IdempotencyStore"));
}

#[test]
fn test_mcp_pubsub_and_topology_tools() {
    use std::io::Write;
    use std::process::Stdio;
    use serde_json::json;

    let scratch = Scratch::new();
    let schema_dir = scratch.0.join("schemas");
    std::fs::create_dir_all(&schema_dir).unwrap();

    let event_json = r#"{
        "$schema": "https://json-schema.org/draft/2020-12/schema",
        "title": "InventoryReserved",
        "x-domain": "warehouse",
        "x-event-type": "warehouse.inventory.reserved.v1",
        "x-published-by": ["warehouse-service"],
        "x-subscribed-by": ["order-service"],
        "type": "object",
        "properties": {
            "sku": { "type": "string" }
        },
        "required": ["sku"]
    }"#;
    std::fs::write(schema_dir.join("inventory_reserved.json"), event_json).unwrap();

    let mut child = Command::new(env!("CARGO_BIN_EXE_clrinf-codegen"))
        .arg("mcp")
        .current_dir(root().join("../.."))
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("Failed to spawn clrinf-codegen mcp");

    let mut stdin = child.stdin.take().expect("Failed to take stdin");

    // 1. tools/list
    let list_req = json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "tools/list"
    });
    writeln!(stdin, "{}", serde_json::to_string(&list_req).unwrap()).unwrap();

    // 2. call clrinf_validate_topology
    let topo_req = json!({
        "jsonrpc": "2.0",
        "id": 2,
        "method": "tools/call",
        "params": {
            "name": "clrinf_validate_topology",
            "arguments": {
                "schema_dir": schema_dir.to_str().unwrap()
            }
        }
    });
    writeln!(stdin, "{}", serde_json::to_string(&topo_req).unwrap()).unwrap();

    // 3. call clrinf_generate_pubsub
    let pubsub_out = scratch.0.join("pubsub_mcp");
    let pubsub_req = json!({
        "jsonrpc": "2.0",
        "id": 3,
        "method": "tools/call",
        "params": {
            "name": "clrinf_generate_pubsub",
            "arguments": {
                "schema_dir": schema_dir.to_str().unwrap(),
                "lang": "rust",
                "output": pubsub_out.to_str().unwrap()
            }
        }
    });
    writeln!(stdin, "{}", serde_json::to_string(&pubsub_req).unwrap()).unwrap();

    drop(stdin);

    let output = child.wait_with_output().expect("Failed to wait child");
    let stdout = String::from_utf8_lossy(&output.stdout);
    let lines: Vec<&str> = stdout.lines().filter(|l| !l.trim().is_empty()).collect();

    assert!(lines.len() >= 3, "Expected 3 responses from MCP server, got: {}", stdout);

    // Verify tools/list contains the new tools
    let list_res: serde_json::Value = serde_json::from_str(lines[0]).unwrap();
    let tools = list_res["result"]["tools"].as_array().unwrap();
    let names: Vec<&str> = tools.iter().map(|t| t["name"].as_str().unwrap()).collect();
    assert!(names.contains(&"clrinf_validate_topology"), "clrinf_validate_topology missing from tools/list");
    assert!(names.contains(&"clrinf_generate_pubsub"), "clrinf_generate_pubsub missing from tools/list");

    // Verify clrinf_validate_topology call result
    let topo_res: serde_json::Value = serde_json::from_str(lines[1]).unwrap();
    let topo_text = topo_res["result"]["content"][0]["text"].as_str().unwrap();
    assert!(topo_text.contains("\"isValid\": true"));
    assert!(topo_text.contains("warehouse-service"));

    // Verify clrinf_generate_pubsub call result
    let pubsub_res: serde_json::Value = serde_json::from_str(lines[2]).unwrap();
    let pubsub_text = pubsub_res["result"]["content"][0]["text"].as_str().unwrap();
    assert!(pubsub_text.contains("\"success\": true"));
    assert!(pubsub_out.join("inventory_reserved_publisher.rs").exists());
    assert!(pubsub_out.join("inventory_reserved_subscriber.rs").exists());
}

