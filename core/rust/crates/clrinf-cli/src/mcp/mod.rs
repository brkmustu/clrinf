use crate::generator::{ArchStyle, Generator};
use crate::lint::Linter;
use serde_json::{json, Value};
use std::path::Path;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};

pub struct McpServer;

impl McpServer {
    pub async fn run() -> Result<(), Box<dyn std::error::Error>> {
        let stdin = tokio::io::stdin();
        let mut stdout = tokio::io::stdout();
        let mut reader = BufReader::new(stdin);
        let mut line = String::new();

        while reader.read_line(&mut line).await? > 0 {
            let trimmed = line.trim();
            if trimmed.is_empty() {
                line.clear();
                continue;
            }

            if let Ok(request) = serde_json::from_str::<Value>(trimmed) {
                let id = request.get("id").cloned();
                let method = request.get("method").and_then(|m| m.as_str()).unwrap_or("");

                if let Some(id_val) = id {
                    let response = Self::handle_method(method, &request, id_val).await;
                    let out_bytes = serde_json::to_vec(&response)?;
                    stdout.write_all(&out_bytes).await?;
                    stdout.write_all(b"\n").await?;
                    stdout.flush().await?;
                }
            }

            line.clear();
        }

        Ok(())
    }

    async fn handle_method(method: &str, request: &Value, id: Value) -> Value {
        match method {
            "initialize" => json!({
                "jsonrpc": "2.0",
                "id": id,
                "result": {
                    "protocolVersion": "2024-11-05",
                    "capabilities": { "tools": {} },
                    "serverInfo": { "name": "clrinfrs-mcp", "version": "0.1.0" }
                }
            }),

            "tools/list" => json!({
                "jsonrpc": "2.0",
                "id": id,
                "result": {
                    "tools": [
                        {
                            "name": "clrinfrs_lint",
                            "description": "Syn AST ile Rust mimari kurallarını (Domain saflığı, kural sözleşmeleri) deterministik olarak denetler.",
                            "inputSchema": {
                                "type": "object",
                                "properties": {
                                    "path": { "type": "string", "description": "Denetlenecek proje dizini" }
                                }
                            }
                        },
                        {
                            "name": "clrinfrs_generate_project",
                            "description": "Yeni bir Rust projesi üretir (flat, layered veya clean-cqrs).",
                            "inputSchema": {
                                "type": "object",
                                "properties": {
                                    "name": { "type": "string", "description": "Proje adı" },
                                    "arch": { "type": "string", "description": "flat, layered veya clean-cqrs" },
                                    "target_dir": { "type": "string", "description": "Hedef dizin" }
                                },
                                "required": ["name"]
                            }
                        },
                        {
                            "name": "clrinfrs_add_rule",
                            "description": "Mevcut hiçbir dosyaya dokunmadan, bağımsız bir Rust BusinessRule struct'ı oluşturur.",
                            "inputSchema": {
                                "type": "object",
                                "properties": {
                                    "project_path": { "type": "string", "description": "Proje kök dizini" },
                                    "module": { "type": "string", "description": "Modül adı" },
                                    "rule_name": { "type": "string", "description": "Kural adı (örn. MaxDiscountRule)" },
                                    "command_name": { "type": "string", "description": "Hedef komut adı" }
                                },
                                "required": ["rule_name", "command_name"]
                            }
                        }
                    ]
                }
            }),

            "tools/call" => {
                let params = request.get("params");
                let tool_name = params.and_then(|p| p.get("name")).and_then(|n| n.as_str()).unwrap_or("");
                let args = params.and_then(|p| p.get("arguments"));

                let res = match tool_name {
                    "clrinfrs_lint" => {
                        let path = args.and_then(|a| a.get("path")).and_then(|p| p.as_str()).unwrap_or(".");
                        let violations = Linter::lint_directory(Path::new(path));
                        let text = if violations.is_empty() {
                            "✔ Tebrikler! Hiçbir Rust mimari veya kural ihlali bulunamadı.".to_string()
                        } else {
                            serde_json::to_string_pretty(&violations).unwrap_or_default()
                        };
                        json!({
                            "content": [{ "type": "text", "text": text }],
                            "isError": !violations.is_empty()
                        })
                    }

                    "clrinfrs_generate_project" => {
                        let name = args.and_then(|a| a.get("name")).and_then(|n| n.as_str()).unwrap_or("MyApp");
                        let arch_str = args.and_then(|a| a.get("arch")).and_then(|a| a.as_str()).unwrap_or("clean-cqrs");
                        let target_dir = args.and_then(|a| a.get("target_dir")).and_then(|t| t.as_str()).unwrap_or(".");
                        let arch = ArchStyle::parse_str(arch_str);

                        match Generator::new_project(name, Path::new(target_dir), arch, "api") {
                            Ok(_) => json!({
                                "content": [{ "type": "text", "text": format!("✔ Proje '{name}' ({arch_str}) başarıyla oluşturuldu.") }],
                                "isError": false
                            }),
                            Err(e) => json!({
                                "content": [{ "type": "text", "text": format!("Hata: {e}") }],
                                "isError": true
                            }),
                        }
                    }

                    "clrinfrs_add_rule" => {
                        let project_path = args.and_then(|a| a.get("project_path")).and_then(|p| p.as_str()).unwrap_or(".");
                        let module = args.and_then(|a| a.get("module")).and_then(|m| m.as_str()).unwrap_or("Common");
                        let rule_name = args.and_then(|a| a.get("rule_name")).and_then(|r| r.as_str()).unwrap_or("CustomRule");
                        let command_name = args.and_then(|a| a.get("command_name")).and_then(|c| c.as_str()).unwrap_or("CustomCommand");

                        match Generator::add_rule(Path::new(project_path), module, rule_name, command_name) {
                            Ok(path) => json!({
                                "content": [{ "type": "text", "text": format!("✔ İzole kural oluşturuldu: {}\nMevcut hiçbir dosyaya dokunulmadı.", path.display()) }],
                                "isError": false
                            }),
                            Err(e) => json!({
                                "content": [{ "type": "text", "text": format!("Hata: {e}") }],
                                "isError": true
                            }),
                        }
                    }

                    _ => json!({
                        "content": [{ "type": "text", "text": format!("Bilinmeyen araç: {tool_name}") }],
                        "isError": true
                    }),
                };

                json!({
                    "jsonrpc": "2.0",
                    "id": id,
                    "result": res
                })
            }

            _ => json!({
                "jsonrpc": "2.0",
                "id": id,
                "error": { "code": -32601, "message": format!("Method '{method}' not found") }
            }),
        }
    }
}
