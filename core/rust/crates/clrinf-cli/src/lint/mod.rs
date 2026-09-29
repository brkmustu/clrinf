use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;
use syn::visit::Visit;
use syn::{ItemUse, UseTree};
use walkdir::WalkDir;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ArchViolation {
    pub rule_id: String,
    pub severity: String,
    pub message: String,
    pub file_path: String,
    pub line_number: usize,
}

pub struct Linter;

impl Linter {
    pub fn lint_directory(dir: &Path) -> Vec<ArchViolation> {
        let mut violations = Vec::new();

        for entry in WalkDir::new(dir)
            .into_iter()
            .filter_map(Result::ok)
            .filter(|e| e.path().extension().is_some_and(|ext| ext == "rs"))
        {
            let path = entry.path();
            let path_str = path.to_string_lossy();

            // Skip target directory
            if path_str.contains("/target/") || path_str.contains("\\target\\") {
                continue;
            }

            if let Ok(content) = fs::read_to_string(path) {
                Self::lint_file(path, &content, &mut violations);
            }
        }

        violations
    }

    pub fn lint_file(path: &Path, content: &str, violations: &mut Vec<ArchViolation>) {
        let path_str = path.to_string_lossy().replace('\\', "/");
        let is_domain = path_str.contains("-domain/src") || path_str.contains("/domain/");

        let parsed = match syn::parse_file(content) {
            Ok(f) => f,
            Err(_) => return,
        };

        struct UseVisitor<'a> {
            violations: &'a mut Vec<ArchViolation>,
            path_str: String,
            is_domain: bool,
        }

        impl<'ast, 'a> Visit<'ast> for UseVisitor<'a> {
            fn visit_item_use(&mut self, node: &'ast ItemUse) {
                if self.is_domain {
                    let mut path_prefix = String::new();
                    extract_use_prefix(&node.tree, &mut path_prefix);

                    let forbidden = ["axum", "sqlx", "actix", "reqwest", "tokio::net"];
                    for f in forbidden {
                        if path_prefix.starts_with(f) {
                            self.violations.push(ArchViolation {
                                rule_id: "RUST_ARCH001".into(),
                                severity: "Error".into(),
                                message: format!(
                                    "Domain katmanı framework bağımlılığı içeremez: '{path_prefix}'"
                                ),
                                file_path: self.path_str.clone(),
                                line_number: 1, // syn line spans can be resolved if proc_macro2 spans
                            });
                        }
                    }
                }
                syn::visit::visit_item_use(self, node);
            }
        }

        let mut visitor = UseVisitor {
            violations,
            path_str: path_str.clone(),
            is_domain,
        };
        visitor.visit_file(&parsed);
    }
}

fn extract_use_prefix(tree: &UseTree, out: &mut String) {
    match tree {
        UseTree::Path(p) => {
            if !out.is_empty() {
                out.push_str("::");
            }
            out.push_str(&p.ident.to_string());
            extract_use_prefix(&p.tree, out);
        }
        UseTree::Name(n) => {
            if !out.is_empty() {
                out.push_str("::");
            }
            out.push_str(&n.ident.to_string());
        }
        UseTree::Rename(r) => {
            if !out.is_empty() {
                out.push_str("::");
            }
            out.push_str(&r.ident.to_string());
        }
        UseTree::Group(g) => {
            if let Some(first) = g.items.first() {
                extract_use_prefix(first, out);
            }
        }
        UseTree::Glob(_) => {}
    }
}
