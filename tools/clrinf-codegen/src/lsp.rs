//! Minimal pure-Rust LSP 3.17 diagnostics server for `clrinf lsp`.
//!
//! Communicates over stdio using JSON-RPC 2.0 + `Content-Length` framing and
//! publishes live architecture/drift diagnostics on `textDocument/didOpen` and
//! `textDocument/didSave`.

use crate::topology::IssueSeverity;
use crate::verify;
use anyhow::{bail, Context, Result};
use serde_json::{json, Value};
use std::io::{BufRead, Write};
use std::path::{Path, PathBuf};

/// Read one `Content-Length`-framed LSP message (`Ok(None)` cleanly at EOF).
pub fn read_message<R: BufRead>(reader: &mut R) -> Result<Option<Value>> {
    let mut content_length: Option<usize> = None;
    let mut header_seen = false;

    loop {
        let mut line = String::new();
        let read = reader.read_line(&mut line)?;
        if read == 0 {
            if !header_seen {
                return Ok(None);
            }
            bail!("Unexpected EOF while reading LSP headers");
        }
        header_seen = true;
        let trimmed = line.trim_end_matches(&['\r', '\n'][..]);
        if trimmed.is_empty() {
            break;
        }
        if let Some((name, value)) = trimmed.split_once(':') {
            if name.trim().eq_ignore_ascii_case("content-length") {
                let len: usize = value
                    .trim()
                    .parse()
                    .context("Invalid LSP Content-Length header")?;
                content_length = Some(len);
            }
        }
    }

    let len = content_length.context("Missing Content-Length header in LSP message")?;
    let mut buf = vec![0u8; len];
    reader.read_exact(&mut buf)?;
    let msg: Value = serde_json::from_slice(&buf).context("Invalid JSON in LSP payload")?;
    Ok(Some(msg))
}

/// Write one `Content-Length`-framed LSP message and flush.
pub fn write_message<W: Write>(writer: &mut W, msg: &Value) -> Result<()> {
    let body = serde_json::to_vec(msg)?;
    let header = format!("Content-Length: {}\r\n\r\n", body.len());
    writer.write_all(header.as_bytes())?;
    writer.write_all(&body)?;
    writer.flush()?;
    Ok(())
}

fn percent_decode(input: &str) -> String {
    let bytes = input.as_bytes();
    let mut out = Vec::with_capacity(bytes.len());
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'%' && i + 2 < bytes.len() {
            if let Ok(hex) = std::str::from_utf8(&bytes[i + 1..i + 3]) {
                if let Ok(val) = u8::from_str_radix(hex, 16) {
                    out.push(val);
                    i += 3;
                    continue;
                }
            }
        }
        out.push(bytes[i]);
        i += 1;
    }
    String::from_utf8_lossy(&out).into_owned()
}

/// Convert a `file://` URI to a local filesystem `PathBuf`.
pub fn uri_to_path(uri: &str) -> Option<PathBuf> {
    let rest = uri.strip_prefix("file://")?;
    let rest = if let Some(after_host) = rest.strip_prefix("localhost/") {
        format!("/{after_host}")
    } else {
        rest.to_string()
    };
    let mut decoded = percent_decode(&rest);
    let b = decoded.as_bytes();
    if b.len() >= 3 && b[0] == b'/' && b[1].is_ascii_alphabetic() && b[2] == b':' {
        decoded.remove(0);
    }
    Some(PathBuf::from(decoded))
}

fn normalize_for_cmp(path: &Path) -> String {
    let resolved = std::fs::canonicalize(path).unwrap_or_else(|_| path.to_path_buf());
    let s = resolved.to_string_lossy();
    let trimmed = s.strip_prefix(r"\\?\").unwrap_or(&s);
    let forward = trimmed.replace('\\', "/");
    if cfg!(windows) {
        forward.to_ascii_lowercase()
    } else {
        forward
    }
}

fn same_file(root: &Path, issue_file: &str, target_file: &Path) -> bool {
    normalize_for_cmp(&root.join(issue_file)) == normalize_for_cmp(target_file)
}

fn publish_diagnostics_for(
    workspace_root: &Path,
    uri: &str,
) -> Option<Value> {
    let target_path = uri_to_path(uri)?;
    let diagnostics = match verify::run(workspace_root, Some(std::slice::from_ref(&target_path))) {
        Ok(report) => report
            .issues
            .into_iter()
            .filter(|issue| {
                issue
                    .file
                    .as_deref()
                    .is_some_and(|f| same_file(workspace_root, f, &target_path))
            })
            .map(|issue| {
                let line0 = issue.line.unwrap_or(1).saturating_sub(1);
                let severity = match issue.severity {
                    IssueSeverity::Error => 1,
                    IssueSeverity::Warning => 2,
                };
                json!({
                    "range": {
                        "start": { "line": line0, "character": 0 },
                        "end": { "line": line0, "character": 1000 }
                    },
                    "severity": severity,
                    "code": issue.code,
                    "source": "clrinf",
                    "message": issue.message
                })
            })
            .collect::<Vec<_>>(),
        Err(_) => Vec::new(),
    };

    Some(json!({
        "jsonrpc": "2.0",
        "method": "textDocument/publishDiagnostics",
        "params": {
            "uri": uri,
            "diagnostics": diagnostics
        }
    }))
}

pub fn run_stdio(default_root: &Path) -> Result<()> {
    let stdin = std::io::stdin();
    let stdout = std::io::stdout();
    run_io(default_root, stdin.lock(), stdout.lock())
}

pub fn run_io<R: BufRead, W: Write>(
    default_root: &Path,
    mut reader: R,
    mut writer: W,
) -> Result<()> {
    let mut workspace_root = default_root.to_path_buf();
    let mut shutdown_requested = false;

    while let Some(msg) = read_message(&mut reader)? {
        let method = msg.get("method").and_then(|m| m.as_str()).unwrap_or("");
        let id = msg.get("id").cloned();

        match method {
            "initialize" => {
                if let Some(init_path) = msg
                    .pointer("/params/rootUri")
                    .and_then(|v| v.as_str())
                    .and_then(uri_to_path)
                    .or_else(|| {
                        msg.pointer("/params/workspaceFolders/0/uri")
                            .and_then(|v| v.as_str())
                            .and_then(uri_to_path)
                    })
                    .or_else(|| {
                        msg.pointer("/params/rootPath")
                            .and_then(|v| v.as_str())
                            .map(PathBuf::from)
                    })
                {
                    if init_path.exists() {
                        workspace_root = init_path;
                    }
                }
                if let Some(req_id) = id {
                    write_message(
                        &mut writer,
                        &json!({
                            "jsonrpc": "2.0",
                            "id": req_id,
                            "result": {
                                "capabilities": {
                                    "textDocumentSync": {
                                        "openClose": true,
                                        "change": 0,
                                        "save": { "includeText": false }
                                    }
                                },
                                "serverInfo": {
                                    "name": "clrinf-lsp",
                                    "version": env!("CARGO_PKG_VERSION")
                                }
                            }
                        }),
                    )?;
                }
            }
            "initialized" => {}
            "shutdown" => {
                shutdown_requested = true;
                if let Some(req_id) = id {
                    write_message(
                        &mut writer,
                        &json!({
                            "jsonrpc": "2.0",
                            "id": req_id,
                            "result": Value::Null
                        }),
                    )?;
                }
            }
            "exit" => break,
            "textDocument/didOpen" | "textDocument/didSave" => {
                if let Some(uri) = msg.pointer("/params/textDocument/uri").and_then(|v| v.as_str()) {
                    if let Some(notif) = publish_diagnostics_for(&workspace_root, uri) {
                        write_message(&mut writer, &notif)?;
                    }
                }
            }
            "textDocument/didClose" => {
                if let Some(uri) = msg.pointer("/params/textDocument/uri").and_then(|v| v.as_str()) {
                    write_message(
                        &mut writer,
                        &json!({
                            "jsonrpc": "2.0",
                            "method": "textDocument/publishDiagnostics",
                            "params": {
                                "uri": uri,
                                "diagnostics": []
                            }
                        }),
                    )?;
                }
            }
            _ => {
                if let Some(req_id) = id {
                    let code = if shutdown_requested { -32600 } else { -32601 };
                    write_message(
                        &mut writer,
                        &json!({
                            "jsonrpc": "2.0",
                            "id": req_id,
                            "error": {
                                "code": code,
                                "message": format!("Method not found: {method}")
                            }
                        }),
                    )?;
                }
            }
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    fn frame(msg: &Value) -> Vec<u8> {
        let mut out = Vec::new();
        write_message(&mut out, msg).unwrap();
        out
    }

    fn path_to_uri(path: &Path) -> String {
        let s = path.to_string_lossy();
        let trimmed = s.strip_prefix(r"\\?\").unwrap_or(&s).replace('\\', "/");
        if trimmed.starts_with('/') {
            format!("file://{trimmed}")
        } else {
            format!("file:///{trimmed}")
        }
    }

    #[test]
    fn uri_to_path_decodes_windows_and_unix_uris() {
        assert_eq!(
            uri_to_path("file:///C:/repo/src/order.rs"),
            Some(PathBuf::from("C:/repo/src/order.rs"))
        );
        assert_eq!(
            uri_to_path("file:///c%3A/my%20repo/file.cs"),
            Some(PathBuf::from("c:/my repo/file.cs"))
        );
        assert_eq!(
            uri_to_path("file:///workspace/src/main.rs"),
            Some(PathBuf::from("/workspace/src/main.rs"))
        );
        assert_eq!(uri_to_path("http://example.com"), None);
    }

    #[test]
    fn lsp_publishes_diagnostics_on_open_and_clears_on_close() {
        let dir = std::env::temp_dir().join(format!("clrinf-lsp-test-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(dir.join("src/domain")).unwrap();
        std::fs::write(
            dir.join("clrinf.rules.toml"),
            r#"[[forbid]]
id = "domain-no-infra"
scope = "imports"
paths = ["src/domain"]
pattern = "Infrastructure"
reason = "Domain must stay clean"
"#,
        )
        .unwrap();
        let bad_file = dir.join("src/domain/bad.cs");
        std::fs::write(&bad_file, "using App.Infrastructure.Db;\npublic class Bad {}\n").unwrap();

        let root_uri = path_to_uri(&dir);
        let file_uri = path_to_uri(&bad_file);

        let mut input = Vec::new();
        input.extend(frame(&json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "initialize",
            "params": { "rootUri": root_uri }
        })));
        input.extend(frame(&json!({
            "jsonrpc": "2.0",
            "method": "initialized",
            "params": {}
        })));
        input.extend(frame(&json!({
            "jsonrpc": "2.0",
            "method": "textDocument/didOpen",
            "params": {
                "textDocument": { "uri": file_uri, "languageId": "csharp", "version": 1, "text": "" }
            }
        })));
        input.extend(frame(&json!({
            "jsonrpc": "2.0",
            "method": "textDocument/didClose",
            "params": { "textDocument": { "uri": file_uri } }
        })));
        input.extend(frame(&json!({
            "jsonrpc": "2.0",
            "id": 2,
            "method": "shutdown"
        })));
        input.extend(frame(&json!({
            "jsonrpc": "2.0",
            "method": "exit"
        })));

        let mut output = Vec::new();
        run_io(&dir, Cursor::new(input), &mut output).unwrap();

        let mut out_cursor = Cursor::new(output);
        let mut messages = Vec::new();
        while let Some(m) = read_message(&mut out_cursor).unwrap() {
            messages.push(m);
        }

        assert_eq!(messages.len(), 4);
        assert_eq!(messages[0]["id"], 1);
        assert_eq!(messages[1]["method"], "textDocument/publishDiagnostics");
        let diags = messages[1]["params"]["diagnostics"].as_array().unwrap();
        assert_eq!(diags.len(), 1);
        assert_eq!(diags[0]["code"], "RULE_FORBIDDEN_DEPENDENCY");
        assert_eq!(diags[0]["severity"], 1);
        assert_eq!(diags[0]["range"]["start"]["line"], 0);

        assert_eq!(messages[2]["method"], "textDocument/publishDiagnostics");
        assert_eq!(messages[2]["params"]["diagnostics"].as_array().unwrap().len(), 0);
        assert_eq!(messages[3]["id"], 2);

        let _ = std::fs::remove_dir_all(dir);
    }
}
