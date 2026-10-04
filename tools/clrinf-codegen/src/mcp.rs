use crate::generator::Generator;
use crate::manifest::ProjectManifest;
use crate::module_manager::ModuleManager;
use crate::template::TemplateManager;
use crate::topology::TopologyValidator;
use crate::validator::{check_schemas, validate_schemas};
use crate::worker::{WorkerKind, WorkerManager};
use anyhow::{Context, Result};
use serde_json::{json, Value};
use std::io::{BufRead, Write};
use std::path::{Path, PathBuf};

/// Which MCP tools are exposed. Smaller profiles cut the fixed per-session
/// token cost of tool schemas and reduce tool-selection mistakes in small models.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum McpProfile {
    Full,
    Events,
    Lean,
}

const LEAN_TOOLS: &[&str] = &[
    "clrinf_plan_change",
    "clrinf_verify",
    "clrinf_event_register",
    "clrinf_generate_pubsub",
    "clrinf_scaffold_rule",
    "clrinf_add_entity",
    "clrinf_add_domain_module",
    "clrinf_lint_architecture",
];

const EVENTS_EXTRA_TOOLS: &[&str] = &[
    "clrinf_event_impact",
    "clrinf_topology_drift",
    "clrinf_topology_report",
    "clrinf_validate_topology",
];

impl McpProfile {
    pub fn parse(value: &str) -> Result<Self> {
        match value.trim().to_ascii_lowercase().as_str() {
            "full" => Ok(Self::Full),
            "events" => Ok(Self::Events),
            "lean" => Ok(Self::Lean),
            other => anyhow::bail!("Unknown MCP profile '{other}'; use full, events or lean"),
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            Self::Full => "full",
            Self::Events => "events",
            Self::Lean => "lean",
        }
    }

    pub fn allows(self, tool: &str) -> bool {
        match self {
            Self::Full => true,
            Self::Lean => LEAN_TOOLS.contains(&tool),
            Self::Events => LEAN_TOOLS.contains(&tool) || EVENTS_EXTRA_TOOLS.contains(&tool),
        }
    }
}

pub struct McpServer {
    worker_mgr: WorkerManager,
    workspace_root: Option<PathBuf>,
    profile: McpProfile,
}

impl McpServer {
    pub fn new() -> Self {
        let worker_mgr = WorkerManager::new();
        Self {
            worker_mgr,
            workspace_root: None,
            profile: McpProfile::Full,
        }
    }

    #[allow(dead_code)]
    pub fn with_workspace_root(root: PathBuf) -> Self {
        let worker_mgr = WorkerManager::with_workspace_root(root.clone());
        Self {
            worker_mgr,
            workspace_root: Some(root),
            profile: McpProfile::Full,
        }
    }

    pub fn with_profile(mut self, profile: McpProfile) -> Self {
        self.profile = profile;
        self
    }

    pub fn handle_request(&self, request: &Value) -> Result<Option<Value>> {
        let mut response = self.handle_request_inner(request)?;
        if request.get("method").and_then(|m| m.as_str()) == Some("tools/list") && self.profile != McpProfile::Full {
            if let Some(tools) = response
                .as_mut()
                .and_then(|r| r.pointer_mut("/result/tools"))
                .and_then(|t| t.as_array_mut())
            {
                tools.retain(|t| t["name"].as_str().map_or(false, |n| self.profile.allows(n)));
            }
        }
        Ok(response)
    }

    fn handle_request_inner(&self, request: &Value) -> Result<Option<Value>> {
        let method = request.get("method").and_then(|m| m.as_str()).unwrap_or("");

        // In JSON-RPC 2.0 and MCP specification:
        // Notifications do NOT have an 'id' member. The server MUST NOT reply to a notification.
        // e.g., 'notifications/initialized', 'initialized', 'notifications/cancelled'.
        if request.get("id").is_none() || method.starts_with("notifications/") || method == "initialized" {
            return Ok(None);
        }

        let id = request.get("id").cloned().unwrap_or(Value::Null);

        match method {
            "initialize" => Ok(Some(json!({
                "jsonrpc": "2.0",
                "id": id,
                "result": {
                    "protocolVersion": "2024-11-05",
                    "capabilities": { "tools": {} },
                    "serverInfo": { "name": "clrinf-meta-mcp", "version": "0.1.0" },
                    "instructions": "clrinf architectural framework and codegen tool suite for polyglot systems."
                }
            }))),

            "ping" => Ok(Some(json!({
                "jsonrpc": "2.0",
                "id": id,
                "result": {}
            }))),

            "tools/list" => Ok(Some(json!({
                "jsonrpc": "2.0",
                "id": id,
                "result": {
                    "tools": [
                        {
                            "name": "clrinf_inspect_ecosystem",
                            "description": "Inspects polyglot ecosystem status: C# (Roslyn), Rust (Syn), TypeScript (AST linter), Elixir (AST linter ARCH_EX_*), worker availability, and canonical contracts.",
                            "inputSchema": {
                                "type": "object",
                                "properties": {}
                            }
                        },
                        {
                            "name": "clrinf_lint_architecture",
                            "description": "Runs architectural linter across target language: 'csharp' (Roslyn), 'rust' (Syn), 'typescript' (AST linter), 'elixir' (ARCH_EX_* AST), or 'all'.",
                            "inputSchema": {
                                "type": "object",
                                "properties": {
                                    "lang": { "type": "string", "description": "Target language: csharp, rust, typescript, elixir, or all" },
                                    "path": { "type": "string", "description": "Path to scan (optional)" }
                                }
                            }
                        },
                        {
                            "name": "clrinf_scaffold_rule",
                            "description": "Scaffolds an isolated, testable business rule in C#, Rust, TypeScript, Elixir, or all languages simultaneously.",
                            "inputSchema": {
                                "type": "object",
                                "properties": {
                                    "rule_name": { "type": "string", "description": "Name of the rule (e.g. CheckMaxDiscount)" },
                                    "lang": { "type": "string", "description": "Language target: csharp, rust, typescript, elixir, or all" },
                                    "entity": { "type": "string", "description": "Domain entity or command name (e.g. Order)" },
                                    "error_code": { "type": "string", "description": "Canonical error code (e.g. MAX_DISCOUNT_EXCEEDED)" },
                                    "target_dir": { "type": "string", "description": "Target directory (optional)" }
                                },
                                "required": ["rule_name"]
                            }
                        },
                        {
                            "name": "clrinf_scaffold_otp",
                            "description": "Scaffolds an idiomatic Elixir OTP module consisting of Worker (GenServer), Supervisor, and pure Handler with Result tuple discipline.",
                            "inputSchema": {
                                "type": "object",
                                "properties": {
                                    "module": { "type": "string", "description": "Module name (e.g. OrderProcessor)" },
                                    "app": { "type": "string", "description": "OTP application name (defaults to 'app' or detected project name)" },
                                    "output": { "type": "string", "description": "Destination directory (e.g. lib)" },
                                    "templates_dir": { "type": "string", "description": "Directory containing Tera templates (optional)" }
                                },
                                "required": ["module"]
                            }
                        },
                        {
                            "name": "clrinf_generate_contracts",
                            "description": "Generates idiomatic CloudEvent models and DTOs from JSON Schemas into the target language.",
                            "inputSchema": {
                                "type": "object",
                                "properties": {
                                    "schema_dir": { "type": "string", "description": "Directory containing JSON Schemas" },
                                    "templates_dir": { "type": "string", "description": "Directory containing Tera templates" },
                                    "lang": { "type": "string", "description": "Target language (rust, csharp, typescript, elixir, or all)" },
                                    "output": { "type": "string", "description": "Output directory" }
                                },
                                "required": ["output"]
                            }
                        },
                        {
                            "name": "clrinf_validate_schemas",
                            "description": "Validates JSON Schemas against Draft 2020-12 and verifies unique message identities.",
                            "inputSchema": {
                                "type": "object",
                                "properties": {
                                    "schema_dir": { "type": "string", "description": "Directory containing schemas" }
                                }
                            }
                        },
                        {
                            "name": "clrinf_new_project",
                            "description": "Scaffolds a new service starter natively (C# with clean-cqrs/flat/layered, fp/oop) or from template catalog.",
                            "inputSchema": {
                                "type": "object",
                                "properties": {
                                    "name": { "type": "string", "description": "Name of the new project" },
                                    "lang": { "type": "string", "description": "Target language: csharp, rust, typescript (default: csharp)" },
                                    "template": { "type": "string", "description": "Template name (optional, see template list)" },
                                    "arch": { "type": "string", "description": "Architecture style for C#: clean-cqrs, flat, layered (default: clean-cqrs)" },
                                    "paradigm": { "type": "string", "description": "Paradigm for C#: fp or oop (default: fp)" },
                                    "dispatcher": { "type": "string", "description": "Dispatcher for C#: native or mediatr (default: native)" },
                                    "templates_dir": { "type": "string", "description": "Directory containing templates (optional)" }
                                },
                                "required": ["name"]
                            }
                        },
                        {
                            "name": "clrinf_validate_topology",
                            "description": "Validates cross-service Pub/Sub topology (dead events, orphan subscribers, version gap upcaster chains) and generates an ASCII topology graph.",
                            "inputSchema": {
                                "type": "object",
                                "properties": {
                                    "schema_dir": { "type": "string", "description": "Directory containing JSON Schemas (optional)" },
                                    "upcasters_dir": { "type": "string", "description": "Directory containing YAML upcaster definitions (optional)" }
                                }
                            }
                        },
                        {
                            "name": "clrinf_topology_drift",
                            "description": "Detects drift between declared event topology (x-published-by / x-subscribed-by) and what service source code actually publishes/handles, including raw outbox writes that bypass generated publishers.",
                            "inputSchema": {
                                "type": "object",
                                "properties": {
                                    "schema_dir": { "type": "string", "description": "Directory containing JSON Schemas (optional)" },
                                    "services": { "type": "array", "items": { "type": "string" }, "description": "Service source mappings as NAME=PATH" }
                                },
                                "required": ["services"]
                            }
                        },
                        {
                            "name": "clrinf_event_impact",
                            "description": "Impact analysis for an event: publishers, subscribers, sibling versions, fields and the exact source files affected. Use it to load only the relevant files before changing an event.",
                            "inputSchema": {
                                "type": "object",
                                "properties": {
                                    "event": { "type": "string", "description": "Event type (x-event-type) or schema title" },
                                    "schema_dir": { "type": "string", "description": "Directory containing JSON Schemas (optional)" },
                                    "src": { "type": "array", "items": { "type": "string" }, "description": "Source roots to scan (default: workspace root)" }
                                },
                                "required": ["event"]
                            }
                        },
                        {
                            "name": "clrinf_topology_report",
                            "description": "PR-style topology change report (added/removed events, subscriber changes, breaking field changes, new diagnostics) comparing a base directory or git ref to the current schemas.",
                            "inputSchema": {
                                "type": "object",
                                "properties": {
                                    "schema_dir": { "type": "string", "description": "Head schema directory (optional)" },
                                    "base_dir": { "type": "string", "description": "Base schema directory" },
                                    "base_ref": { "type": "string", "description": "Git ref used as baseline (alternative to base_dir)" },
                                    "upcasters_dir": { "type": "string", "description": "Directory containing YAML upcaster definitions (optional)" }
                                }
                            }
                        },
                        {
                            "name": "clrinf_event_register",
                            "description": "Registers a service as publisher or subscriber of an event in its schema and returns what remains for the agent to implement (business logic only).",
                            "inputSchema": {
                                "type": "object",
                                "properties": {
                                    "event_type": { "type": "string", "description": "x-event-type of the event" },
                                    "service": { "type": "string", "description": "Service name" },
                                    "role": { "type": "string", "description": "'publisher' or 'subscriber'" },
                                    "schema_dir": { "type": "string", "description": "Directory containing JSON Schemas (optional)" }
                                },
                                "required": ["event_type", "service", "role"]
                            }
                        },
                        {
                            "name": "clrinf_plan_change",
                            "description": "Fill-in-the-blanks plan for an event change. Previews (or applies) registering a service as publisher/subscriber and returns only what is left to implement: symbols to fill in, files to read, generated files not to edit, and architecture constraints.",
                            "inputSchema": {
                                "type": "object",
                                "properties": {
                                    "kind": { "type": "string", "description": "'event_subscribe' or 'event_publish'" },
                                    "event_type": { "type": "string", "description": "x-event-type of the event" },
                                    "service": { "type": "string", "description": "Service name" },
                                    "apply": { "type": "boolean", "description": "Register the service in the schema now (default: preview only)" },
                                    "src": { "type": "array", "items": { "type": "string" }, "description": "Source roots to scan (default: workspace root)" },
                                    "schema_dir": { "type": "string", "description": "Directory containing JSON Schemas (optional)" }
                                },
                                "required": ["kind", "event_type", "service"]
                            }
                        },
                        {
                            "name": "clrinf_verify",
                            "description": "Runs project verification (topology, code-vs-contract drift, dependency rules from clrinf.rules.toml). Use after edits; pass 'files' to check only what changed.",
                            "inputSchema": {
                                "type": "object",
                                "properties": {
                                    "files": { "type": "array", "items": { "type": "string" }, "description": "Only verify these files (default: whole project)" },
                                    "path": { "type": "string", "description": "Project root (default: workspace root)" }
                                }
                            }
                        },
                        {
                            "name": "clrinf_generate_pubsub",
                            "description": "Generates CloudEvent publisher envelopes and subscriber shells with idempotency enforcement in Rust, C#, and TypeScript.",
                            "inputSchema": {
                                "type": "object",
                                "properties": {
                                    "schema_dir": { "type": "string", "description": "Directory containing JSON Schemas (optional)" },
                                    "templates_dir": { "type": "string", "description": "Directory containing Tera templates (optional)" },
                                    "lang": { "type": "string", "description": "Target language: rust, csharp, typescript, or all (default: all)" },
                                    "output": { "type": "string", "description": "Output directory" }
                                },
                                "required": ["output"]
                            }
                        },
                        {
                            "name": "clrinf_project_status",
                            "description": "Inspects project manifest (clrinf.toml), language, architecture, enabled cross-cutting modules, and Cedar policies.",
                            "inputSchema": {
                                "type": "object",
                                "properties": {
                                    "path": { "type": "string", "description": "Project directory path (default: current directory)" }
                                }
                            }
                        },
                        {
                            "name": "clrinf_module_manage",
                            "description": "Manages cross-cutting concern modules (caching, logging, transaction, authentication, authorization with Cedar, idempotency, outbox). Supports add, remove, list, and sync actions.",
                            "inputSchema": {
                                "type": "object",
                                "properties": {
                                    "action": { "type": "string", "description": "Action to perform: 'add', 'remove', 'list', or 'sync'" },
                                    "module": { "type": "string", "description": "Module name (e.g. caching, logging, authorization)" },
                                    "provider": { "type": "string", "description": "Provider choice (e.g. cedar, memory, redis, jwt)" },
                                    "path": { "type": "string", "description": "Project directory path (default: current directory)" }
                                },
                                "required": ["action"]
                            }
                        },
                        {
                            "name": "clrinf_add_domain_module",
                            "description": "Scaffolds a new domain feature module with handlers, models, business rules, and multi-tenant Cedar policy (policies/<name>.cedar) using the native language worker.",
                            "inputSchema": {
                                "type": "object",
                                "properties": {
                                    "name": { "type": "string", "description": "Domain module name (e.g. Orders, Catalog)" },
                                    "path": { "type": "string", "description": "Project directory path (default: current directory)" }
                                },
                                "required": ["name"]
                            }
                        },
                        {
                            "name": "clrinf_add_entity",
                            "description": "Scaffolds a CRUD domain entity with repository port, in-memory adapter, and multi-tenant claims.",
                            "inputSchema": {
                                "type": "object",
                                "properties": {
                                    "name": { "type": "string", "description": "Entity name (e.g. OrderItem)" },
                                    "module": { "type": "string", "description": "Parent module name (e.g. Orders)" },
                                    "properties": {
                                        "type": "array",
                                        "items": { "type": "string" },
                                        "description": "Properties in 'name:type' format (e.g. ['name:string', 'price:f64'])"
                                    },
                                    "path": { "type": "string", "description": "Project directory path (default: current directory)" }
                                },
                                "required": ["name", "module"]
                            }
                        },
                        {
                            "name": "clrinf_add_security",
                            "description": "Adds Security module (JWT, Users, Roles, Claims, Password Hashing) and injects DbSets into DbContext.",
                            "inputSchema": {
                                "type": "object",
                                "properties": {
                                    "mode": { "type": "string", "description": "Security mode: 'basic' (JWT + Refresh Token) or 'advanced' (JWT + 2FA/OTP/Email). Default: 'basic'" },
                                    "path": { "type": "string", "description": "Project directory path (default: current directory)" }
                                }
                            }
                        },
                        {
                            "name": "clrinf_adopt_module",
                            "description": "Adopts and transplants any cross-cutting (caching, logging, authz) or domain feature module (e.g. deals, contacts, activities) into an existing target project (.csproj, Cargo.toml, or package.json) in raw (pure self-contained zero-dependency) or wired mode.",
                            "inputSchema": {
                                "type": "object",
                                "properties": {
                                    "module": { "type": "string", "description": "Module name (e.g. deals, contacts, activities, caching, authz, crm)" },
                                    "to_project": { "type": "string", "description": "Target project file (.csproj, Cargo.toml, package.json) or directory" },
                                    "target_dir": { "type": "string", "description": "Optional sub-directory inside target project (e.g. src/Features/Sales)" },
                                    "mode": { "type": "string", "description": "Adoption mode: 'raw' (pure self-contained code) or 'wired' (auto-hooked into module tree/DI). Default: 'raw'" },
                                    "as": { "type": "string", "description": "Optional alias name to rename module (e.g. 'Sales' for 'deals')" },
                                    "source_project": { "type": "string", "description": "Optional path to external source project to transplant from" }
                                },
                                "required": ["module", "to_project"]
                            }
                        },
                        {
                            "name": "clrinf_list_catalog",
                            "description": "Lists official built-in domain features (deals, contacts, activities, crm) and cross-cutting concern modules (caching, logging, authz, etc.) available for adoption into any project in raw or wired mode.",
                            "inputSchema": {
                                "type": "object",
                                "properties": {}
                            }
                        },
                        {
                            "name": "clrinf_get_docs",
                            "description": "Returns token-budgeted, language-isolated architectural rules, contracts, and idioms. Only reads the specified language to conserve LLM context window tokens. Auto-detects language if project_path is provided.",
                            "inputSchema": {
                                "type": "object",
                                "properties": {
                                    "lang": { "type": "string", "description": "Target language: 'csharp', 'rust', 'typescript', 'cedar', 'cli', or 'all'" },
                                    "project_path": { "type": "string", "description": "Optional project path to auto-detect language from .csproj, Cargo.toml, or package.json" }
                                }
                            }
                        },
                        {
                            "name": "clrinf_graft_ask",
                            "description": "Queries the Graft architectural knowledge graph with an intent, returning ranked code spans and crux definitions inlined without reading entire files.",
                            "inputSchema": {
                                "type": "object",
                                "properties": {
                                    "query": { "type": "string", "description": "Natural language query or identifier name (e.g. 'IBusinessRule', 'Cedar authorizer')" },
                                    "in_scope": { "type": "string", "description": "Optional sub-project scope (e.g. 'core/csharp', 'core/rust', 'core/typescript')" }
                                },
                                "required": ["query"]
                            }
                        },
                        {
                            "name": "clrinf_graft_skeleton",
                            "description": "Skims the definition signatures and line spans of a file (~10x cheaper in tokens than reading the whole file).",
                            "inputSchema": {
                                "type": "object",
                                "properties": {
                                    "file": { "type": "string", "description": "Relative or absolute path to the source file" }
                                },
                                "required": ["file"]
                            }
                        }
                    ]
                }
            }))),

            "tools/call" => {
                let params = request.get("params").cloned().unwrap_or(Value::Null);
                let tool_name = params.get("name").and_then(|n| n.as_str()).unwrap_or("");
                let args = params.get("arguments").cloned().unwrap_or(json!({}));

                match self.execute_tool(tool_name, &args) {
                    Ok(result_value) => Ok(Some(json!({
                        "jsonrpc": "2.0",
                        "id": id,
                        "result": {
                            "content": [
                                {
                                    "type": "text",
                                    "text": serde_json::to_string_pretty(&result_value).unwrap_or_else(|_| result_value.to_string())
                                }
                            ]
                        }
                    }))),
                    Err(err) => Ok(Some(json!({
                        "jsonrpc": "2.0",
                        "id": id,
                        "result": {
                            "content": [
                                {
                                    "type": "text",
                                    "text": format!("Error executing {}: {}", tool_name, err)
                                }
                            ],
                            "isError": true
                        }
                    }))),
                }
            }

            _ => Ok(Some(json!({
                "jsonrpc": "2.0",
                "id": id,
                "error": {
                    "code": -32601,
                    "message": format!("Method not found: {}", method)
                }
            }))),
        }
    }

    fn execute_tool(&self, name: &str, args: &Value) -> Result<Value> {
        if !self.profile.allows(name) {
            anyhow::bail!("Tool '{}' is not available in the '{}' MCP profile", name, self.profile.name());
        }
        match name {
            "clrinf_inspect_ecosystem" => {
                let workers = self.worker_mgr.resolve_all_workers();
                Ok(json!({
                    "framework": "clrinf-codegen Meta Orchestrator",
                    "architecture": "Polyglot CloudEvents, Business Rules & Architectural Linters",
                    "workers": workers,
                    "canonicalContracts": {
                        "schemas": ["_context.schema.json", "_error.schema.json", "_envelope.schema.json"],
                        "rules": {
                            "csharp": "IBusinessRule<T> & RulePipeline (Roslyn ARCH001-ARCH004)",
                            "rust": "BusinessRule<T> & RulePipeline (Syn RUST_ARCH001)",
                            "typescript": "Rule<TCtx> & pipeRules, Result<T, E> (TS AST ARCH_TS_001)",
                            "elixir": "Pure Handler Result monad & GenServer (@impl true) (AST ARCH_EX_001-ARCH_EX_004)"
                        }
                    }
                }))
            }

            "clrinf_lint_architecture" => {
                let lang_arg = args.get("lang").and_then(|l| l.as_str()).unwrap_or("all");
                let path_arg = args.get("path").and_then(|p| p.as_str()).map(Path::new);

                let kinds: Vec<WorkerKind> = if lang_arg.eq_ignore_ascii_case("all") {
                    WorkerKind::all().to_vec()
                } else if let Some(k) = WorkerKind::from_str_loose(lang_arg) {
                    vec![k]
                } else {
                    WorkerKind::all().to_vec()
                };

                let mut results = Vec::new();
                for kind in kinds {
                    match self.worker_mgr.lint(kind, path_arg) {
                        Ok(res) => results.push(json!({
                            "worker": kind.as_str(),
                            "success": res.success,
                            "exit_code": res.exit_code,
                            "stdout": res.stdout,
                            "stderr": res.stderr,
                        })),
                        Err(e) => results.push(json!({
                            "worker": kind.as_str(),
                            "success": false,
                            "error": e.to_string(),
                        })),
                    }
                }

                Ok(json!({ "lintResults": results }))
            }

            "clrinf_scaffold_rule" => {
                let rule_name = args.get("rule_name").and_then(|r| r.as_str()).unwrap_or("SampleRule");
                let lang_arg = args.get("lang").and_then(|l| l.as_str()).unwrap_or("all");
                let entity = args.get("entity").and_then(|e| e.as_str());
                let error_code = args.get("error_code").and_then(|e| e.as_str());
                let target_dir = args.get("target_dir").and_then(|t| t.as_str()).map(Path::new);

                let kinds: Vec<WorkerKind> = if lang_arg.eq_ignore_ascii_case("all") {
                    WorkerKind::all().to_vec()
                } else if let Some(k) = WorkerKind::from_str_loose(lang_arg) {
                    vec![k]
                } else {
                    WorkerKind::all().to_vec()
                };

                let mut results = Vec::new();
                for kind in kinds {
                    match self.worker_mgr.scaffold_rule(kind, rule_name, entity, error_code, target_dir) {
                        Ok(res) => results.push(json!({
                            "worker": kind.as_str(),
                            "success": res.success,
                            "stdout": res.stdout,
                            "stderr": res.stderr,
                        })),
                        Err(e) => results.push(json!({
                            "worker": kind.as_str(),
                            "success": false,
                            "error": e.to_string(),
                        })),
                    }
                }

                Ok(json!({ "scaffoldResults": results }))
            }

            "clrinf_scaffold_otp" => {
                let module = args.get("module").and_then(|m| m.as_str()).context("module is required")?;
                let app = args.get("app").and_then(|a| a.as_str()).unwrap_or("app");
                let output_str = args.get("output").and_then(|o| o.as_str()).unwrap_or("lib");
                let output_dir = Path::new(output_str);

                let default_tmpl = self
                    .workspace_root
                    .as_ref()
                    .map(|r| r.join("tools/clrinf-codegen/templates"))
                    .unwrap_or_else(|| PathBuf::from("./tools/clrinf-codegen/templates"));
                let tmpl_dir = args
                    .get("templates_dir")
                    .and_then(|s| s.as_str())
                    .map(Path::new)
                    .unwrap_or(&default_tmpl);

                let gen = Generator::new(tmpl_dir)?;
                let created = gen.scaffold_otp(module, app, output_dir)?;
                let created_paths: Vec<String> = created.iter().map(|p| p.display().to_string()).collect();

                Ok(json!({
                    "success": true,
                    "module": module,
                    "app": app,
                    "createdFiles": created_paths
                }))
            }

            "clrinf_generate_contracts" => {
                let default_schema = self.workspace_root.as_ref()
                    .map(|r| r.join("tools/clrinf-codegen/schemas"))
                    .unwrap_or_else(|| PathBuf::from("./tools/clrinf-codegen/schemas"));
                let default_tmpl = self.workspace_root.as_ref()
                    .map(|r| r.join("tools/clrinf-codegen/templates"))
                    .unwrap_or_else(|| PathBuf::from("./tools/clrinf-codegen/templates"));

                let schema_dir = args.get("schema_dir").and_then(|s| s.as_str()).map(Path::new).unwrap_or(&default_schema);
                let tmpl_dir = args.get("templates_dir").and_then(|s| s.as_str()).map(Path::new).unwrap_or(&default_tmpl);
                let lang_arg = args.get("lang").and_then(|l| l.as_str()).unwrap_or("all");
                let out_dir_str = args.get("output").and_then(|o| o.as_str()).unwrap_or("./generated");
                let out_dir = Path::new(out_dir_str);

                let gen = Generator::new(tmpl_dir)?;
                if lang_arg.eq_ignore_ascii_case("all") {
                    let langs = ["rust", "csharp", "typescript", "elixir"];
                    let mut generated = Vec::new();
                    for l in &langs {
                        let target_out = out_dir.join(l);
                        let count = gen.generate(schema_dir, l, &target_out)?;
                        generated.push(json!({ "lang": l, "count": count, "dir": target_out.display().to_string() }));
                    }
                    Ok(json!({ "success": true, "targets": generated }))
                } else {
                    let count = gen.generate(schema_dir, lang_arg, out_dir)?;
                    Ok(json!({ "success": true, "lang": lang_arg, "count": count, "dir": out_dir.display().to_string() }))
                }
            }

            "clrinf_validate_schemas" => {
                let default_schema = self.workspace_root.as_ref()
                    .map(|r| r.join("tools/clrinf-codegen/schemas"))
                    .unwrap_or_else(|| PathBuf::from("./tools/clrinf-codegen/schemas"));
                let schema_dir = args.get("schema_dir").and_then(|s| s.as_str()).map(Path::new).unwrap_or(&default_schema);
                validate_schemas(schema_dir)?;
                check_schemas(schema_dir)?;
                Ok(json!({ "status": "PASS", "message": "Schemas successfully validated against Draft 2020-12" }))
            }

            "clrinf_new_project" => {
                let name = args.get("name").and_then(|n| n.as_str()).unwrap_or("my-service");
                let tmpl = args.get("template").and_then(|t| t.as_str());
                let lang = args.get("lang").and_then(|l| l.as_str()).unwrap_or("csharp");

                if let Some(template_name) = tmpl {
                    let default_tmpl = self.workspace_root.as_ref()
                        .map(|r| r.join("templates"))
                        .unwrap_or_else(|| PathBuf::from("./templates"));
                    let tmpl_dir = args.get("templates_dir").and_then(|t| t.as_str()).map(Path::new).unwrap_or(&default_tmpl);

                    let mgr = TemplateManager::new(tmpl_dir);
                    let target_dir = Path::new(name);
                    mgr.scaffold(template_name, target_dir)?;
                    Ok(json!({
                        "status": "SUCCESS",
                        "project": name,
                        "template": template_name,
                        "directory": target_dir.display().to_string()
                    }))
                } else if lang.eq_ignore_ascii_case("csharp") {
                    let arch_str = args.get("arch").and_then(|a| a.as_str()).unwrap_or("clean-cqrs");
                    let arch_style = match arch_str.to_lowercase().as_str() {
                        "flat" => crate::manifest::ArchStyle::Flat,
                        "layered" => crate::manifest::ArchStyle::Layered,
                        _ => crate::manifest::ArchStyle::CleanCqrs,
                    };
                    let disp_str = args.get("dispatcher").and_then(|d| d.as_str()).unwrap_or("native");
                    let disp = match disp_str.to_lowercase().as_str() {
                        "mediatr" => crate::manifest::Dispatcher::Mediatr,
                        "mediatornet" => crate::manifest::Dispatcher::MediatorNet,
                        _ => crate::manifest::Dispatcher::Native,
                    };
                    let paradigm = args.get("paradigm").and_then(|p| p.as_str()).unwrap_or("fp");
                    let target_dir = PathBuf::from(name);
                    let parent_dir = target_dir.parent().unwrap_or_else(|| Path::new("."));
                    let project_name = target_dir.file_name().and_then(|s| s.to_str()).unwrap_or(name);

                    let opts = crate::csharp::project::CreateProjectOptions {
                        name: project_name,
                        target_path: parent_dir,
                        arch: arch_style,
                        host_type: crate::manifest::HostType::Api,
                        dispatcher: disp,
                        paradigm,
                        profile: "standard",
                        db_context: None,
                        skip_dotnet_exec: false,
                    };

                    let res = crate::csharp::project::create_project(&opts)?;
                    Ok(json!({
                        "status": "SUCCESS",
                        "project": project_name,
                        "language": "csharp",
                        "architecture": arch_str,
                        "paradigm": paradigm,
                        "dispatcher": disp_str,
                        "directory": res.project_dir.display().to_string(),
                        "files_created": res.files_created.iter().map(|f| f.display().to_string()).collect::<Vec<_>>()
                    }))
                } else {
                    anyhow::bail!("Unsupported language '{}'. Specify 'template' to scaffold from templates.", lang);
                }
            }

            "clrinf_validate_topology" => {
                let default_schema = self.workspace_root.as_ref()
                    .map(|r| r.join("tools/clrinf-codegen/schemas"))
                    .unwrap_or_else(|| PathBuf::from("./tools/clrinf-codegen/schemas"));
                let default_upcasters = self.workspace_root.as_ref()
                    .map(|r| r.join("tools/clrinf-codegen/schemas"))
                    .unwrap_or_else(|| PathBuf::from("./tools/clrinf-codegen/schemas"));

                let schema_dir = args.get("schema_dir").and_then(|s| s.as_str()).map(Path::new).unwrap_or(&default_schema);
                let upcasters_dir = args.get("upcasters_dir").and_then(|s| s.as_str()).map(Path::new).unwrap_or(&default_upcasters);

                let report = TopologyValidator::validate(schema_dir, Some(upcasters_dir))?;
                Ok(json!({
                    "isValid": report.is_valid,
                    "errorsCount": report.errors_count(),
                    "warningsCount": report.warnings_count(),
                    "issues": report.issues,
                    "asciiGraph": report.format_ascii_graph(),
                    "diagnostics": report.format_diagnostics(),
                }))
            }

            "clrinf_topology_drift" => {
                let default_schema = self.workspace_root.as_ref()
                    .map(|r| r.join("tools/clrinf-codegen/schemas"))
                    .unwrap_or_else(|| PathBuf::from("./tools/clrinf-codegen/schemas"));
                let schema_dir = args.get("schema_dir").and_then(|s| s.as_str()).map(Path::new).unwrap_or(&default_schema);
                let specs: Vec<String> = args.get("services").and_then(|s| s.as_array())
                    .map(|a| a.iter().filter_map(|v| v.as_str().map(String::from)).collect())
                    .unwrap_or_default();
                if specs.is_empty() {
                    anyhow::bail!("'services' must contain at least one NAME=PATH entry");
                }
                let sources = specs.iter().map(|s| crate::drift::ServiceSource::parse(s)).collect::<Result<Vec<_>>>()?;
                let report = crate::drift::detect(schema_dir, &sources)?;
                Ok(json!({
                    "isValid": report.is_valid,
                    "errorsCount": report.errors_count(),
                    "warningsCount": report.warnings_count(),
                    "issues": report.issues,
                    "diagnostics": report.format_diagnostics(),
                }))
            }

            "clrinf_event_impact" => {
                let default_schema = self.workspace_root.as_ref()
                    .map(|r| r.join("tools/clrinf-codegen/schemas"))
                    .unwrap_or_else(|| PathBuf::from("./tools/clrinf-codegen/schemas"));
                let schema_dir = args.get("schema_dir").and_then(|s| s.as_str()).map(Path::new).unwrap_or(&default_schema);
                let event = args.get("event").and_then(|s| s.as_str()).context("'event' is required")?;
                let mut roots: Vec<PathBuf> = args.get("src").and_then(|s| s.as_array())
                    .map(|a| a.iter().filter_map(|v| v.as_str().map(PathBuf::from)).collect())
                    .unwrap_or_default();
                if roots.is_empty() {
                    roots.push(self.workspace_root.clone().unwrap_or_else(|| PathBuf::from(".")));
                }
                let report = crate::impact::analyze(schema_dir, event, &roots)?;
                let text = report.format_text();
                let mut value = serde_json::to_value(&report)?;
                value["summary"] = json!(text);
                Ok(value)
            }

            "clrinf_topology_report" => {
                let default_schema = self.workspace_root.as_ref()
                    .map(|r| r.join("tools/clrinf-codegen/schemas"))
                    .unwrap_or_else(|| PathBuf::from("./tools/clrinf-codegen/schemas"));
                let schema_dir = args.get("schema_dir").and_then(|s| s.as_str()).map(Path::new).unwrap_or(&default_schema);
                let upcasters_dir = args.get("upcasters_dir").and_then(|s| s.as_str()).map(Path::new).unwrap_or(schema_dir);
                let (base, cleanup) = match (args.get("base_dir").and_then(|s| s.as_str()), args.get("base_ref").and_then(|s| s.as_str())) {
                    (Some(dir), _) => (PathBuf::from(dir), None),
                    (None, Some(reference)) => {
                        let dir = crate::report::materialize_git_ref(schema_dir, reference)?;
                        (dir.clone(), Some(dir))
                    }
                    (None, None) => anyhow::bail!("Provide 'base_dir' or 'base_ref'"),
                };
                let result = crate::report::compare(&base, schema_dir, Some(upcasters_dir));
                if let Some(dir) = cleanup {
                    let _ = std::fs::remove_dir_all(dir);
                }
                let diff = result?;
                Ok(json!({
                    "hasBreaking": diff.has_breaking(),
                    "markdown": diff.to_markdown(),
                    "diff": diff,
                }))
            }

            "clrinf_event_register" => {
                let default_schema = self.workspace_root.as_ref()
                    .map(|r| r.join("tools/clrinf-codegen/schemas"))
                    .unwrap_or_else(|| PathBuf::from("./tools/clrinf-codegen/schemas"));
                let schema_dir = args.get("schema_dir").and_then(|s| s.as_str()).map(Path::new).unwrap_or(&default_schema);
                let event_type = args.get("event_type").and_then(|s| s.as_str()).context("'event_type' is required")?;
                let service = args.get("service").and_then(|s| s.as_str()).context("'service' is required")?;
                let role = match args.get("role").and_then(|s| s.as_str()) {
                    Some("publisher") => crate::event_scaffold::Role::Publisher,
                    Some("subscriber") => crate::event_scaffold::Role::Subscriber,
                    _ => anyhow::bail!("'role' must be 'publisher' or 'subscriber'"),
                };
                let outcome = crate::event_scaffold::register(schema_dir, event_type, role, service)?;
                Ok(json!({
                    "changed": outcome.changed,
                    "schemaFile": outcome.schema_file.display().to_string(),
                    "title": outcome.title,
                    "next": "Run clrinf_generate_pubsub, then implement only the handler/publish call site business logic.",
                }))
            }

            "clrinf_plan_change" => {
                let root = self.workspace_root.clone().unwrap_or_else(|| PathBuf::from("."));
                let rules = crate::rules::Rules::load(&root)?;
                let default_schema = rules.as_ref().map(|r| r.schema_dir())
                    .unwrap_or_else(|| root.join(crate::rules::DEFAULT_SCHEMA_DIR));
                let schema_dir = args.get("schema_dir").and_then(|s| s.as_str()).map(PathBuf::from).unwrap_or(default_schema);
                let role = match args.get("kind").and_then(|s| s.as_str()) {
                    Some("event_subscribe") => crate::event_scaffold::Role::Subscriber,
                    Some("event_publish") => crate::event_scaffold::Role::Publisher,
                    _ => anyhow::bail!("'kind' must be 'event_subscribe' or 'event_publish'"),
                };
                let event_type = args.get("event_type").and_then(|s| s.as_str()).context("'event_type' is required")?;
                let service = args.get("service").and_then(|s| s.as_str()).context("'service' is required")?;
                let apply = args.get("apply").and_then(|v| v.as_bool()).unwrap_or(false);
                let mut roots: Vec<PathBuf> = args.get("src").and_then(|s| s.as_array())
                    .map(|a| a.iter().filter_map(|v| v.as_str().map(PathBuf::from)).collect())
                    .unwrap_or_default();
                if roots.is_empty() {
                    roots.push(root.clone());
                }
                let plan = crate::plan::plan_event(&schema_dir, event_type, service, role, &roots, apply, rules.as_ref())?;
                Ok(serde_json::to_value(plan)?)
            }

            "clrinf_verify" => {
                let root = args.get("path").and_then(|s| s.as_str()).map(PathBuf::from)
                    .or_else(|| self.workspace_root.clone())
                    .unwrap_or_else(|| PathBuf::from("."));
                let files: Vec<PathBuf> = args.get("files").and_then(|s| s.as_array())
                    .map(|a| a.iter().filter_map(|v| v.as_str()).map(|p| {
                        let p = PathBuf::from(p);
                        if p.is_absolute() { p } else { root.join(p) }
                    }).collect())
                    .unwrap_or_default();
                let only = if files.is_empty() { None } else { Some(files.as_slice()) };
                let report = crate::verify::run(&root, only)?;
                Ok(json!({
                    "isValid": report.is_valid,
                    "errorsCount": report.errors_count(),
                    "checks": report.checks,
                    "issues": report.issues,
                    "diagnostics": report.format_text(false),
                }))
            }

            "clrinf_generate_pubsub" => {
                let default_schema = self.workspace_root.as_ref()
                    .map(|r| r.join("tools/clrinf-codegen/schemas"))
                    .unwrap_or_else(|| PathBuf::from("./tools/clrinf-codegen/schemas"));
                let default_tmpl = self.workspace_root.as_ref()
                    .map(|r| r.join("tools/clrinf-codegen/templates"))
                    .unwrap_or_else(|| PathBuf::from("./tools/clrinf-codegen/templates"));

                let schema_dir = args.get("schema_dir").and_then(|s| s.as_str()).map(Path::new).unwrap_or(&default_schema);
                let tmpl_dir = args.get("templates_dir").and_then(|s| s.as_str()).map(Path::new).unwrap_or(&default_tmpl);
                let lang_arg = args.get("lang").and_then(|l| l.as_str()).unwrap_or("all");
                let out_dir_str = args.get("output").and_then(|o| o.as_str()).unwrap_or("./generated-pubsub");
                let out_dir = Path::new(out_dir_str);

                let gen = Generator::new(tmpl_dir)?;
                if lang_arg.eq_ignore_ascii_case("all") {
                    let langs = ["rust", "csharp", "typescript"];
                    let mut generated = Vec::new();
                    for l in &langs {
                        let target_out = out_dir.join(l);
                        let count = gen.generate_pubsub(schema_dir, l, &target_out)?;
                        generated.push(json!({ "lang": l, "count": count, "dir": target_out.display().to_string() }));
                    }
                    Ok(json!({ "success": true, "targets": generated }))
                } else {
                    let count = gen.generate_pubsub(schema_dir, lang_arg, out_dir)?;
                    Ok(json!({ "success": true, "lang": lang_arg, "count": count, "dir": out_dir.display().to_string() }))
                }
            }

            "clrinf_project_status" => {
                let path_str = args.get("path").and_then(|p| p.as_str()).unwrap_or(".");
                let path = Path::new(path_str);
                let abs_path = std::fs::canonicalize(path).unwrap_or_else(|_| path.to_path_buf());
                let (manifest, manifest_path) = ProjectManifest::discover(&abs_path)?;
                let enabled = manifest.enabled_modules();
                Ok(json!({
                    "manifest_path": manifest_path.display().to_string(),
                    "project_name": manifest.project.name,
                    "language": manifest.project.lang.as_str(),
                    "architecture": format!("{:?}", manifest.project.arch),
                    "deployment": format!("{:?}", manifest.project.deployment),
                    "enabled_modules": enabled,
                    "cedar_policies_dir": manifest.modules.authorization.policy_path,
                }))
            }

            "clrinf_module_manage" => {
                let action = args.get("action").and_then(|a| a.as_str()).unwrap_or("list");
                let path_str = args.get("path").and_then(|p| p.as_str()).unwrap_or(".");
                let path = Path::new(path_str);
                let abs_path = std::fs::canonicalize(path).unwrap_or_else(|_| path.to_path_buf());
                let (mut manifest, manifest_path) = ProjectManifest::discover(&abs_path)?;
                let manager = ModuleManager::new(None)?;

                match action {
                    "add" => {
                        let module = args.get("module").and_then(|m| m.as_str()).ok_or_else(|| {
                            anyhow::anyhow!("'module' argument is required for 'add' action")
                        })?;
                        let provider = args.get("provider").and_then(|p| p.as_str());
                        let created = manager.add_module(&mut manifest, &manifest_path, module, provider)?;
                        Ok(json!({
                            "action": "add",
                            "module": module,
                            "provider": provider,
                            "created_files": created.into_iter().map(|p| p.display().to_string()).collect::<Vec<_>>(),
                            "success": true
                        }))
                    }
                    "remove" => {
                        let module = args.get("module").and_then(|m| m.as_str()).ok_or_else(|| {
                            anyhow::anyhow!("'module' argument is required for 'remove' action")
                        })?;
                        let removed = manager.remove_module(&mut manifest, &manifest_path, module)?;
                        Ok(json!({
                            "action": "remove",
                            "module": module,
                            "removed_files": removed.into_iter().map(|p| p.display().to_string()).collect::<Vec<_>>(),
                            "success": true
                        }))
                    }
                    "sync" => {
                        let report = manager.sync_modules(&manifest, &abs_path)?;
                        Ok(json!({
                            "action": "sync",
                            "synced_modules": report.synced,
                            "cleaned_modules": report.cleaned,
                            "success": true
                        }))
                    }
                    "list" => {
                        let enabled = manifest.enabled_modules();
                        Ok(json!({
                            "action": "list",
                            "enabled_modules": enabled,
                            "caching": manifest.modules.caching.enabled,
                            "logging": manifest.modules.logging.enabled,
                            "transaction": manifest.modules.transaction.enabled,
                            "authentication": manifest.modules.authentication.enabled,
                            "authorization": manifest.modules.authorization.enabled,
                            "idempotency": manifest.modules.idempotency.enabled,
                            "outbox": manifest.modules.outbox.enabled,
                        }))
                    }
                    _ => anyhow::bail!("Unknown action: '{}'. Supported: add, remove, sync, list", action),
                }
            }

            "clrinf_add_domain_module" => {
                let name = args.get("name").and_then(|n| n.as_str()).ok_or_else(|| {
                    anyhow::anyhow!("'name' is required for clrinf_add_domain_module")
                })?;
                let path_str = args.get("path").and_then(|p| p.as_str()).unwrap_or(".");
                let path = Path::new(path_str);
                let abs_path = std::fs::canonicalize(path).unwrap_or_else(|_| path.to_path_buf());
                let (manifest, _) = ProjectManifest::discover(&abs_path)?;

                if manifest.project.lang == crate::manifest::Lang::CSharp {
                    let pattern = args.get("pattern").and_then(|p| p.as_str()).map(|s| match s.to_lowercase().as_str() {
                        "basic" => crate::manifest::PatternStyle::Basic,
                        "separated" => crate::manifest::PatternStyle::Separated,
                        _ => crate::manifest::PatternStyle::Flat,
                    });
                    let opts = crate::csharp::module::AddModuleOptions {
                        module_name: name,
                        project_path: &abs_path,
                        manifest: &manifest,
                        pattern,
                    };
                    let res = crate::csharp::module::add_module(&opts)?;
                    return Ok(json!({
                        "module": name,
                        "language": "csharp",
                        "success": true,
                        "files_created": res.files_created.iter().map(|p| p.to_string_lossy()).collect::<Vec<_>>(),
                        "files_modified": res.files_modified.iter().map(|p| p.to_string_lossy()).collect::<Vec<_>>(),
                    }));
                }

                let kind = match manifest.project.lang {
                    crate::manifest::Lang::CSharp => WorkerKind::CSharp,
                    crate::manifest::Lang::Rust => WorkerKind::Rust,
                    crate::manifest::Lang::TypeScript => WorkerKind::TypeScript,
                    crate::manifest::Lang::Elixir => WorkerKind::Elixir,
                };

                let res = self.worker_mgr.add_module(kind, name, Some(&abs_path))?;
                Ok(json!({
                    "module": name,
                    "language": manifest.project.lang.as_str(),
                    "success": res.success,
                    "stdout": res.stdout,
                    "stderr": res.stderr
                }))
            }

            "clrinf_add_entity" => {
                let name = args.get("name").and_then(|n| n.as_str()).ok_or_else(|| {
                    anyhow::anyhow!("'name' is required for clrinf_add_entity")
                })?;
                let module = args.get("module").and_then(|m| m.as_str()).ok_or_else(|| {
                    anyhow::anyhow!("'module' is required for clrinf_add_entity")
                })?;
                let path_str = args.get("path").and_then(|p| p.as_str()).unwrap_or(".");
                let path = Path::new(path_str);
                let abs_path = std::fs::canonicalize(path).unwrap_or_else(|_| path.to_path_buf());
                let (manifest, _) = ProjectManifest::discover(&abs_path)?;

                let props_val = args.get("properties").and_then(|p| p.as_array());
                let mut parsed_props = Vec::new();
                let default_props = vec![];
                let props_list = props_val.unwrap_or(&default_props);
                for p in props_list {
                    if let Some(s) = p.as_str() {
                        let mut parts = s.splitn(2, ':');
                        let k = parts.next().unwrap_or("");
                        let v = parts.next().unwrap_or("string");
                        parsed_props.push((k, v));
                    }
                }

                if manifest.project.lang == crate::manifest::Lang::CSharp {
                    let templates_dir = if let Some(ref root) = self.worker_mgr.workspace_root() {
                        root.join("tools").join("clrinf-codegen").join("templates")
                    } else {
                        PathBuf::from("./templates")
                    };
                    let opts = crate::csharp::entity::AddEntityOptions {
                        entity_name: name,
                        module_name: module,
                        properties: &parsed_props,
                        id_type: args.get("id_type").and_then(|v| v.as_str()),
                        project_path: &abs_path,
                        manifest: &manifest,
                        templates_dir: &templates_dir,
                    };
                    let res = crate::csharp::entity::add_entity(&opts)?;
                    return Ok(json!({
                        "entity": name,
                        "module": module,
                        "language": "csharp",
                        "pattern": "vertical-slice",
                        "success": true,
                        "files_created": res.files_created.iter().map(|p| p.to_string_lossy()).collect::<Vec<_>>(),
                        "files_modified": res.files_modified.iter().map(|p| p.to_string_lossy()).collect::<Vec<_>>(),
                    }));
                }

                let kind = match manifest.project.lang {
                    crate::manifest::Lang::CSharp => WorkerKind::CSharp,
                    crate::manifest::Lang::Rust => WorkerKind::Rust,
                    crate::manifest::Lang::TypeScript => WorkerKind::TypeScript,
                    crate::manifest::Lang::Elixir => WorkerKind::Elixir,
                };

                let res = self.worker_mgr.add_entity(kind, name, module, &parsed_props, Some(&abs_path))?;
                Ok(json!({
                    "entity": name,
                    "module": module,
                    "language": manifest.project.lang.as_str(),
                    "success": res.success,
                    "stdout": res.stdout,
                    "stderr": res.stderr
                }))
            }

            "clrinf_add_security" => {
                let mode_str = args.get("mode").and_then(|m| m.as_str()).unwrap_or("basic");
                let path_str = args.get("path").and_then(|p| p.as_str()).unwrap_or(".");
                let path = Path::new(path_str);
                let abs_path = std::fs::canonicalize(path).unwrap_or_else(|_| path.to_path_buf());
                let (manifest, _) = ProjectManifest::discover(&abs_path)?;

                let templates_dir = if let Some(ref root) = self.worker_mgr.workspace_root() {
                    root.join("tools").join("clrinf-codegen").join("templates")
                } else {
                    PathBuf::from("./templates")
                };

                let sec_mode = crate::csharp::security::SecurityMode::from_str(mode_str);
                let opts = crate::csharp::security::AddSecurityOptions {
                    project_path: &abs_path,
                    manifest: &manifest,
                    templates_dir: &templates_dir,
                    mode: sec_mode,
                };
                let res = crate::csharp::security::add_security(&opts)?;
                Ok(json!({
                    "mode": mode_str,
                    "language": manifest.project.lang.as_str(),
                    "success": true,
                    "files_created": res.files_created.iter().map(|p| p.to_string_lossy()).collect::<Vec<_>>(),
                    "files_modified": res.files_modified.iter().map(|p| p.to_string_lossy()).collect::<Vec<_>>(),
                }))
            }

            "clrinf_adopt_module" => {
                let module = args.get("module").and_then(|m| m.as_str()).unwrap_or("");
                let to_project_str = args.get("to_project").and_then(|p| p.as_str()).unwrap_or(".");
                let to_project = PathBuf::from(to_project_str);
                let target_dir = args.get("target_dir").and_then(|t| t.as_str()).map(PathBuf::from);
                let mode_str = args.get("mode").and_then(|m| m.as_str()).unwrap_or("raw");
                let alias = args.get("as").and_then(|a| a.as_str()).map(|s| s.to_string());
                let source_project = args.get("source_project").and_then(|s| s.as_str()).map(PathBuf::from);

                let adopt_mode = match mode_str.to_lowercase().as_str() {
                    "wired" | "integrated" | "hooked" => crate::module_adapter::AdoptMode::Wired,
                    _ => crate::module_adapter::AdoptMode::Raw,
                };

                let opts = crate::module_adapter::ModuleAdoptOptions {
                    module: module.to_string(),
                    to_project,
                    target_dir,
                    mode: adopt_mode,
                    r#as: alias,
                    source_project,
                };

                let result = crate::module_adapter::ModuleAdapter::adopt(&opts)?;
                Ok(serde_json::to_value(result)?)
            }

            "clrinf_list_catalog" => {
                let catalog = crate::module_adapter::get_built_in_catalog();
                Ok(serde_json::to_value(catalog)?)
            }

            "clrinf_get_docs" => {
                let lang = args.get("lang").and_then(|l| l.as_str());
                let project_path = args.get("project_path").and_then(|p| p.as_str()).map(Path::new);
                let content = crate::docs_provider::get_language_docs(lang, project_path)?;
                Ok(json!({
                    "language": lang.unwrap_or("auto-detected"),
                    "docs": content
                }))
            }

            "clrinf_graft_ask" => {
                let query = args.get("query").and_then(|q| q.as_str()).unwrap_or("");
                let in_scope = args.get("in_scope").and_then(|s| s.as_str());

                let mut cmd = std::process::Command::new("graft");
                cmd.arg("ask").arg(query).arg("--source");
                if let Some(scope) = in_scope {
                    cmd.arg("--in").arg(scope);
                }

                match cmd.output() {
                    Ok(output) => {
                        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
                        let stderr = String::from_utf8_lossy(&output.stderr).to_string();
                        Ok(json!({
                            "success": output.status.success(),
                            "result": stdout,
                            "error": if !output.status.success() { Some(stderr) } else { None }
                        }))
                    }
                    Err(e) => {
                        Ok(json!({
                            "success": false,
                            "error": format!("Failed to execute 'graft': {}. Ensure graft is installed and built.", e)
                        }))
                    }
                }
            }

            "clrinf_graft_skeleton" => {
                let file = args.get("file").and_then(|f| f.as_str()).unwrap_or("");
                let mut cmd = std::process::Command::new("graft");
                cmd.arg("skeleton").arg(file);

                match cmd.output() {
                    Ok(output) => {
                        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
                        let stderr = String::from_utf8_lossy(&output.stderr).to_string();
                        Ok(json!({
                            "success": output.status.success(),
                            "skeleton": stdout,
                            "error": if !output.status.success() { Some(stderr) } else { None }
                        }))
                    }
                    Err(e) => {
                        Ok(json!({
                            "success": false,
                            "error": format!("Failed to execute 'graft skeleton': {}. Ensure graft is installed.", e)
                        }))
                    }
                }
            }

            _ => anyhow::bail!("Unknown MCP tool: {}", name),
        }
    }

    pub fn run_stdio(&self) -> Result<()> {
        let stdin = std::io::stdin();
        let mut stdout = std::io::stdout();
        let reader = stdin.lock();

        for line in reader.lines() {
            let line = line?;
            let trimmed = line.trim();
            if trimmed.is_empty() {
                continue;
            }

            if let Ok(req) = serde_json::from_str::<Value>(trimmed) {
                match self.handle_request(&req) {
                    Ok(Some(res)) => {
                        let res_str = serde_json::to_string(&res)?;
                        stdout.write_all(res_str.as_bytes())?;
                        stdout.write_all(b"\n")?;
                        stdout.flush()?;
                    }
                    Ok(None) => {}
                    Err(e) => {
                        let id = req.get("id").cloned().unwrap_or(Value::Null);
                        let err_res = json!({
                            "jsonrpc": "2.0",
                            "id": id,
                            "error": { "code": -32603, "message": e.to_string() }
                        });
                        let res_str = serde_json::to_string(&err_res)?;
                        stdout.write_all(res_str.as_bytes())?;
                        stdout.write_all(b"\n")?;
                        stdout.flush()?;
                    }
                }
            }
        }

        Ok(())
    }
}
