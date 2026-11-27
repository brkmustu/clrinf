//! Language-aware lexical and structural analysis for Rust, C#, TypeScript, and Elixir.
//!
//! Provides exact comment/string stripping (preserving line numbers) and structural
//! import statement extraction without requiring external language toolchains.

use std::path::Path;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SourceLang {
    Rust,
    CSharp,
    TypeScript,
    Elixir,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ImportStmt {
    pub line: usize,
    pub raw: String,
    pub target: String,
}

/// Detect supported language by file extension.
pub fn detect_lang(path: &Path) -> Option<SourceLang> {
    let ext = path.extension()?.to_str()?.to_ascii_lowercase();
    match ext.as_str() {
        "rs" => Some(SourceLang::Rust),
        "cs" => Some(SourceLang::CSharp),
        "ts" | "tsx" | "mts" | "cts" => Some(SourceLang::TypeScript),
        "ex" | "exs" => Some(SourceLang::Elixir),
        _ => None,
    }
}

/// Strip comments AND string/char literal contents, preserving line numbers (`\n` kept intact).
pub fn strip_comments_and_strings(lang: SourceLang, src: &str) -> String {
    scan_lex(lang, src, true)
}

/// Strip comments only (keeping string literal contents intact), preserving line numbers.
pub fn strip_comments_only(lang: SourceLang, src: &str) -> String {
    scan_lex(lang, src, false)
}

fn push_masked_slice(out: &mut String, slice: &str) {
    for ch in slice.chars() {
        if ch == '\n' {
            out.push('\n');
        } else {
            out.push(' ');
        }
    }
}

fn push_span(out: &mut String, slice: &str, mask: bool) {
    if mask {
        push_masked_slice(out, slice);
    } else {
        out.push_str(slice);
    }
}

fn is_ident_byte(b: u8) -> bool {
    b.is_ascii_alphanumeric() || b == b'_'
}

fn scan_quoted(bytes: &[u8], start: usize, quote: u8) -> usize {
    let mut i = start;
    while i < bytes.len() {
        if bytes[i] == b'\\' {
            i = (i + 2).min(bytes.len());
            continue;
        }
        if bytes[i] == quote {
            return i + 1;
        }
        i += 1;
    }
    bytes.len()
}

fn scan_triple_quoted(bytes: &[u8], start: usize, quote: u8) -> usize {
    let mut i = start;
    while i < bytes.len() {
        if bytes[i] == b'\\' {
            i = (i + 2).min(bytes.len());
            continue;
        }
        if i + 2 < bytes.len()
            && bytes[i] == quote
            && bytes[i + 1] == quote
            && bytes[i + 2] == quote
        {
            return i + 3;
        }
        i += 1;
    }
    bytes.len()
}

/// Detect whether `'` at `idx` in Rust starts a char literal (e.g. `'a'`, `'\n'`)
/// rather than a lifetime (`'a`, `'static`). Returns end byte index after closing `'`.
fn rust_char_literal_end(src: &str, idx: usize) -> Option<usize> {
    let rest = &src[idx + 1..];
    let mut chars = rest.chars();
    let first = chars.next()?;
    if first == '\n' || first == '\r' || first == '\'' {
        return None;
    }
    if first == '\\' {
        let mut offset = 1 + 1; // opening '\'' + '\\'
        for ch in rest[1..].chars() {
            if ch == '\n' || ch == '\r' || offset > 16 {
                return None;
            }
            offset += ch.len_utf8();
            if ch == '\'' {
                return Some(idx + offset);
            }
        }
        None
    } else {
        let second = chars.next()?;
        if second == '\'' {
            Some(idx + 1 + first.len_utf8() + 1)
        } else {
            None
        }
    }
}

/// Try matching a Rust raw string (`r"..."`, `r#"..."#`, `br#"..."#`, `cr"..."`) at `idx`.
/// Returns `(content_start_after_prefix_and_quote, end_idx)` if matched.
fn rust_raw_string_end(bytes: &[u8], idx: usize) -> Option<usize> {
    if idx > 0 && is_ident_byte(bytes[idx - 1]) {
        return None;
    }
    let mut pos = idx;
    if (bytes[pos] == b'b' || bytes[pos] == b'c') && pos + 1 < bytes.len() && bytes[pos + 1] == b'r'
    {
        pos += 2;
    } else if bytes[pos] == b'r' {
        pos += 1;
    } else {
        return None;
    }
    let mut hashes = 0usize;
    while pos < bytes.len() && bytes[pos] == b'#' {
        hashes += 1;
        pos += 1;
    }
    if pos >= bytes.len() || bytes[pos] != b'"' {
        return None;
    }
    pos += 1;
    while pos < bytes.len() {
        if bytes[pos] == b'"' {
            let mut matched = 0usize;
            while matched < hashes
                && pos + 1 + matched < bytes.len()
                && bytes[pos + 1 + matched] == b'#'
            {
                matched += 1;
            }
            if matched == hashes {
                return Some(pos + 1 + hashes);
            }
        }
        pos += 1;
    }
    Some(bytes.len())
}

fn csharp_raw_string_end(bytes: &[u8], idx: usize) -> Option<usize> {
    let mut pos = idx;
    while pos < bytes.len() && bytes[pos] == b'$' {
        pos += 1;
    }
    let quote_start = pos;
    while pos < bytes.len() && bytes[pos] == b'"' {
        pos += 1;
    }
    let quotes = pos - quote_start;
    if quotes < 3 {
        return None;
    }
    while pos < bytes.len() {
        if bytes[pos] == b'"' {
            let mut run = 0usize;
            while pos + run < bytes.len() && bytes[pos + run] == b'"' {
                run += 1;
            }
            if run >= quotes {
                return Some(pos + quotes);
            }
            pos += run;
            continue;
        }
        pos += 1;
    }
    Some(bytes.len())
}

fn csharp_verbatim_string_end(bytes: &[u8], idx: usize) -> Option<usize> {
    let quote_pos = if bytes.get(idx..idx + 2) == Some(b"@\"") {
        idx + 1
    } else if bytes.get(idx..idx + 3) == Some(b"$@\"")
        || bytes.get(idx..idx + 3) == Some(b"@$\"")
    {
        idx + 2
    } else {
        return None;
    };
    let mut pos = quote_pos + 1;
    while pos < bytes.len() {
        if bytes[pos] == b'"' {
            if pos + 1 < bytes.len() && bytes[pos + 1] == b'"' {
                pos += 2;
                continue;
            }
            return Some(pos + 1);
        }
        pos += 1;
    }
    Some(bytes.len())
}

fn scan_lex(lang: SourceLang, src: &str, mask_strings: bool) -> String {
    let bytes = src.as_bytes();
    let mut out = String::with_capacity(src.len());
    let mut i = 0usize;

    while i < bytes.len() {
        match lang {
            SourceLang::Rust => {
                if bytes[i] == b'/' && i + 1 < bytes.len() && bytes[i + 1] == b'/' {
                    let end = bytes[i..]
                        .iter()
                        .position(|&b| b == b'\n')
                        .map_or(bytes.len(), |p| i + p);
                    push_masked_slice(&mut out, &src[i..end]);
                    i = end;
                    continue;
                }
                if bytes[i] == b'/' && i + 1 < bytes.len() && bytes[i + 1] == b'*' {
                    let start = i;
                    i += 2;
                    let mut depth = 1usize;
                    while i < bytes.len() && depth > 0 {
                        if bytes[i] == b'/' && i + 1 < bytes.len() && bytes[i + 1] == b'*' {
                            depth += 1;
                            i += 2;
                        } else if bytes[i] == b'*' && i + 1 < bytes.len() && bytes[i + 1] == b'/' {
                            depth -= 1;
                            i += 2;
                        } else {
                            i += 1;
                        }
                    }
                    push_masked_slice(&mut out, &src[start..i]);
                    continue;
                }
                if let Some(end) = rust_raw_string_end(bytes, i) {
                    push_span(&mut out, &src[i..end], mask_strings);
                    i = end;
                    continue;
                }
                if bytes[i] == b'b'
                    && i + 1 < bytes.len()
                    && bytes[i + 1] == b'"'
                    && (i == 0 || !is_ident_byte(bytes[i - 1]))
                {
                    let end = scan_quoted(bytes, i + 2, b'"');
                    push_span(&mut out, &src[i..end], mask_strings);
                    i = end;
                    continue;
                }
                if bytes[i] == b'"' {
                    let end = scan_quoted(bytes, i + 1, b'"');
                    push_span(&mut out, &src[i..end], mask_strings);
                    i = end;
                    continue;
                }
                if bytes[i] == b'\'' {
                    if let Some(end) = rust_char_literal_end(src, i) {
                        push_span(&mut out, &src[i..end], mask_strings);
                        i = end;
                        continue;
                    }
                }
            }
            SourceLang::CSharp => {
                if bytes[i] == b'/' && i + 1 < bytes.len() && bytes[i + 1] == b'/' {
                    let end = bytes[i..]
                        .iter()
                        .position(|&b| b == b'\n')
                        .map_or(bytes.len(), |p| i + p);
                    push_masked_slice(&mut out, &src[i..end]);
                    i = end;
                    continue;
                }
                if bytes[i] == b'/' && i + 1 < bytes.len() && bytes[i + 1] == b'*' {
                    let start = i;
                    i += 2;
                    while i < bytes.len() {
                        if bytes[i] == b'*' && i + 1 < bytes.len() && bytes[i + 1] == b'/' {
                            i += 2;
                            break;
                        }
                        i += 1;
                    }
                    push_masked_slice(&mut out, &src[start..i]);
                    continue;
                }
                if let Some(end) = csharp_raw_string_end(bytes, i) {
                    push_span(&mut out, &src[i..end], mask_strings);
                    i = end;
                    continue;
                }
                if let Some(end) = csharp_verbatim_string_end(bytes, i) {
                    push_span(&mut out, &src[i..end], mask_strings);
                    i = end;
                    continue;
                }
                if bytes[i] == b'$' && i + 1 < bytes.len() && bytes[i + 1] == b'"' {
                    let end = scan_quoted(bytes, i + 2, b'"');
                    push_span(&mut out, &src[i..end], mask_strings);
                    i = end;
                    continue;
                }
                if bytes[i] == b'"' {
                    let end = scan_quoted(bytes, i + 1, b'"');
                    push_span(&mut out, &src[i..end], mask_strings);
                    i = end;
                    continue;
                }
                if bytes[i] == b'\'' {
                    let end = scan_quoted(bytes, i + 1, b'\'');
                    push_span(&mut out, &src[i..end], mask_strings);
                    i = end;
                    continue;
                }
            }
            SourceLang::TypeScript => {
                if bytes[i] == b'/' && i + 1 < bytes.len() && bytes[i + 1] == b'/' {
                    let end = bytes[i..]
                        .iter()
                        .position(|&b| b == b'\n')
                        .map_or(bytes.len(), |p| i + p);
                    push_masked_slice(&mut out, &src[i..end]);
                    i = end;
                    continue;
                }
                if bytes[i] == b'/' && i + 1 < bytes.len() && bytes[i + 1] == b'*' {
                    let start = i;
                    i += 2;
                    while i < bytes.len() {
                        if bytes[i] == b'*' && i + 1 < bytes.len() && bytes[i + 1] == b'/' {
                            i += 2;
                            break;
                        }
                        i += 1;
                    }
                    push_masked_slice(&mut out, &src[start..i]);
                    continue;
                }
                if bytes[i] == b'"' || bytes[i] == b'\'' || bytes[i] == b'`' {
                    let q = bytes[i];
                    let end = scan_quoted(bytes, i + 1, q);
                    push_span(&mut out, &src[i..end], mask_strings);
                    i = end;
                    continue;
                }
            }
            SourceLang::Elixir => {
                if bytes[i] == b'#' {
                    let end = bytes[i..]
                        .iter()
                        .position(|&b| b == b'\n')
                        .map_or(bytes.len(), |p| i + p);
                    push_masked_slice(&mut out, &src[i..end]);
                    i = end;
                    continue;
                }
                if bytes.get(i..i + 3) == Some(b"\"\"\"") {
                    let end = scan_triple_quoted(bytes, i + 3, b'"');
                    push_span(&mut out, &src[i..end], mask_strings);
                    i = end;
                    continue;
                }
                if bytes.get(i..i + 3) == Some(b"'''") {
                    let end = scan_triple_quoted(bytes, i + 3, b'\'');
                    push_span(&mut out, &src[i..end], mask_strings);
                    i = end;
                    continue;
                }
                if bytes[i] == b'"' || bytes[i] == b'\'' {
                    let q = bytes[i];
                    let end = scan_quoted(bytes, i + 1, q);
                    push_span(&mut out, &src[i..end], mask_strings);
                    i = end;
                    continue;
                }
            }
        }

        let ch = src[i..].chars().next().unwrap();
        out.push(ch);
        i += ch.len_utf8();
    }

    out
}

fn collapse_whitespace(s: &str) -> String {
    s.split_whitespace().collect::<Vec<_>>().join(" ")
}

fn extract_quoted_specifier(s: &str) -> Option<String> {
    for quote in ['\'', '"', '`'] {
        if let Some(start) = s.find(quote) {
            let rest = &s[start + 1..];
            if let Some(end) = rest.find(quote) {
                return Some(rest[..end].to_string());
            }
        }
    }
    None
}

/// Extract structural import/using/alias statements from source code.
pub fn extract_imports(lang: SourceLang, src: &str) -> Vec<ImportStmt> {
    let cleaned = strip_comments_only(lang, src);
    match lang {
        SourceLang::Rust => extract_rust_imports(&cleaned),
        SourceLang::CSharp => extract_csharp_imports(&cleaned),
        SourceLang::TypeScript => extract_ts_imports(&cleaned),
        SourceLang::Elixir => extract_elixir_imports(&cleaned),
    }
}

fn extract_rust_imports(cleaned: &str) -> Vec<ImportStmt> {
    let mut out = Vec::new();
    let mut buf = String::new();
    let mut start_line = 1usize;
    let mut brace_depth = 0usize;

    for (idx, line) in cleaned.lines().enumerate() {
        let line_no = idx + 1;
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        if buf.is_empty() {
            // Skip attributes so `#[allow(...)]` on next line doesn't fuse with `use`
            if trimmed.starts_with("#[") || trimmed.starts_with("#![") {
                continue;
            }
            start_line = line_no;
        }
        if !buf.is_empty() {
            buf.push(' ');
        }
        buf.push_str(trimmed);
        for ch in trimmed.chars() {
            match ch {
                '{' => brace_depth += 1,
                '}' => brace_depth = brace_depth.saturating_sub(1),
                ';' if brace_depth == 0 => {
                    let raw = collapse_whitespace(&buf);
                    if let Some(target) = parse_rust_use(&raw) {
                        out.push(ImportStmt {
                            line: start_line,
                            raw,
                            target,
                        });
                    }
                    buf.clear();
                    break;
                }
                _ => {}
            }
        }
        if brace_depth == 0 && !buf.is_empty() && !looks_like_rust_import_start(&buf) {
            buf.clear();
        }
    }
    out
}

fn looks_like_rust_import_start(s: &str) -> bool {
    let t = strip_rust_vis(s.trim_start());
    t.starts_with("use ") || t.starts_with("extern crate ") || t == "pub"
}

fn strip_rust_vis(s: &str) -> &str {
    let s = s.trim_start();
    if let Some(rest) = s.strip_prefix("pub") {
        let rest = rest.trim_start();
        if rest.starts_with('(') {
            if let Some(close) = rest.find(')') {
                return rest[close + 1..].trim_start();
            }
        }
        if s.starts_with("pub ") || s.starts_with("pub(") {
            return rest;
        }
    }
    s
}

fn parse_rust_use(stmt: &str) -> Option<String> {
    let rest = strip_rust_vis(stmt);
    let target = rest
        .strip_prefix("use ")
        .or_else(|| rest.strip_prefix("extern crate "))?;
    Some(target.trim_end_matches(';').trim().to_string())
}

fn extract_csharp_imports(cleaned: &str) -> Vec<ImportStmt> {
    let mut out = Vec::new();
    let mut buf = String::new();
    let mut start_line = 1usize;

    for (idx, line) in cleaned.lines().enumerate() {
        let line_no = idx + 1;
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        if buf.is_empty() {
            start_line = line_no;
        } else {
            buf.push(' ');
        }
        buf.push_str(trimmed);

        if trimmed.ends_with(';') {
            let raw = collapse_whitespace(&buf);
            buf.clear();
            let after_global = raw.strip_prefix("global ").map(str::trim_start).unwrap_or(&raw);
            let Some(rest) = after_global.strip_prefix("using ") else {
                continue;
            };
            let rest_trimmed = rest.trim_start();
            if rest_trimmed.starts_with('(')
                || rest_trimmed.starts_with("var ")
                || rest_trimmed.starts_with("await ")
                || raw.contains('(')
            {
                continue;
            }
            let target = rest_trimmed
                .strip_prefix("static ")
                .unwrap_or(rest_trimmed)
                .trim_end_matches(';')
                .trim();
            let target = target
                .split_once('=')
                .map_or(target, |(_, rhs)| rhs.trim())
                .to_string();
            out.push(ImportStmt {
                line: start_line,
                raw,
                target,
            });
        } else if trimmed.ends_with('{')
            || trimmed.ends_with('}')
            || (!buf.trim_start().starts_with("using") && !buf.trim_start().starts_with("global"))
        {
            buf.clear();
        }
    }
    out
}

fn extract_ts_imports(cleaned: &str) -> Vec<ImportStmt> {
    let mut out = Vec::new();
    let mut buf = String::new();
    let mut start_line = 1usize;
    let mut collecting = false;

    for (idx, line) in cleaned.lines().enumerate() {
        let line_no = idx + 1;
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }

        if !collecting
            && (trimmed.starts_with("import ")
                || trimmed.starts_with("import{")
                || trimmed.starts_with("export "))
        {
            collecting = true;
            start_line = line_no;
            buf.clear();
        }

        if collecting {
            if !buf.is_empty() {
                buf.push(' ');
            }
            buf.push_str(trimmed);
            let has_from_target = buf.contains(" from ")
                && extract_quoted_specifier(buf.split(" from ").last().unwrap_or("")).is_some();
            let is_side_effect_import = buf.starts_with("import ")
                && !buf.contains(" from ")
                && !buf.contains('{')
                && extract_quoted_specifier(&buf).is_some();

            if trimmed.ends_with(';') || has_from_target || is_side_effect_import {
                let raw = collapse_whitespace(&buf);
                let target = if let Some((_, rhs)) = raw.rsplit_once(" from ") {
                    extract_quoted_specifier(rhs)
                } else if raw.starts_with("import ") {
                    extract_quoted_specifier(&raw)
                } else {
                    None
                };
                if let Some(target) = target {
                    out.push(ImportStmt {
                        line: start_line,
                        raw,
                        target,
                    });
                }
                buf.clear();
                collecting = false;
            }
            continue;
        }

        for kw in ["require(", "import("] {
            let mut search_from = 0usize;
            while let Some(pos) = trimmed[search_from..].find(kw) {
                let abs = search_from + pos;
                let after = &trimmed[abs + kw.len()..];
                if let Some(target) = after.split(')').next().and_then(extract_quoted_specifier) {
                    out.push(ImportStmt {
                        line: line_no,
                        raw: trimmed.to_string(),
                        target,
                    });
                }
                search_from = abs + kw.len();
            }
        }
    }
    out
}

fn extract_elixir_imports(cleaned: &str) -> Vec<ImportStmt> {
    let mut out = Vec::new();
    for (idx, line) in cleaned.lines().enumerate() {
        let trimmed = line.trim();
        for kw in ["alias ", "import ", "use ", "require "] {
            if let Some(rest) = trimmed.strip_prefix(kw) {
                out.push(ImportStmt {
                    line: idx + 1,
                    raw: trimmed.to_string(),
                    target: rest.trim().to_string(),
                });
                break;
            }
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rust_strips_nested_comments_raw_strings_and_keeps_lifetimes() {
        let src = r##"
/* outer /* nested HiddenInfra */ still comment */
use crate::orders::PlaceOrder;
use crate::Infrastructure::{
    DbPool,
    Repo,
};
fn foo<'a>(x: &'a str) -> &'a str {
    let s = r#"HiddenInfra"#;
    let c = 'a';
    // HiddenInfra comment
    x
}
"##;
        let code = strip_comments_and_strings(SourceLang::Rust, src);
        assert_eq!(code.lines().count(), src.lines().count());
        assert!(!code.contains("HiddenInfra"));
        assert!(code.contains("fn foo<'a>(x: &'a str)"));

        let imports = extract_imports(SourceLang::Rust, src);
        assert_eq!(imports.len(), 2);
        assert_eq!(imports[0].target, "crate::orders::PlaceOrder");
        assert!(imports[1].target.contains("Infrastructure"));
        assert_eq!(imports[1].line, 4);
    }

    #[test]
    fn csharp_distinguishes_using_directives_from_resource_usings_and_verbatim_strings() {
        let src = r#"
using System;
using MyApp.Infrastructure.Data;
// using Fake.Infrastructure;
public class Worker {
    public void Run() {
        var text = @"MyApp.Infrastructure";
        using var conn = Open();
        using (var tx = conn.Begin()) { }
    }
}
"#;
        let code = strip_comments_and_strings(SourceLang::CSharp, src);
        assert!(!code.contains("Fake.Infrastructure"));
        assert!(!code.contains("MyApp.Infrastructure\""));
        assert!(code.contains("using var conn = Open();"));

        let imports = extract_imports(SourceLang::CSharp, src);
        assert_eq!(imports.len(), 2);
        assert_eq!(imports[0].target, "System");
        assert_eq!(imports[1].target, "MyApp.Infrastructure.Data");
    }

    #[test]
    fn typescript_extracts_multiline_imports_and_ignores_comments_and_templates() {
        let src = r#"
import {
  OrderRepository
} from "../Infrastructure/db";
// import { Bad } from "../Infrastructure/bad";
const msg = `hello ${name} Infrastructure`;
const lazy = () => require("../Infrastructure/legacy");
"#;
        let code = strip_comments_and_strings(SourceLang::TypeScript, src);
        assert!(!code.contains("Infrastructure"));

        let imports = extract_imports(SourceLang::TypeScript, src);
        assert_eq!(imports.len(), 2);
        assert_eq!(imports[0].target, "../Infrastructure/db");
        assert_eq!(imports[0].line, 2);
        assert_eq!(imports[1].target, "../Infrastructure/legacy");
        assert_eq!(imports[1].line, 7);
    }

    #[test]
    fn elixir_handles_comments_heredocs_and_aliases() {
        let src = r#"
defmodule Orders.Domain do
  @doc """
  Mentions Infrastructure in docs.
  """
  alias MyApp.Infrastructure.Repo
  # alias MyApp.Infrastructure.Fake
end
"#;
        let code = strip_comments_and_strings(SourceLang::Elixir, src);
        assert!(!code.contains("docs."));
        assert!(!code.contains("Fake"));

        let imports = extract_imports(SourceLang::Elixir, src);
        assert_eq!(imports.len(), 1);
        assert_eq!(imports[0].target, "MyApp.Infrastructure.Repo");
        assert_eq!(imports[0].line, 6);
    }
}
