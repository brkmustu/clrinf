mod compat;
mod csharp;
mod drift;
mod event_scaffold;
mod impact;
mod report;
mod scan;
mod agents;
mod bench;
mod hook;
mod plan;
mod rules;
mod verify;
mod docs_provider;
mod generator;
mod manifest;
mod mcp;
mod module_adapter;
mod module_manager;
mod module_wiring;
mod schema;
mod template;
mod topology;
mod upcaster;
mod validator;
mod worker;

use anyhow::Result;
use clap::{Parser, Subcommand};
use generator::Generator;
use manifest::ProjectManifest;
use mcp::McpServer;
use module_adapter::{print_catalog, AdoptMode, ModuleAdapter, ModuleAdoptOptions};
use module_manager::ModuleManager;
use std::path::{Path, PathBuf};
use template::TemplateManager;
use validator::{check_schemas, validate_schemas};
use worker::{WorkerKind, WorkerManager};

#[derive(Parser)]
#[command(name = "clrinf-codegen")]
#[command(author = "clrinf Team")]
#[command(version)]
#[command(about = "Polyglot contracts and template catalog (Rust, C#, TypeScript, Elixir)", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Validate JSON schemas against Draft 2020-12
    Validate {
        /// Directory containing JSON schemas
        #[arg(short, long, default_value = "./tools/clrinf-codegen/schemas")]
        schema_dir: PathBuf,
    },
    /// Generate idiomatic types and models for target languages
    Generate {
        /// Directory containing JSON schemas
        #[arg(short, long, default_value = "./tools/clrinf-codegen/schemas")]
        schema_dir: PathBuf,

        /// Directory containing Tera templates
        #[arg(short, long, default_value = "./tools/clrinf-codegen/templates")]
        templates_dir: PathBuf,

        /// Target programming language (rust, csharp, typescript)
        #[arg(short, long)]
        lang: Option<String>,

        /// Output directory for generated code
        #[arg(short, long)]
        output: PathBuf,
    },
    /// Generate legacy adapter sketches (review required; handlers fail until implemented)
    GenerateSlice {
        /// Schema file or directory containing schemas
        #[arg(short, long, default_value = "./tools/clrinf-codegen/schemas")]
        schema: PathBuf,

        /// Target language stack: 'csharp', 'rust', or 'all'
        #[arg(short, long, default_value = "all")]
        lang: String,

        /// Directory containing Tera templates
        #[arg(short, long, default_value = "./tools/clrinf-codegen/templates")]
        templates_dir: PathBuf,

        /// Output root directory for generated CQRS slice
        #[arg(short, long, default_value = "./generated-slices")]
        output: PathBuf,
    },
    /// Generate event upcasters (v1 -> v2) for Rust, C# and TypeScript
    GenerateUpcasters {
        /// Directory containing YAML upcaster definitions
        #[arg(short, long, default_value = "./tools/clrinf-codegen/schemas")]
        schema_dir: PathBuf,

        /// Target language ('rust', 'csharp', 'typescript' or 'all')
        #[arg(short, long, default_value = "all")]
        lang: String,

        /// Directory containing Tera templates
        #[arg(short, long, default_value = "./tools/clrinf-codegen/templates")]
        templates_dir: PathBuf,

        /// Output directory for upcasters
        #[arg(short, long, default_value = "./generated-upcasters")]
        output: PathBuf,
    },
    /// Generate CloudEvent publisher envelopes and subscriber shells with idempotency enforcement
    GeneratePubsub {
        /// Directory containing JSON schemas
        #[arg(short, long, default_value = "./tools/clrinf-codegen/schemas")]
        schema_dir: PathBuf,

        /// Directory containing Tera templates
        #[arg(short, long, default_value = "./tools/clrinf-codegen/templates")]
        templates_dir: PathBuf,

        /// Target language ('rust', 'csharp', 'typescript', or 'all')
        #[arg(short, long, default_value = "all")]
        lang: String,

        /// Output directory for generated pub/sub code
        #[arg(short, long, default_value = "./generated-pubsub")]
        output: PathBuf,
    },
    /// Generate all declared modules, entities, and security from manifest (clrinf.toml or codegen.toml)
    #[command(alias = "fullstack")]
    GenerateAll {
        /// Path to configuration file (defaults to codegen.toml or clrinf.toml in directory)
        #[arg(short, long)]
        config: Option<PathBuf>,

        /// Directory containing Tera templates
        #[arg(long, default_value = "./templates")]
        templates_dir: PathBuf,

        /// Target project directory (defaults to current directory)
        #[arg(long, default_value = ".")]
        path: PathBuf,
    },
    /// Validate cross-service pub/sub topology, dead events, and evolution chains
    #[command(subcommand)]
    Topology(TopologyCommands),
    /// Event-first scaffolding: register a service as publisher/subscriber and regenerate shells
    #[command(subcommand)]
    Event(EventCommands),
    /// Check schema validity, unique identities and supported code generation constructs
    Check {
        /// Directory containing JSON schemas
        #[arg(short, long, default_value = "./tools/clrinf-codegen/schemas")]
        schema_dir: PathBuf,
    },
    /// Manage and scaffold manifest-described application templates
    #[command(subcommand)]
    Template(TemplateCommands),
    /// Create a new project (scaffold natively or from template)
    New {
        /// Project directory name to create
        name: String,

        /// Target language ('csharp', 'rust', 'typescript', etc.)
        #[arg(short, long)]
        lang: Option<String>,

        /// Template name (see 'template list')
        #[arg(short, long)]
        template: Option<String>,

        /// Architecture style for C# ('flat', 'layered', 'clean-cqrs')
        #[arg(long, default_value = "clean-cqrs")]
        arch: String,

        /// Programming paradigm for C# ('fp' or 'oop')
        #[arg(long, default_value = "fp")]
        paradigm: String,

        /// Dispatcher framework ('native', 'mediatr')
        #[arg(long, default_value = "native")]
        dispatcher: String,

        /// Directory containing templates
        #[arg(long, default_value = "./templates")]
        templates_dir: PathBuf,
    },
    /// Inspect and manage federated language workers (C#, Rust, TypeScript, Elixir)
    #[command(subcommand)]
    Worker(WorkerCommands),
    /// Run architectural linters across language workers (Roslyn, Syn, TS AST, Elixir AST)
    Lint {
        /// Target language: 'csharp', 'rust', 'typescript', 'elixir', or 'all'
        #[arg(short, long, default_value = "all")]
        lang: String,

        /// Optional file or directory path to scan
        #[arg(short, long)]
        path: Option<PathBuf>,
    },
    /// Manage business rules across language workers
    #[command(subcommand)]
    Rule(RuleCommands),
    /// Scaffold idiomatic application components (e.g. Elixir OTP)
    #[command(subcommand)]
    Scaffold(ScaffoldCommands),
    /// Initialize a clrinf project manifest (clrinf.toml) in the current or target directory
    Init {
        /// Target directory (defaults to current directory)
        #[arg(short, long, default_value = ".")]
        path: PathBuf,

        /// Project name (defaults to detected project name or directory name)
        #[arg(short, long)]
        name: Option<String>,

        /// Target language (defaults to auto-detection from Cargo.toml, package.json, mix.exs, or .csproj)
        #[arg(short, long)]
        lang: Option<String>,

        /// Architecture style
        #[arg(short, long, default_value = "clean-cqrs")]
        arch: String,

        /// Configuration profile: 'minimal' (pure domain, zero concerns), 'standard' (logging + transaction), or 'full' (all 7 concerns enabled)
        #[arg(long, default_value = "standard")]
        profile: String,

        /// Dispatcher framework: 'native', 'mediatr', or 'mediatornet'
        #[arg(long, default_value = "native")]
        dispatcher: String,
    },
    /// Manage cross-cutting concern modules
    #[command(subcommand)]
    Module(ModuleCommands),
    /// Adopt and transplant a module into an existing project (alias for 'clrinf module adopt')
    Adopt {
        /// Module name to adopt (e.g. deals, contacts, activities, caching, authz, crm)
        module: String,

        /// Target project file (.csproj, Cargo.toml, package.json) or directory
        #[arg(long)]
        to_project: PathBuf,

        /// Target sub-directory inside project (e.g. src/Features/Sales)
        #[arg(long)]
        target_dir: Option<PathBuf>,

        /// Adoption mode: 'raw' (zero-dependency pure code) or 'wired' (auto-hooked into module tree/DI)
        #[arg(long, default_value = "raw")]
        mode: String,

        /// Optional alias name for the module (e.g. 'Sales' for 'deals')
        #[arg(long = "as")]
        r#as: Option<String>,

        /// Optional source project path to copy module from
        #[arg(long)]
        source_project: Option<PathBuf>,
    },
    /// List all built-in official modules available for adoption (alias for 'clrinf module catalog')
    Catalog,
    /// Add domain modules or entities via language worker
    #[command(subcommand)]
    Add(AddCommands),
    /// Show current project status from clrinf.toml
    Status {
        /// Project directory (defaults to current directory)
        #[arg(short, long, default_value = ".")]
        path: PathBuf,
    },
    /// Show language-specific architectural guides and rules for LLM context optimization
    Docs {
        /// Target language: 'csharp', 'rust', 'typescript', 'elixir', 'cedar', 'cli', or 'all'
        #[arg(short, long)]
        lang: Option<String>,

        /// Optional project directory or file to auto-detect language from (.csproj, Cargo.toml, package.json, mix.exs)
        #[arg(short, long)]
        project: Option<PathBuf>,
    },
    /// Start Model Context Protocol (MCP) JSON-RPC 2.0 stdio server
    Mcp {
        /// Tool profile: full (default), events, or lean. Falls back to CLRINF_MCP_PROFILE, then clrinf.rules.toml [mcp]
        #[arg(long)]
        profile: Option<String>,
    },
    /// Install or run agent hooks that verify architecture after every edit
    #[command(subcommand)]
    Hook(HookCommands),
    /// Verify topology, code-vs-contract drift and dependency rules (clrinf.rules.toml)
    Verify {
        /// Project root
        #[arg(long, default_value = ".")]
        path: PathBuf,
        /// Only verify files changed relative to git HEAD (plus untracked)
        #[arg(long)]
        changed: bool,
        /// Emit machine-readable JSON
        #[arg(long)]
        json: bool,
    },
    /// Measure context cost (tokens) with and without clrinf's targeted slice
    #[command(subcommand)]
    Bench(BenchCommands),
    /// Fill-in-the-blanks plan for an event change (what is left for the agent to implement)
    Plan {
        /// Event type declared by x-event-type
        event_type: String,
        #[arg(long)]
        service: String,
        /// 'subscriber' or 'publisher'
        #[arg(long, default_value = "subscriber")]
        role: String,
        #[arg(short, long)]
        schema_dir: Option<PathBuf>,
        /// Source roots to scan (default: project root)
        #[arg(long = "src")]
        src: Vec<PathBuf>,
        /// Register the service in the schema now instead of previewing
        #[arg(long)]
        apply: bool,
        #[arg(long, default_value = ".")]
        path: PathBuf,
    },
    /// Keep AGENTS.md / CLAUDE.md / GEMINI.md / Cursor rules in sync with clrinf.rules.toml
    #[command(subcommand)]
    Agents(AgentsCommands),
}

#[derive(Subcommand)]
enum TopologyCommands {
    /// Check for dead events, orphan subscribers, and version gap upcaster chains
    Check {
        /// Directory containing JSON schemas
        #[arg(short, long, default_value = "./tools/clrinf-codegen/schemas")]
        schema_dir: PathBuf,

        /// Directory containing YAML upcaster definitions
        #[arg(short, long, default_value = "./tools/clrinf-codegen/schemas")]
        upcasters_dir: PathBuf,

        /// Render ASCII dependency topology graph
        #[arg(long, default_value_t = true)]
        ascii: bool,
    },
    /// Detect drift between declared topology (schemas) and what service source code actually does
    Drift {
        /// Directory containing JSON schemas
        #[arg(short, long, default_value = "./tools/clrinf-codegen/schemas")]
        schema_dir: PathBuf,

        /// Service source mapping as NAME=PATH (repeatable)
        #[arg(long = "service", required = true)]
        services: Vec<String>,

        /// Emit machine-readable JSON instead of text
        #[arg(long)]
        json: bool,
    },
    /// Show which services, versions and source files are affected by changing an event
    Impact {
        /// Event type (e.g. billing.invoice.issued.v1) or schema title
        event: String,

        /// Directory containing JSON schemas
        #[arg(short, long, default_value = "./tools/clrinf-codegen/schemas")]
        schema_dir: PathBuf,

        /// Source roots to scan for affected files (repeatable)
        #[arg(long = "src")]
        src: Vec<PathBuf>,

        /// Emit machine-readable JSON instead of text
        #[arg(long)]
        json: bool,
    },
    /// Produce a PR-friendly topology change report comparing a base schema set to the current one
    Report {
        /// Current (head) schema directory
        #[arg(short, long, default_value = "./tools/clrinf-codegen/schemas")]
        schema_dir: PathBuf,

        /// Base schema directory to compare against
        #[arg(long, conflicts_with = "base_ref")]
        base_dir: Option<PathBuf>,

        /// Git ref (branch, tag, commit) whose schema directory is the baseline
        #[arg(long)]
        base_ref: Option<String>,

        /// Directory containing YAML upcaster definitions
        #[arg(short, long, default_value = "./tools/clrinf-codegen/schemas")]
        upcasters_dir: PathBuf,

        /// Output format: markdown or json
        #[arg(long, default_value = "markdown")]
        format: String,

        /// Exit non-zero when the diff contains breaking or error-level changes
        #[arg(long)]
        fail_on_breaking: bool,
    },
}

#[derive(Subcommand)]
enum HookCommands {
    /// Install the hook (claude: PostToolUse after Write/Edit; git: pre-commit running `verify --changed`)
    Install {
        /// 'claude', 'git' or 'all'
        #[arg(long, default_value = "claude")]
        agent: String,
        #[arg(long, default_value = ".")]
        path: PathBuf,
        /// Executable name written into the hook
        #[arg(long, default_value = "clrinf")]
        command: String,
        /// Replace an existing foreign git pre-commit hook
        #[arg(long)]
        force: bool,
    },
    /// Remove hooks installed by clrinf
    Uninstall {
        #[arg(long, default_value = "all")]
        agent: String,
        #[arg(long, default_value = ".")]
        path: PathBuf,
    },
    /// Hook entry point: reads the agent tool payload on stdin (exit code 2 feeds violations back to the agent)
    Run {
        #[arg(long, default_value = ".")]
        path: PathBuf,
    },
}

#[derive(Subcommand)]
enum BenchCommands {
    /// Context-cost benchmark for an event change plus fixed MCP tool-schema cost per profile
    Context {
        /// Event type or schema title
        event: String,
        #[arg(short, long)]
        schema_dir: Option<PathBuf>,
        /// Source roots to scan (default: project root)
        #[arg(long = "src")]
        src: Vec<PathBuf>,
        #[arg(long, default_value = ".")]
        path: PathBuf,
        #[arg(long)]
        json: bool,
    },
}

#[derive(Subcommand)]
enum AgentsCommands {
    /// Write/refresh the managed clrinf block in agent instruction files
    Sync {
        #[arg(long, default_value = ".")]
        path: PathBuf,
        /// Do not write; exit non-zero when files are out of date (CI)
        #[arg(long)]
        check: bool,
    },
}

#[derive(Subcommand)]
enum EventCommands {
    /// Register SERVICE as a subscriber of EVENT_TYPE and regenerate subscriber shells
    Subscribe {
        /// Event type declared by x-event-type
        event_type: String,
        #[arg(long)]
        service: String,
        #[arg(short, long, default_value = "./tools/clrinf-codegen/schemas")]
        schema_dir: PathBuf,
        #[arg(short, long, default_value = "./tools/clrinf-codegen/templates")]
        templates_dir: PathBuf,
        /// Target language ('rust', 'csharp', 'typescript', or 'all')
        #[arg(short, long, default_value = "all")]
        lang: String,
        #[arg(short, long, default_value = "./generated-pubsub")]
        output: PathBuf,
        /// Only update the schema declaration; skip code generation
        #[arg(long)]
        no_generate: bool,
    },
    /// Register SERVICE as a publisher of EVENT_TYPE and regenerate publisher shells
    Publish {
        event_type: String,
        #[arg(long)]
        service: String,
        #[arg(short, long, default_value = "./tools/clrinf-codegen/schemas")]
        schema_dir: PathBuf,
        #[arg(short, long, default_value = "./tools/clrinf-codegen/templates")]
        templates_dir: PathBuf,
        #[arg(short, long, default_value = "all")]
        lang: String,
        #[arg(short, long, default_value = "./generated-pubsub")]
        output: PathBuf,
        #[arg(long)]
        no_generate: bool,
    },
}

#[derive(Subcommand)]
enum WorkerCommands {
    /// Check availability, path resolution and versions of language workers
    Check,
}

#[derive(Subcommand)]
enum RuleCommands {
    /// Scaffold an isolated, testable business rule in target language(s)
    New {
        /// Name of the rule (e.g. CheckMaxDiscount)
        name: String,

        /// Target language ('csharp', 'rust', 'typescript', 'elixir', or 'all')
        #[arg(short, long, default_value = "all")]
        lang: String,

        /// Target entity or command name (e.g. Order)
        #[arg(short, long)]
        entity: Option<String>,

        /// Canonical error code (e.g. MAX_DISCOUNT_EXCEEDED)
        #[arg(long)]
        error_code: Option<String>,

        /// Target directory
        #[arg(short, long)]
        target_dir: Option<PathBuf>,
    },
}

#[derive(Subcommand)]
enum ScaffoldCommands {
    /// Scaffold an idiomatic OTP worker, supervisor and pure handler in Elixir
    Otp {
        /// Module name (e.g. OrderProcessor)
        module: String,

        /// Application or project name (defaults to detected project name or 'app')
        #[arg(short, long)]
        app: Option<String>,

        /// Destination path (defaults to 'lib')
        #[arg(short, long, default_value = "lib")]
        path: PathBuf,

        /// Directory containing templates
        #[arg(long, default_value = "./tools/clrinf-codegen/templates")]
        templates_dir: PathBuf,
    },
}

#[derive(Subcommand)]
enum TemplateCommands {
    /// List available templates and their declared capabilities
    List {
        /// Directory containing templates
        #[arg(long, default_value = "./templates")]
        templates_dir: PathBuf,
    },
    /// Scaffold a new project from an existing template
    New {
        /// Target directory name for the new project
        name: String,

        /// Template name to use (see 'template list')
        #[arg(short, long)]
        template: String,

        /// Directory containing templates
        #[arg(long, default_value = "./templates")]
        templates_dir: PathBuf,
    },
}

#[derive(Subcommand)]
enum ModuleCommands {
    /// Add a cross-cutting concern module to the project
    Add {
        /// Module name (caching, logging, transaction, authentication, authorization, idempotency, outbox)
        module: String,

        /// Provider for the module (e.g., memory, redis, jwt, rbac)
        #[arg(short, long)]
        provider: Option<String>,

        /// Project directory (defaults to current directory)
        #[arg(long, default_value = ".")]
        path: PathBuf,
    },
    /// Remove a cross-cutting concern module from the project
    Remove {
        /// Module name to remove
        module: String,

        /// Project directory (defaults to current directory)
        #[arg(long, default_value = ".")]
        path: PathBuf,
    },
    /// List all modules and their status
    List {
        /// Project directory (defaults to current directory)
        #[arg(long, default_value = ".")]
        path: PathBuf,
    },
    /// Synchronize manifest modules with project files and language wiring
    Sync {
        /// Project directory (defaults to current directory)
        #[arg(long, default_value = ".")]
        path: PathBuf,
    },
    /// Adopt and transplant a module into an existing project in raw or wired mode
    Adopt {
        /// Module name to adopt (e.g. deals, contacts, activities, caching, authz, crm)
        module: String,

        /// Target project file (.csproj, Cargo.toml, package.json) or directory
        #[arg(long)]
        to_project: PathBuf,

        /// Target sub-directory inside project (e.g. src/Features/Sales)
        #[arg(long)]
        target_dir: Option<PathBuf>,

        /// Adoption mode: 'raw' (zero-dependency pure code) or 'wired' (auto-hooked into module tree/DI)
        #[arg(long, default_value = "raw")]
        mode: String,

        /// Optional alias name for the module (e.g. 'Sales' for 'deals')
        #[arg(long = "as")]
        r#as: Option<String>,

        /// Optional source project path to copy module from
        #[arg(long)]
        source_project: Option<PathBuf>,
    },
    /// List all built-in official modules available for adoption
    Catalog,
}

#[derive(Subcommand)]
enum AddCommands {
    /// Add a domain feature module with handlers, models, rules and multi-tenant Cedar policy
    Module {
        /// Module name
        name: String,

        /// Code generation pattern: 'flat' (default: multi-file flat layout) or 'basic' (single-file layout)
        #[arg(long, default_value = "flat")]
        pattern: String,

        /// Target project directory (defaults to current directory)
        #[arg(long, default_value = ".")]
        path: PathBuf,
    },
    /// Add a domain entity with CRUD operations (vertical-slice or separated pattern)
    Entity {
        /// Entity name
        name: String,

        /// Parent module name
        #[arg(short, long)]
        module: String,

        /// Entity properties in name:type format (e.g. -p name:string -p price:f64)
        #[arg(short, long)]
        prop: Vec<String>,

        /// Code generation pattern: 'vertical-slice' (single file per operation) or 'separated' (legacy multi-file)
        #[arg(long, default_value = "vertical-slice")]
        pattern: String,

        /// Directory containing Tera templates
        #[arg(long, default_value = "./templates")]
        templates_dir: PathBuf,

        /// Target project directory (defaults to current directory)
        #[arg(long, default_value = ".")]
        path: PathBuf,
    },
    /// Add security module (JWT, Users, Roles, Claims) to the project
    Security {
        /// Security mode: 'basic' (JWT + Refresh Token) or 'advanced' (JWT + 2FA/OTP/Email)
        #[arg(long, default_value = "basic")]
        mode: String,

        /// Directory containing Tera templates
        #[arg(long, default_value = "./templates")]
        templates_dir: PathBuf,

        /// Target project directory (defaults to current directory)
        #[arg(long, default_value = ".")]
        path: PathBuf,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Validate { schema_dir } => {
            println!("🔍 Validating schemas in {} ...", schema_dir.display());
            let count = validate_schemas(&schema_dir)?;
            println!("✅ Successfully validated {} schemas.", count);
        }
        Commands::Check { schema_dir } => {
            let count = check_schemas(&schema_dir)?;
            println!("Checked {count} schemas: valid schemas, unique identities, supported model shapes. This command does not certify runtime adapters or run application tests.");
        }
        Commands::GenerateUpcasters {
            schema_dir,
            lang,
            templates_dir,
            output,
        } => {
            println!(
                "🔄 Event Upcaster (Schema Evolution) üretiliyor (Hedef Dil: {})...",
                lang
            );
            let gen = Generator::new(&templates_dir)?;
            let count = gen.generate_upcasters(&schema_dir, &lang, &output)?;
            println!(
                "✅ Başarıyla {} upcaster üretildi. Çıktı: {}",
                count,
                output.display()
            );
        }
        Commands::Topology(TopologyCommands::Check {
            schema_dir,
            upcasters_dir,
            ascii,
        }) => {
            println!("🌐 Validating cross-service pub/sub topology from {} ...", schema_dir.display());
            let report = topology::TopologyValidator::validate(&schema_dir, Some(&upcasters_dir))?;

            if ascii {
                println!("\n{}", report.format_ascii_graph());
            } else {
                println!("{}", report.format_diagnostics());
            }

            if !report.is_valid {
                anyhow::bail!("Topology validation failed with {} fatal error(s).", report.errors_count());
            }
        }
        Commands::Topology(TopologyCommands::Drift {
            schema_dir,
            services,
            json,
        }) => {
            let sources = services
                .iter()
                .map(|s| drift::ServiceSource::parse(s))
                .collect::<Result<Vec<_>>>()?;
            let report = drift::detect(&schema_dir, &sources)?;
            if json {
                println!("{}", serde_json::to_string_pretty(&report)?);
            } else {
                println!("{}", report.format_diagnostics());
            }
            if !report.is_valid {
                anyhow::bail!("Drift check failed with {} error(s).", report.errors_count());
            }
        }
        Commands::Topology(TopologyCommands::Impact {
            event,
            schema_dir,
            src,
            json,
        }) => {
            let roots = if src.is_empty() { vec![PathBuf::from(".")] } else { src };
            let report = impact::analyze(&schema_dir, &event, &roots)?;
            if json {
                println!("{}", serde_json::to_string_pretty(&report)?);
            } else {
                println!("{}", report.format_text());
            }
        }
        Commands::Topology(TopologyCommands::Report {
            schema_dir,
            base_dir,
            base_ref,
            upcasters_dir,
            format,
            fail_on_breaking,
        }) => {
            let (base, cleanup) = match (base_dir, base_ref) {
                (Some(dir), _) => (dir, None),
                (None, Some(reference)) => {
                    let dir = report::materialize_git_ref(&schema_dir, &reference)?;
                    (dir.clone(), Some(dir))
                }
                (None, None) => anyhow::bail!("Provide --base-dir or --base-ref"),
            };
            let result = report::compare(&base, &schema_dir, Some(&upcasters_dir));
            if let Some(dir) = cleanup {
                let _ = std::fs::remove_dir_all(dir);
            }
            let diff = result?;
            match format.as_str() {
                "json" => println!("{}", serde_json::to_string_pretty(&diff)?),
                "markdown" | "md" => println!("{}", diff.to_markdown()),
                other => anyhow::bail!("Unsupported --format '{other}'; use markdown or json"),
            }
            if fail_on_breaking && diff.has_breaking() {
                anyhow::bail!("Topology report contains breaking or error-level changes.");
            }
        }
        Commands::Event(command) => {
            let (role, event_type, service, schema_dir, templates_dir, lang, output, no_generate) = match command {
                EventCommands::Subscribe { event_type, service, schema_dir, templates_dir, lang, output, no_generate } => {
                    (event_scaffold::Role::Subscriber, event_type, service, schema_dir, templates_dir, lang, output, no_generate)
                }
                EventCommands::Publish { event_type, service, schema_dir, templates_dir, lang, output, no_generate } => {
                    (event_scaffold::Role::Publisher, event_type, service, schema_dir, templates_dir, lang, output, no_generate)
                }
            };
            let outcome = event_scaffold::register(&schema_dir, &event_type, role, &service)?;
            let verb = if role == event_scaffold::Role::Subscriber { "subscriber" } else { "publisher" };
            if outcome.changed {
                println!("✅ Registered '{}' as {} of {} in {}", service, verb, event_type, outcome.schema_file.display());
            } else {
                println!("ℹ️  '{}' is already a {} of {}.", service, verb, event_type);
            }
            if !no_generate {
                let gen = Generator::new(&templates_dir)?;
                let langs: Vec<&str> = if lang.eq_ignore_ascii_case("all") {
                    vec!["rust", "csharp", "typescript"]
                } else {
                    vec![lang.as_str()]
                };
                for l in langs {
                    let lang_out = if lang.eq_ignore_ascii_case("all") { output.join(l) } else { output.clone() };
                    let count = gen.generate_pubsub(&schema_dir, l, &lang_out)?;
                    println!("📡 Generated {} pub/sub files for {} in {}", count, l, lang_out.display());
                }
            }
            let report = topology::TopologyValidator::validate(&schema_dir, Some(&schema_dir))?;
            let relevant: Vec<_> = report.issues.iter().filter(|i| i.event_type.starts_with(&event_type) || event_type.contains(&i.event_type)).collect();
            for issue in relevant {
                println!("⚠️  [{}] {}", issue.code, issue.message);
            }
            println!(
                "👉 Next: implement the {} body for '{}' ({}). Business logic only; envelope/idempotency are generated.",
                if role == event_scaffold::Role::Subscriber { "handler" } else { "publish call site" },
                event_type,
                outcome.title
            );
        }
        Commands::GeneratePubsub {
            schema_dir,
            templates_dir,
            lang,
            output,
        } => {
            let gen = Generator::new(&templates_dir)?;
            let langs: Vec<&str> = if lang.eq_ignore_ascii_case("all") {
                vec!["rust", "csharp", "typescript"]
            } else {
                vec![lang.as_str()]
            };

            let mut total = 0;
            for l in langs {
                let lang_out = if lang.eq_ignore_ascii_case("all") {
                    output.join(l)
                } else {
                    output.clone()
                };
                println!("📡 Generating CloudEvent publisher & subscriber shells for \x1b[1;33m{}\x1b[0m to {} ...", l, lang_out.display());
                let count = gen.generate_pubsub(&schema_dir, l, &lang_out)?;
                println!("✅ Generated {} pub/sub files for {}.", count, l);
                total += count;
            }
            println!("🎉 Total pub/sub files generated: {}", total);
        }
        Commands::GenerateAll {
            config,
            templates_dir,
            path,
        } => {
            let abs_path = std::fs::canonicalize(&path).unwrap_or_else(|_| path.clone());
            let (manifest, manifest_path) = match config {
                Some(ref cfg_path) => {
                    let p = if cfg_path.is_absolute() {
                        cfg_path.clone()
                    } else if cfg_path.exists() {
                        std::fs::canonicalize(cfg_path).unwrap_or_else(|_| cfg_path.clone())
                    } else {
                        abs_path.join(cfg_path)
                    };
                    if p.file_name().and_then(|s| s.to_str()).map(|s| s.contains("codegen")).unwrap_or(false) {
                        (ProjectManifest::load_from_codegen_toml(&p)?, p)
                    } else {
                        (ProjectManifest::load(&p)?, p)
                    }
                }
                None => ProjectManifest::discover(&abs_path)?,
            };

            let tpl_dir = if templates_dir.is_dir() {
                templates_dir.clone()
            } else if let Some(ref root) = WorkerManager::new().workspace_root() {
                root.join("tools").join("clrinf-codegen").join("templates")
            } else {
                templates_dir.clone()
            };

            println!(
                "🚀 Proje için toplu kod üretimi başlatılıyor: '{}' ({})\n",
                manifest.project.name,
                manifest_path.display()
            );
            let mut total_created = 0;
            let mut total_modified = 0;

            // 1. Security generation if enabled
            if manifest.project.secured {
                println!("🔐 Güvenlik modülü üretiliyor...");
                let sec_opts = csharp::security::AddSecurityOptions {
                    project_path: &abs_path,
                    manifest: &manifest,
                    templates_dir: &tpl_dir,
                    mode: csharp::security::SecurityMode::Basic,
                };
                match csharp::security::add_security(&sec_opts) {
                    Ok(res) => {
                        println!(
                            "  ✅ Güvenlik modülü oluşturuldu ({} dosya eklendi, {} güncellendi).",
                            res.files_created.len(),
                            res.files_modified.len()
                        );
                        total_created += res.files_created.len();
                        total_modified += res.files_modified.len();
                    }
                    Err(e) => eprintln!("  ⚠️ Güvenlik modülü üretilirken hata: {}", e),
                }
            }

            // 2. Iterate declared modules
            for module in &manifest.declared_modules {
                println!("\n📦 Modül üretiliyor: '{}' ...", module.name);
                let mod_opts = csharp::module::AddModuleOptions {
                    module_name: &module.name,
                    project_path: &abs_path,
                    manifest: &manifest,
                    pattern: Some(manifest.project.pattern),
                };
                let mod_res = csharp::module::add_module(&mod_opts)?;
                total_created += mod_res.files_created.len();
                total_modified += mod_res.files_modified.len();

                // Entity generation with vertical slices
                let props: Vec<(&str, &str)> = module
                    .properties
                    .iter()
                    .map(|p| (p.name.as_str(), p.prop_type.as_str()))
                    .collect();

                let ent_opts = csharp::entity::AddEntityOptions {
                    entity_name: &module.name,
                    module_name: &module.name,
                    properties: &props,
                    id_type: Some(module.id_type.as_str()),
                    project_path: &abs_path,
                    manifest: &manifest,
                    templates_dir: &tpl_dir,
                };
                let ent_res = csharp::entity::add_entity(&ent_opts)?;
                total_created += ent_res.files_created.len();
                total_modified += ent_res.files_modified.len();
            }

            println!(
                "\n🎉 Toplu kod üretimi başarıyla tamamlandı: {} dosya oluşturuldu, {} dosya güncellendi.",
                total_created, total_modified
            );
        }
        Commands::Generate {
            schema_dir,
            templates_dir,
            lang,
            output,
        } => {
            let gen = Generator::new(&templates_dir)?;

            if let Some(target_lang) = lang {
                println!(
                    "⚙️  Generating {} code from {} to {} ...",
                    target_lang,
                    schema_dir.display(),
                    output.display()
                );
                let count = gen.generate(&schema_dir, &target_lang, &output)?;
                println!("✅ Generated {} files for {}.", count, target_lang);
            } else {
                for target_lang in &["rust", "csharp", "typescript", "elixir"] {
                    let lang_out = output.join(target_lang);
                    println!(
                        "⚙️  Generating {} code to {} ...",
                        target_lang,
                        lang_out.display()
                    );
                    let count = gen.generate(&schema_dir, target_lang, &lang_out)?;
                    println!("✅ Generated {} files for {}.", count, target_lang);
                }
            }
        }
        Commands::GenerateSlice {
            schema,
            lang,
            templates_dir,
            output,
        } => {
            eprintln!("Legacy adapter sketches only: not a runnable service. Review adapter dependencies, policies and business behavior; generated handlers deliberately fail until implemented. Use template new for a runnable minimal starter.");
            println!("⚡ Generating full-slice CQRS architecture (Stack: \x1b[1;33m{}\x1b[0m) from '{}' ...", lang, schema.display());
            let gen = Generator::new(&templates_dir)?;
            let count = gen.generate_slice(&schema, &lang, &output)?;
            println!("Generated {} review-required adapter sketch files.", count);
            println!("📂 Çıktı konumu: {}\n", output.display());
        }
        Commands::Template(TemplateCommands::List { templates_dir }) => {
            let mgr = TemplateManager::new(&templates_dir);
            let templates = mgr.list_templates()?;
            println!("📦 Mevcut clrinf Şablonları:\n");
            for (name, desc, path) in templates {
                println!("  • \x1b[1;32m{}\x1b[0m", name);
                println!(
                    "    {}\n    \x1b[2mKonum: {}\x1b[0m\n",
                    desc,
                    path.display()
                );
            }
        }
        Commands::Template(TemplateCommands::New {
            name,
            template,
            templates_dir,
        }) => {
            let mgr = TemplateManager::new(&templates_dir);
            let target_path = PathBuf::from(&name);
            mgr.scaffold(&template, &target_path)?;
        }
        Commands::New {
            name,
            lang,
            template,
            arch,
            paradigm,
            dispatcher,
            templates_dir,
        } => {
            if let Some(ref tmpl) = template {
                let mgr = TemplateManager::new(&templates_dir);
                let target_path = PathBuf::from(&name);
                mgr.scaffold(tmpl, &target_path)?;
            } else {
                let target_lang = lang.as_deref().unwrap_or("csharp");
                if target_lang.eq_ignore_ascii_case("csharp") {
                    let arch_style = match arch.to_lowercase().as_str() {
                        "flat" => manifest::ArchStyle::Flat,
                        "layered" => manifest::ArchStyle::Layered,
                        _ => manifest::ArchStyle::CleanCqrs,
                    };
                    let disp = match dispatcher.to_lowercase().as_str() {
                        "mediatr" => manifest::Dispatcher::Mediatr,
                        "mediatornet" => manifest::Dispatcher::MediatorNet,
                        _ => manifest::Dispatcher::Native,
                    };
                    let target_dir = PathBuf::from(&name);
                    let parent_dir = target_dir.parent().unwrap_or_else(|| Path::new("."));
                    let project_name = target_dir.file_name().and_then(|s| s.to_str()).unwrap_or(&name);

                    let opts = csharp::project::CreateProjectOptions {
                        name: project_name,
                        target_path: parent_dir,
                        arch: arch_style,
                        host_type: manifest::HostType::Api,
                        dispatcher: disp,
                        paradigm: &paradigm,
                        profile: "standard",
                        db_context: None,
                        skip_dotnet_exec: false,
                    };

                    println!("🏗️  Scaffolding C# project '{}' (arch: {}, paradigm: {}, dispatcher: {})...", project_name, arch, paradigm, dispatcher);
                    let res = csharp::project::create_project(&opts)?;
                    println!("🎉 Project created successfully at: {}\nCreated files:", res.project_dir.display());
                    for f in &res.files_created {
                        println!("  ✅ {}", f.display());
                    }
                } else {
                    anyhow::bail!("Desteklenmeyen dil: '{}'. Şablon kullanmak için '--template <ad>' belirtin.", target_lang);
                }
            }
        }
        Commands::Worker(WorkerCommands::Check) => {
            let mgr = WorkerManager::new();
            let workers = mgr.resolve_all_workers();
            println!("🔧 Federated clrinf Language Workers Status:\n");
            for w in workers {
                let status_icon = if w.is_available { "✅" } else { "❌" };
                println!("{} \x1b[1m{}\x1b[0m ({})", status_icon, w.name, w.kind.as_str());
                if w.is_available {
                    if let Some(ref ver) = w.version {
                        println!("   Sürüm: {}", ver);
                    }
                    println!("   Komut: {} {:?}", w.executable, w.args_prefix);
                } else {
                    println!("   \x1b[33mKurulum Rehberi:\x1b[0m {}", w.install_hint);
                }
                println!();
            }
        }
        Commands::Lint { lang, path } => {
            let mgr = WorkerManager::new();
            let kinds: Vec<WorkerKind> = if lang.eq_ignore_ascii_case("all") {
                WorkerKind::all().to_vec()
            } else if let Some(k) = WorkerKind::from_str_loose(&lang) {
                vec![k]
            } else {
                anyhow::bail!("Desteklenmeyen dil: {}. (csharp, rust, typescript, all)", lang);
            };

            let mut all_passed = true;
            for kind in kinds {
                println!("🔍 Running architectural linter for \x1b[1m{}\x1b[0m ...", kind.as_str());
                match mgr.lint(kind, path.as_deref()) {
                    Ok(res) => {
                        if !res.stdout.trim().is_empty() {
                            println!("{}", res.stdout.trim());
                        }
                        if !res.stderr.trim().is_empty() {
                            eprintln!("{}", res.stderr.trim());
                        }
                        if !res.success {
                            all_passed = false;
                            println!("❌ [{}] Mimari denetim ihlal tespit etti veya başarısız oldu.\n", kind.as_str());
                        } else {
                            println!("✅ [{}] Mimari denetim başarılı.\n", kind.as_str());
                        }
                    }
                    Err(e) => {
                        all_passed = false;
                        eprintln!("❌ [{}] Linter çalıştırılamadı: {}\n", kind.as_str(), e);
                    }
                }
            }

            if !all_passed {
                anyhow::bail!("Bir veya daha fazla dilde mimari ihlal tespit edildi.");
            }
        }
        Commands::Rule(RuleCommands::New {
            name,
            lang,
            entity,
            error_code,
            target_dir,
        }) => {
            let mgr = WorkerManager::new();
            let kinds: Vec<WorkerKind> = if lang.eq_ignore_ascii_case("all") {
                WorkerKind::all().to_vec()
            } else if let Some(k) = WorkerKind::from_str_loose(&lang) {
                vec![k]
            } else {
                anyhow::bail!("Desteklenmeyen dil: {}. (csharp, rust, typescript, elixir, all)", lang);
            };

            for kind in kinds {
                println!("⚡ Scaffolding business rule '{}' for \x1b[1m{}\x1b[0m ...", name, kind.as_str());
                match mgr.scaffold_rule(kind, &name, entity.as_deref(), error_code.as_deref(), target_dir.as_deref()) {
                    Ok(res) => {
                        if !res.stdout.trim().is_empty() {
                            println!("{}", res.stdout.trim());
                        }
                        if !res.stderr.trim().is_empty() {
                            eprintln!("{}", res.stderr.trim());
                        }
                        if res.success {
                            println!("✅ [{}] Kural ve test başarıyla üretildi.\n", kind.as_str());
                        } else {
                            eprintln!("❌ [{}] Kural üretimi başarısız oldu.\n", kind.as_str());
                        }
                    }
                    Err(e) => {
                        eprintln!("❌ [{}] Kural üretilemedi: {}\n", kind.as_str(), e);
                    }
                }
            }
        }
        Commands::Scaffold(ScaffoldCommands::Otp {
            module,
            app,
            path,
            templates_dir,
        }) => {
            let app_name = app.unwrap_or_else(|| {
                manifest::detect_project(Path::new("."))
                    .and_then(|d| d.name)
                    .unwrap_or_else(|| "app".to_string())
            });

            println!(
                "⚡ Scaffolding OTP module '{}' for app '{}' into '{}' ...",
                module,
                app_name,
                path.display()
            );
            let gen = generator::Generator::new(&templates_dir)?;
            let created = gen.scaffold_otp(&module, &app_name, &path)?;
            println!("✅ Başarıyla {} OTP dosyası üretildi:", created.len());
            for f in created {
                println!("   • {}", f.display());
            }
        }
        Commands::Init {
            path,
            name,
            lang,
            arch,
            profile,
            dispatcher,
        } => {
            let abs_path = std::fs::canonicalize(&path).unwrap_or_else(|_| path.clone());
            let detected = manifest::detect_project(&abs_path);
            if let Some(ref d) = detected {
                println!(
                    "🔍 Mevcut proje algılandı ({}): Dil = {}, Ad = {:?}",
                    d.source_file,
                    d.lang.as_str(),
                    d.name
                );
            }

            let lang_enum = if let Some(ref l) = lang {
                match l.to_lowercase().as_str() {
                    "csharp" | "cs" | "dotnet" => manifest::Lang::CSharp,
                    "rust" | "rs" => manifest::Lang::Rust,
                    "typescript" | "ts" | "js" => manifest::Lang::TypeScript,
                    "elixir" | "ex" => manifest::Lang::Elixir,
                    _ => anyhow::bail!("Desteklenmeyen dil: {}. (csharp, rust, typescript, elixir)", l),
                }
            } else if let Some(ref d) = detected {
                d.lang
            } else {
                manifest::Lang::CSharp
            };

            let project_name = name
                .or_else(|| detected.and_then(|d| d.name))
                .unwrap_or_else(|| {
                    abs_path
                        .file_name()
                        .and_then(|n| n.to_str())
                        .unwrap_or("MyApp")
                        .to_string()
                    });

            let arch_enum = match arch.to_lowercase().as_str() {
                "flat" => manifest::ArchStyle::Flat,
                "layered" => manifest::ArchStyle::Layered,
                "clean-cqrs" | "cleancqrs" => manifest::ArchStyle::CleanCqrs,
                _ => anyhow::bail!("Desteklenmeyen mimari: {}. (flat, layered, clean-cqrs)", arch),
            };

            let dispatcher_enum = match dispatcher.to_lowercase().as_str() {
                "mediatr" => manifest::Dispatcher::Mediatr,
                "mediatornet" | "mediator.net" => manifest::Dispatcher::MediatorNet,
                _ => manifest::Dispatcher::Native,
            };

            let manifest_path = abs_path.join("clrinf.toml");
            if manifest_path.exists() {
                anyhow::bail!(
                    "clrinf.toml already exists at {}. Use 'clrinf module add' to modify modules.",
                    manifest_path.display()
                );
            }

            let modules_config = match profile.to_lowercase().as_str() {
                "minimal" | "lean" | "pure" => manifest::ModulesConfig::minimal(),
                "full" | "batteries" | "all" => manifest::ModulesConfig::full(),
                _ => manifest::ModulesConfig::standard(),
            };

            let m = ProjectManifest {
                project: manifest::ProjectConfig {
                    name: project_name.clone(),
                    lang: lang_enum,
                    arch: arch_enum,
                    dispatcher: dispatcher_enum,
                    ..Default::default()
                },
                modules: modules_config,
                ..Default::default()
            };

            if !abs_path.exists() {
                std::fs::create_dir_all(&abs_path)?;
            }
            m.save(&manifest_path)?;

            // If 'full' profile was requested, automatically scaffold and wire all modules
            if profile.eq_ignore_ascii_case("full") || profile.eq_ignore_ascii_case("batteries") || profile.eq_ignore_ascii_case("all") {
                let mgr = ModuleManager::new(None)?;
                let _ = mgr.sync_modules(&m, &abs_path)?;
            }

            println!(
                "✅ clrinf.toml başarıyla oluşturuldu: {}\n   Proje: {}\n   Dil: {}\n   Mimari: {:?}\n   Profil: {}",
                manifest_path.display(),
                project_name,
                lang_enum.as_str(),
                arch_enum,
                profile
            );
        }
        Commands::Status { path } => {
            let abs_path = std::fs::canonicalize(&path).unwrap_or_else(|_| path.clone());
            let (m, manifest_path) = ProjectManifest::discover(&abs_path)?;
            println!("📋 clrinf Proje Durumu\n");
            println!("   Manifest: {}", manifest_path.display());
            println!("   Proje: {}", m.project.name);
            println!("   Dil: {}", m.project.lang.as_str());
            println!("   Mimari: {:?}", m.project.arch);
            println!("   Dağıtım: {:?}", m.project.deployment);
            println!("   Host: {:?}", m.project.host_type);
            println!("\n   Etkin Modüller:");
            let enabled = m.enabled_modules();
            if enabled.is_empty() {
                println!("   (hiçbir opsiyonel modül etkin değil)");
            } else {
                for name in &enabled {
                    println!("   ✅ {}", name);
                }
            }
            println!();
        }
        Commands::Module(ModuleCommands::List { path }) => {
            let abs_path = std::fs::canonicalize(&path).unwrap_or_else(|_| path.clone());
            let (m, _) = ProjectManifest::discover(&abs_path)?;
            println!("📦 Modül Durumu:\n");

            let modules: Vec<(&str, bool, &str)> = vec![
                ("caching", m.modules.caching.enabled, "Önbellekleme"),
                ("logging", m.modules.logging.enabled, "Yapılandırılmış Loglama"),
                ("transaction", m.modules.transaction.enabled, "Transaction Yönetimi"),
                ("authentication", m.modules.authentication.enabled, "Kimlik Doğrulama"),
                ("authorization", m.modules.authorization.enabled, "Yetkilendirme"),
                ("error_handling", m.modules.error_handling.enabled, "Hata Yönetimi"),
                ("validation", m.modules.validation.enabled, "Girdi Doğrulama"),
                ("idempotency", m.modules.idempotency.enabled, "Idempotency"),
                ("outbox", m.modules.outbox.enabled, "Transactional Outbox"),
            ];

            for (name, enabled, desc) in modules {
                let icon = if enabled { "✅" } else { "⬜" };
                println!("  {} {:20} — {}", icon, name, desc);
            }
            println!();
        }
        Commands::Module(ModuleCommands::Add {
            module,
            provider,
            path,
        }) => {
            let abs_path = std::fs::canonicalize(&path).unwrap_or_else(|_| path.clone());
            let (mut m, manifest_path) = ProjectManifest::discover(&abs_path)?;
            let manager = ModuleManager::new(None)?;
            let created = manager.add_module(&mut m, &manifest_path, &module, provider.as_deref())?;
            println!("✅ '{}' modülü başarıyla eklendi ve bağlandı ({} dosya oluşturuldu).", module, created.len());
            for f in created {
                println!("   • {}", f.display());
            }
        }
        Commands::Module(ModuleCommands::Remove { module, path }) => {
            let abs_path = std::fs::canonicalize(&path).unwrap_or_else(|_| path.clone());
            let (mut m, manifest_path) = ProjectManifest::discover(&abs_path)?;
            let manager = ModuleManager::new(None)?;
            let removed = manager.remove_module(&mut m, &manifest_path, &module)?;
            println!("✅ '{}' modülü devre dışı bırakıldı ve temizlendi ({} dosya silindi).", module, removed.len());
            for f in removed {
                println!("   • {}", f.display());
            }
        }
        Commands::Module(ModuleCommands::Sync { path }) => {
            let abs_path = std::fs::canonicalize(&path).unwrap_or_else(|_| path.clone());
            let (m, _) = ProjectManifest::discover(&abs_path)?;
            let manager = ModuleManager::new(None)?;
            let report = manager.sync_modules(&m, &abs_path)?;
            println!("🔄 Modül senkronizasyonu tamamlandı:");
            println!("   Etkin ve bağlanan modüller: {:?}", report.synced);
            if !report.cleaned.is_empty() {
                println!("   Temizlenen modüller: {:?}", report.cleaned);
            }
        }
        Commands::Module(ModuleCommands::Adopt {
            module,
            to_project,
            target_dir,
            mode,
            r#as,
            source_project,
        }) => {
            execute_module_adopt(module, to_project, target_dir, mode, r#as, source_project)?;
        }
        Commands::Adopt {
            module,
            to_project,
            target_dir,
            mode,
            r#as,
            source_project,
        } => {
            execute_module_adopt(module, to_project, target_dir, mode, r#as, source_project)?;
        }
        Commands::Module(ModuleCommands::Catalog) | Commands::Catalog => {
            print_catalog();
        }
        Commands::Add(AddCommands::Module { name, pattern, path }) => {
            let abs_path = std::fs::canonicalize(&path).unwrap_or_else(|_| path.clone());
            let (m, _) = ProjectManifest::discover(&abs_path)?;
            match m.project.lang {
                manifest::Lang::CSharp => {
                    println!("🚀 Modül ekleniyor (C#): '{}' (Desen: {}) ...", name, pattern);
                    let pattern_style = match pattern.to_lowercase().as_str() {
                        "basic" => manifest::PatternStyle::Basic,
                        "separated" => manifest::PatternStyle::Separated,
                        _ => manifest::PatternStyle::Flat,
                    };
                    let opts = csharp::module::AddModuleOptions {
                        module_name: &name,
                        project_path: &abs_path,
                        manifest: &m,
                        pattern: Some(pattern_style),
                    };
                    let result = csharp::module::add_module(&opts)?;
                    println!(
                        "\n✅ {} dosya oluşturuldu, {} dosya güncellendi.",
                        result.files_created.len(),
                        result.files_modified.len()
                    );
                }
                _ => {
                    let worker_mgr = WorkerManager::new();
                    let worker_kind = match m.project.lang {
                        manifest::Lang::CSharp => WorkerKind::CSharp,
                        manifest::Lang::Rust => WorkerKind::Rust,
                        manifest::Lang::TypeScript => WorkerKind::TypeScript,
                        manifest::Lang::Elixir => WorkerKind::Elixir,
                    };
                    println!("🚀 Modül ekleniyor: '{}' (Dil: {}) ...", name, m.project.lang.as_str());
                    let res = worker_mgr.add_module(worker_kind, &name, Some(&abs_path))?;
                    if res.success {
                        println!("{}", res.stdout.trim());
                    } else {
                        eprintln!("{}", res.stderr.trim());
                        std::process::exit(res.exit_code.unwrap_or(1));
                    }
                }
            }
        }
        Commands::Add(AddCommands::Entity {
            name,
            module,
            prop,
            pattern,
            templates_dir,
            path,
        }) => {
            let abs_path = std::fs::canonicalize(&path).unwrap_or_else(|_| path.clone());
            let (m, _) = ProjectManifest::discover(&abs_path)?;

            // Resolve templates directory
            let tpl_dir = if templates_dir.is_dir() {
                templates_dir.clone()
            } else if let Some(ref root) = WorkerManager::new().workspace_root() {
                root.join("tools").join("clrinf-codegen").join("templates")
            } else {
                templates_dir.clone()
            };

            let parsed_props: Vec<(&str, &str)> = prop
                .iter()
                .map(|s| {
                    let mut parts = s.splitn(2, ':');
                    let k = parts.next().unwrap_or("");
                    let v = parts.next().unwrap_or("string");
                    (k, v)
                })
                .collect();

            match m.project.lang {
                manifest::Lang::CSharp => {
                    if pattern.eq_ignore_ascii_case("vertical-slice") {
                        println!(
                            "🚀 Entity ekleniyor (vertical-slice): '{}' -> Modül: '{}' ...",
                            name, module
                        );
                        let opts = csharp::entity::AddEntityOptions {
                            entity_name: &name,
                            module_name: &module,
                            properties: &parsed_props,
                            id_type: None,
                            project_path: &abs_path,
                            manifest: &m,
                            templates_dir: &tpl_dir,
                        };
                        let result = csharp::entity::add_entity(&opts)?;
                        println!("\n✅ {} dosya oluşturuldu, {} dosya güncellendi.",
                            result.files_created.len(),
                            result.files_modified.len()
                        );
                    } else {
                        // Fallback to legacy worker-based generation
                        println!(
                            "🚀 Entity ekleniyor (separated): '{}' -> Modül: '{}' ...",
                            name, module
                        );
                        let worker_mgr = WorkerManager::new();
                        let res = worker_mgr.add_entity(WorkerKind::CSharp, &name, &module, &parsed_props, Some(&abs_path))?;
                        if res.success {
                            println!("{}", res.stdout.trim());
                        } else {
                            eprintln!("{}", res.stderr.trim());
                            std::process::exit(res.exit_code.unwrap_or(1));
                        }
                    }
                }
                _ => {
                    // Non-C# languages: delegate to language worker
                    let worker_mgr = WorkerManager::new();
                    let worker_kind = match m.project.lang {
                        manifest::Lang::CSharp => WorkerKind::CSharp,
                        manifest::Lang::Rust => WorkerKind::Rust,
                        manifest::Lang::TypeScript => WorkerKind::TypeScript,
                        manifest::Lang::Elixir => WorkerKind::Elixir,
                    };
                    println!(
                        "🚀 Entity ekleniyor: '{}' -> Modül: '{}' (Dil: {}) ...",
                        name, module, m.project.lang.as_str()
                    );
                    let res = worker_mgr.add_entity(worker_kind, &name, &module, &parsed_props, Some(&abs_path))?;
                    if res.success {
                        println!("{}", res.stdout.trim());
                    } else {
                        eprintln!("{}", res.stderr.trim());
                        std::process::exit(res.exit_code.unwrap_or(1));
                    }
                }
            }
        }
        Commands::Add(AddCommands::Security {
            mode,
            templates_dir,
            path,
        }) => {
            let abs_path = std::fs::canonicalize(&path).unwrap_or_else(|_| path.clone());
            let (m, _) = ProjectManifest::discover(&abs_path)?;
            let tpl_dir = if templates_dir.is_dir() {
                templates_dir.clone()
            } else if let Some(ref root) = WorkerManager::new().workspace_root() {
                root.join("tools").join("clrinf-codegen").join("templates")
            } else {
                templates_dir.clone()
            };

            let sec_mode = csharp::security::SecurityMode::from_str(&mode);
            println!("🔐 Güvenlik modülü ekleniyor (Mod: {:?}) ...", sec_mode);
            let opts = csharp::security::AddSecurityOptions {
                project_path: &abs_path,
                manifest: &m,
                templates_dir: &tpl_dir,
                mode: sec_mode,
            };
            let res = csharp::security::add_security(&opts)?;
            println!(
                "\n✅ {} dosya oluşturuldu, {} dosya güncellendi.",
                res.files_created.len(),
                res.files_modified.len()
            );
        }
        Commands::Docs { lang, project } => {
            let docs = docs_provider::get_language_docs(lang.as_deref(), project.as_deref())?;
            println!("{}", docs);
        }
        Commands::Mcp { profile } => {
            let rules = rules::Rules::load(Path::new("."))?;
            let chosen = profile
                .or_else(|| std::env::var("CLRINF_MCP_PROFILE").ok())
                .or_else(|| rules.as_ref().and_then(|r| r.file.mcp.profile.clone()));
            let profile = match chosen {
                Some(name) => mcp::McpProfile::parse(&name)?,
                None => mcp::McpProfile::Full,
            };
            eprintln!(
                "[clrinf-meta-mcp] Starting Model Context Protocol stdio server (profile: {})...",
                profile.name()
            );
            let server = McpServer::new().with_profile(profile);
            server.run_stdio()?;
        }
        Commands::Hook(command) => match command {
            HookCommands::Install { agent, path, command, force } => {
                let all = agent.eq_ignore_ascii_case("all");
                if all || agent.eq_ignore_ascii_case("claude") {
                    if hook::install_claude(&path, &command)? {
                        println!("✅ Claude Code hook installed in {}", path.join(".claude/settings.json").display());
                    } else {
                        println!("ℹ️  Claude Code hook already installed.");
                    }
                }
                if all || agent.eq_ignore_ascii_case("git") {
                    let hook_path = hook::install_git(&path, &command, force)?;
                    println!("✅ Git pre-commit hook installed at {}", hook_path.display());
                }
                if !all && !["claude", "git"].contains(&agent.to_ascii_lowercase().as_str()) {
                    anyhow::bail!("Unknown --agent '{agent}'; use claude, git or all");
                }
            }
            HookCommands::Uninstall { agent, path } => {
                let all = agent.eq_ignore_ascii_case("all");
                let mut removed = false;
                if all || agent.eq_ignore_ascii_case("claude") {
                    removed |= hook::uninstall_claude(&path)?;
                }
                if all || agent.eq_ignore_ascii_case("git") {
                    removed |= hook::uninstall_git(&path)?;
                }
                println!("{}", if removed { "✅ Hooks removed." } else { "ℹ️  No clrinf hooks found." });
            }
            HookCommands::Run { path } => {
                let stdin = std::io::read_to_string(std::io::stdin()).unwrap_or_default();
                let (code, message) = hook::run(&path, &stdin);
                if !message.is_empty() {
                    eprint!("{message}");
                }
                if code != 0 {
                    std::process::exit(code);
                }
            }
        },
        Commands::Verify { path, changed, json } => {
            let only = if changed { Some(hook::changed_files(&path)) } else { None };
            let report = verify::run(&path, only.as_deref())?;
            if json {
                println!("{}", serde_json::to_string_pretty(&report)?);
            } else {
                print!("{}", report.format_text(false));
                println!(
                    "{} verify: {} check(s) run, {} error(s).",
                    if report.is_valid { "✅" } else { "❌" },
                    report.checks.len(),
                    report.errors_count()
                );
            }
            if !report.is_valid {
                anyhow::bail!("Verification failed with {} error(s).", report.errors_count());
            }
        }
        Commands::Bench(BenchCommands::Context { event, schema_dir, src, path, json }) => {
            let rules = rules::Rules::load(&path)?;
            let schema_dir = schema_dir.unwrap_or_else(|| {
                rules.as_ref().map(|r| r.schema_dir()).unwrap_or_else(|| path.join(rules::DEFAULT_SCHEMA_DIR))
            });
            let roots = if src.is_empty() { vec![path.clone()] } else { src };
            let result = bench::context(&schema_dir, &event, &roots, rules.as_ref())?;
            if json {
                println!("{}", serde_json::to_string_pretty(&result)?);
            } else {
                println!("{}", result.format_text());
            }
        }
        Commands::Plan { event_type, service, role, schema_dir, src, apply, path } => {
            let role = match role.to_ascii_lowercase().as_str() {
                "subscriber" | "subscribe" => event_scaffold::Role::Subscriber,
                "publisher" | "publish" => event_scaffold::Role::Publisher,
                other => anyhow::bail!("Unknown --role '{other}'; use subscriber or publisher"),
            };
            let rules = rules::Rules::load(&path)?;
            let schema_dir = schema_dir.unwrap_or_else(|| {
                rules.as_ref().map(|r| r.schema_dir()).unwrap_or_else(|| path.join(rules::DEFAULT_SCHEMA_DIR))
            });
            let roots = if src.is_empty() { vec![path.clone()] } else { src };
            let plan = plan::plan_event(&schema_dir, &event_type, &service, role, &roots, apply, rules.as_ref())?;
            println!("{}", serde_json::to_string_pretty(&plan)?);
        }
        Commands::Agents(AgentsCommands::Sync { path, check }) => {
            let rules = rules::Rules::load(&path)?;
            let results = agents::sync(&path, rules.as_ref(), check)?;
            let mut stale = 0;
            for (file, status) in &results {
                let label = match status {
                    agents::Status::Created => "created",
                    agents::Status::Updated => "updated",
                    agents::Status::Unchanged => "unchanged",
                };
                if *status != agents::Status::Unchanged {
                    stale += 1;
                }
                println!("{label:>9}  {}", file.display());
            }
            if check && stale > 0 {
                anyhow::bail!("{stale} agent instruction file(s) out of date; run `clrinf agents sync`.");
            }
        }
    }

    Ok(())
}

fn execute_module_adopt(
    module: String,
    to_project: PathBuf,
    target_dir: Option<PathBuf>,
    mode: String,
    r#as: Option<String>,
    source_project: Option<PathBuf>,
) -> Result<()> {
    let adopt_mode = match mode.to_lowercase().as_str() {
        "wired" | "integrated" | "hooked" => AdoptMode::Wired,
        _ => AdoptMode::Raw,
    };
    let opts = ModuleAdoptOptions {
        module: module.clone(),
        to_project,
        target_dir,
        mode: adopt_mode,
        r#as,
        source_project,
    };
    let result = ModuleAdapter::adopt(&opts)?;
    println!("🌿 Modül başarıyla nakledildi: '{}' ({})", result.module_name, result.mode.to_uppercase());
    println!("   Hedef Dil: {}", result.lang);
    println!("   Hedef Dizin: {}", result.target_dir.display());
    if let Some(ref a) = result.alias {
        println!("   Takma Ad (Alias): {}", a);
    }
    if !result.files_created.is_empty() {
        println!("   Oluşturulan Dosyalar ({}):", result.files_created.len());
        for f in &result.files_created {
            println!("     • {}", f.display());
        }
    }
    if !result.files_modified.is_empty() {
        println!("   Güncellenen/Bağlanan Dosyalar ({}):", result.files_modified.len());
        for f in &result.files_modified {
            println!("     • {}", f.display());
        }
    }
    if let Some(ref p) = result.policy_created {
        println!("   🛡️  Multi-Tenant Cedar Yetkilendirme Politikası:");
        println!("     • {}", p.display());
    }
    for note in &result.notes {
        println!("   💡 {}", note);
    }
    Ok(())
}

