// @clrinf:generated — Module manifest parser for clrinf.toml
//
// Parses the language-agnostic project manifest (clrinf.toml) that declares
// which cross-cutting concern modules are enabled and their provider choices.
// This manifest drives code generation across all supported languages.

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

// ─── Project Section ────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectManifest {
    pub project: ProjectConfig,
    #[serde(default)]
    pub modules: ModulesConfig,
    #[serde(default)]
    pub declared_modules: Vec<DeclarativeModule>,
    #[serde(default)]
    pub ui: Option<UiConfig>,
}

impl Default for ProjectManifest {
    fn default() -> Self {
        Self {
            project: ProjectConfig::default(),
            modules: ModulesConfig::default(),
            declared_modules: Vec::new(),
            ui: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectConfig {
    pub name: String,
    pub lang: Lang,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub src_path: Option<String>,
    #[serde(default = "default_arch")]
    pub arch: ArchStyle,
    #[serde(default = "default_deployment")]
    pub deployment: DeploymentMode,
    #[serde(default = "default_host_type")]
    pub host_type: HostType,
    #[serde(default = "default_dispatcher")]
    pub dispatcher: Dispatcher,
    #[serde(default = "default_paradigm")]
    pub paradigm: Paradigm,
    #[serde(default = "default_api_style")]
    pub api_style: ApiStyle,
    #[serde(default = "default_pattern")]
    pub pattern: PatternStyle,
    #[serde(default)]
    pub db_context: Option<String>,
    #[serde(default)]
    pub namespace: Option<String>,
    #[serde(default)]
    pub structure_mode: Option<String>,
    #[serde(default)]
    pub folder_name: Option<String>,
    #[serde(default)]
    pub secured: bool,
}

impl Default for ProjectConfig {
    fn default() -> Self {
        Self {
            name: "App".to_string(),
            lang: Lang::CSharp,
            description: None,
            src_path: None,
            arch: ArchStyle::default(),
            deployment: DeploymentMode::default(),
            host_type: HostType::default(),
            dispatcher: Dispatcher::default(),
            paradigm: Paradigm::default(),
            api_style: ApiStyle::default(),
            pattern: PatternStyle::default(),
            db_context: None,
            namespace: None,
            structure_mode: None,
            folder_name: None,
            secured: false,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Lang {
    CSharp,
    Rust,
    TypeScript,
    Elixir,
}

impl Default for Lang {
    fn default() -> Self {
        Lang::CSharp
    }
}

impl Lang {
    pub fn as_str(&self) -> &'static str {
        match self {
            Lang::CSharp => "csharp",
            Lang::Rust => "rust",
            Lang::TypeScript => "typescript",
            Lang::Elixir => "elixir",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ArchStyle {
    Flat,
    Layered,
    CleanCqrs,
}

impl Default for ArchStyle {
    fn default() -> Self {
        ArchStyle::CleanCqrs
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum DeploymentMode {
    Monolith,
    Distributed,
}

impl Default for DeploymentMode {
    fn default() -> Self {
        DeploymentMode::Monolith
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum HostType {
    Api,
    Mvc,
    Console,
    Worker,
}

impl Default for HostType {
    fn default() -> Self {
        HostType::Api
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Dispatcher {
    Native,
    Mediatr,
    #[serde(rename = "mediatornet")]
    MediatorNet,
}

impl Dispatcher {
    pub fn as_str(&self) -> &'static str {
        match self {
            Dispatcher::Native => "native",
            Dispatcher::Mediatr => "mediatr",
            Dispatcher::MediatorNet => "mediatornet",
        }
    }
}

impl Default for Dispatcher {
    fn default() -> Self {
        Dispatcher::Native
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Paradigm {
    Fp,
    Oop,
}

impl Default for Paradigm {
    fn default() -> Self {
        Paradigm::Fp
    }
}

impl Paradigm {
    pub fn as_str(&self) -> &'static str {
        match self {
            Paradigm::Fp => "fp",
            Paradigm::Oop => "oop",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ApiStyle {
    Minimal,
    Controller,
}

impl Default for ApiStyle {
    fn default() -> Self {
        ApiStyle::Minimal
    }
}

#[allow(dead_code)]
impl ApiStyle {
    pub fn as_str(&self) -> &'static str {
        match self {
            ApiStyle::Minimal => "minimal",
            ApiStyle::Controller => "controller",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum PatternStyle {
    #[serde(alias = "vertical-slice", alias = "vertical_slice")]
    Flat,
    Separated,
    Basic,
}

impl Default for PatternStyle {
    fn default() -> Self {
        PatternStyle::Flat
    }
}

impl PatternStyle {
    pub fn as_str(&self) -> &'static str {
        match self {
            PatternStyle::Flat => "flat",
            PatternStyle::Separated => "separated",
            PatternStyle::Basic => "basic",
        }
    }
}

fn default_arch() -> ArchStyle {
    ArchStyle::CleanCqrs
}
fn default_deployment() -> DeploymentMode {
    DeploymentMode::Monolith
}
fn default_host_type() -> HostType {
    HostType::Api
}
fn default_dispatcher() -> Dispatcher {
    Dispatcher::Native
}
fn default_paradigm() -> Paradigm {
    Paradigm::Fp
}
fn default_api_style() -> ApiStyle {
    ApiStyle::Minimal
}
fn default_pattern() -> PatternStyle {
    PatternStyle::Flat
}
fn default_id_type() -> String {
    "string".to_string()
}

// ─── Declarative Modules Section (codegen.toml / fullstack support) ─────────

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct DeclarativeModule {
    pub name: String,
    #[serde(default = "default_id_type")]
    pub id_type: String,
    #[serde(default = "default_true")]
    pub generate_ui: bool,
    #[serde(default)]
    pub properties: Vec<DeclarativeProperty>,
    #[serde(default)]
    pub backend: Option<DeclarativeBackendModuleConfig>,
    #[serde(default)]
    pub ui: Option<DeclarativeUiModuleConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct DeclarativeUiModuleConfig {
    #[serde(default = "default_true")]
    pub list_page: bool,
    #[serde(default = "default_true")]
    pub detail_page: bool,
    #[serde(default = "default_true")]
    pub form_page: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct UiConfig {
    #[serde(default)]
    pub r#type: Option<String>,
    #[serde(default)]
    pub path: Option<String>,
    #[serde(default)]
    pub base_url: Option<String>,
    #[serde(default)]
    pub http_client: Option<String>,
    #[serde(default)]
    pub state_strategy: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct DeclarativeProperty {
    pub name: String,
    #[serde(rename = "type")]
    pub prop_type: String,
    #[serde(default)]
    pub required: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct DeclarativeBackendModuleConfig {
    #[serde(default = "default_true")]
    pub caching: bool,
    #[serde(default = "default_true")]
    pub logging: bool,
    #[serde(default = "default_true")]
    pub transaction: bool,
    #[serde(default = "default_true")]
    pub secured: bool,
}

// ─── Modules Section ────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ModulesConfig {
    #[serde(default)]
    pub caching: CachingConfig,
    #[serde(default)]
    pub logging: LoggingConfig,
    #[serde(default)]
    pub transaction: TransactionConfig,
    #[serde(default)]
    pub authentication: AuthenticationConfig,
    #[serde(default)]
    pub authorization: AuthorizationConfig,
    #[serde(default)]
    pub error_handling: ErrorHandlingConfig,
    #[serde(default)]
    pub validation: ValidationConfig,
    #[serde(default)]
    pub idempotency: IdempotencyConfig,
    #[serde(default)]
    pub outbox: OutboxConfig,
}

impl ModulesConfig {
    /// Pure / Minimal: All optional cross-cutting concerns are disabled.
    /// Only error handling and validation (core contracts) remain enabled.
    pub fn minimal() -> Self {
        Self {
            caching: CachingConfig { enabled: false, ..Default::default() },
            logging: LoggingConfig { enabled: false, ..Default::default() },
            transaction: TransactionConfig { enabled: false, ..Default::default() },
            authentication: AuthenticationConfig { enabled: false, ..Default::default() },
            authorization: AuthorizationConfig { enabled: false, ..Default::default() },
            error_handling: ErrorHandlingConfig { enabled: true },
            validation: ValidationConfig { enabled: true },
            idempotency: IdempotencyConfig { enabled: false, ..Default::default() },
            outbox: OutboxConfig { enabled: false, ..Default::default() },
        }
    }

    /// Standard: Logging and transaction enabled by default.
    pub fn standard() -> Self {
        Self::default()
    }

    /// Full: All 7 cross-cutting concerns enabled with default providers.
    pub fn full() -> Self {
        Self {
            caching: CachingConfig { enabled: true, ..Default::default() },
            logging: LoggingConfig { enabled: true, ..Default::default() },
            transaction: TransactionConfig { enabled: true, ..Default::default() },
            authentication: AuthenticationConfig { enabled: true, ..Default::default() },
            authorization: AuthorizationConfig { enabled: true, ..Default::default() },
            error_handling: ErrorHandlingConfig { enabled: true },
            validation: ValidationConfig { enabled: true },
            idempotency: IdempotencyConfig { enabled: true, ..Default::default() },
            outbox: OutboxConfig { enabled: true, ..Default::default() },
        }
    }
}

// --- Caching ---

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CachingConfig {
    #[serde(default)]
    pub enabled: bool,
    #[serde(default = "default_caching_provider")]
    pub provider: CachingProvider,
    #[serde(default = "default_cache_ttl")]
    pub default_ttl_seconds: u64,
}

impl Default for CachingConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            provider: CachingProvider::Memory,
            default_ttl_seconds: 300,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum CachingProvider {
    Memory,
    Redis,
    Distributed,
    Custom,
}

fn default_caching_provider() -> CachingProvider {
    CachingProvider::Memory
}
fn default_cache_ttl() -> u64 {
    300
}

// --- Logging ---

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoggingConfig {
    #[serde(default = "default_true")]
    pub enabled: bool,
    #[serde(default = "default_logging_provider")]
    pub provider: LoggingProvider,
}

impl Default for LoggingConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            provider: LoggingProvider::Structured,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum LoggingProvider {
    Structured,
    Console,
    Opentelemetry,
    Custom,
}

fn default_logging_provider() -> LoggingProvider {
    LoggingProvider::Structured
}

// --- Transaction ---

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionConfig {
    #[serde(default = "default_true")]
    pub enabled: bool,
    #[serde(default = "default_transaction_provider")]
    pub provider: TransactionProvider,
}

impl Default for TransactionConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            provider: TransactionProvider::Native,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TransactionProvider {
    Native,
    Saga,
    Custom,
}

fn default_transaction_provider() -> TransactionProvider {
    TransactionProvider::Native
}

// --- Authentication ---

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthenticationConfig {
    #[serde(default)]
    pub enabled: bool,
    #[serde(default = "default_auth_provider")]
    pub provider: AuthProvider,
    #[serde(default)]
    pub jwt: JwtConfig,
}

impl Default for AuthenticationConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            provider: AuthProvider::Jwt,
            jwt: JwtConfig::default(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AuthProvider {
    Jwt,
    Oauth2,
    Apikey,
    Custom,
}

fn default_auth_provider() -> AuthProvider {
    AuthProvider::Jwt
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JwtConfig {
    #[serde(default = "default_jwt_issuer")]
    pub issuer: String,
    #[serde(default = "default_jwt_audience")]
    pub audience: String,
    #[serde(default = "default_jwt_expiration")]
    pub expiration_minutes: u64,
}

impl Default for JwtConfig {
    fn default() -> Self {
        Self {
            issuer: "clrinf".to_string(),
            audience: "clrinf-api".to_string(),
            expiration_minutes: 60,
        }
    }
}

fn default_jwt_issuer() -> String {
    "clrinf".to_string()
}
fn default_jwt_audience() -> String {
    "clrinf-api".to_string()
}
fn default_jwt_expiration() -> u64 {
    60
}

// --- Authorization ---

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthorizationConfig {
    #[serde(default)]
    pub enabled: bool,
    #[serde(default = "default_authz_provider")]
    pub provider: AuthzProvider,
    #[serde(default = "default_policy_path")]
    pub policy_path: String,
}

impl Default for AuthorizationConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            provider: AuthzProvider::Policy,
            policy_path: default_policy_path(),
        }
    }
}

fn default_policy_path() -> String {
    "policies".to_string()
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AuthzProvider {
    Policy,
    Rbac,
    Abac,
    Cedar,
    Custom,
}

fn default_authz_provider() -> AuthzProvider {
    AuthzProvider::Policy
}

// --- Error Handling ---

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorHandlingConfig {
    #[serde(default = "default_true")]
    pub enabled: bool,
}

impl Default for ErrorHandlingConfig {
    fn default() -> Self {
        Self { enabled: true }
    }
}

// --- Validation ---

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationConfig {
    #[serde(default = "default_true")]
    pub enabled: bool,
}

impl Default for ValidationConfig {
    fn default() -> Self {
        Self { enabled: true }
    }
}

// --- Idempotency ---

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IdempotencyConfig {
    #[serde(default)]
    pub enabled: bool,
    #[serde(default = "default_idempotency_provider")]
    pub provider: IdempotencyProvider,
}

impl Default for IdempotencyConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            provider: IdempotencyProvider::Memory,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum IdempotencyProvider {
    Memory,
    Redis,
    Database,
    Custom,
}

fn default_idempotency_provider() -> IdempotencyProvider {
    IdempotencyProvider::Memory
}

// --- Outbox ---

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutboxConfig {
    #[serde(default)]
    pub enabled: bool,
    #[serde(default = "default_outbox_provider")]
    pub provider: OutboxProvider,
}

impl Default for OutboxConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            provider: OutboxProvider::Database,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum OutboxProvider {
    Database,
    Memory,
    Custom,
}

fn default_outbox_provider() -> OutboxProvider {
    OutboxProvider::Database
}

// ─── Shared Defaults ────────────────────────────────────────────────────────

fn default_true() -> bool {
    true
}

#[derive(Debug, Deserialize)]
struct CodegenTomlFile {
    project: Option<CodegenTomlProject>,
    backend: Option<CodegenTomlBackend>,
    ui: Option<CodegenTomlUi>,
    #[serde(default)]
    modules: Vec<CodegenTomlModule>,
}

#[derive(Debug, Deserialize)]
struct CodegenTomlProject {
    name: Option<String>,
    description: Option<String>,
}

#[derive(Debug, Deserialize)]
struct CodegenTomlBackend {
    arch_style: Option<String>,
    paradigm: Option<String>,
    dispatcher: Option<String>,
    api_style: Option<String>,
    pattern: Option<String>,
    secured: Option<bool>,
    path: Option<String>,
    db_context: Option<String>,
    namespace: Option<String>,
    structure: Option<CodegenTomlStructure>,
}

#[derive(Debug, Deserialize)]
struct CodegenTomlStructure {
    mode: Option<String>,
    folder_name: Option<String>,
}

#[derive(Debug, Deserialize)]
struct CodegenTomlUi {
    #[serde(rename = "type")]
    ui_type: Option<String>,
    path: Option<String>,
    base_url: Option<String>,
    http_client: Option<String>,
    state: Option<CodegenTomlUiState>,
}

#[derive(Debug, Deserialize)]
struct CodegenTomlUiState {
    strategy: Option<String>,
}

#[derive(Debug, Deserialize)]
struct CodegenTomlModule {
    name: String,
    #[serde(default = "default_id_type")]
    id_type: String,
    #[serde(default = "default_true")]
    generate_ui: bool,
    #[serde(default)]
    properties: Vec<DeclarativeProperty>,
    backend: Option<DeclarativeBackendModuleConfig>,
    ui: Option<DeclarativeUiModuleConfig>,
}

// ─── Parsing & Discovery ────────────────────────────────────────────────────

impl ProjectManifest {
    /// Parse a manifest from a TOML string.
    pub fn from_toml(content: &str) -> Result<Self> {
        toml::from_str(content).context("Failed to parse clrinf.toml manifest")
    }

    /// Load a manifest from a file path.
    pub fn load(path: &Path) -> Result<Self> {
        let content =
            std::fs::read_to_string(path).with_context(|| format!("Failed to read {}", path.display()))?;
        Self::from_toml(&content)
    }

    /// Parse a manifest from a codegen.toml (ClrinfCS legacy / fullstack format).
    pub fn from_codegen_toml(content: &str) -> Result<Self> {
        let raw: CodegenTomlFile = toml::from_str(content)
            .context("Failed to parse codegen.toml manifest format")?;

        let (project_name, project_desc) = match raw.project.as_ref() {
            Some(p) => (
                p.name.clone().unwrap_or_else(|| "App".to_string()),
                p.description.clone(),
            ),
            None => ("App".to_string(), None),
        };

        let backend = raw.backend.unwrap_or_else(|| CodegenTomlBackend {
            arch_style: None,
            paradigm: None,
            dispatcher: None,
            api_style: None,
            pattern: None,
            secured: None,
            path: None,
            db_context: None,
            namespace: None,
            structure: None,
        });

        let arch = match backend.arch_style.as_deref() {
            Some("flat") => ArchStyle::Flat,
            Some("layered") => ArchStyle::Layered,
            _ => ArchStyle::CleanCqrs,
        };

        let paradigm = match backend.paradigm.as_deref() {
            Some("oop") => Paradigm::Oop,
            _ => Paradigm::Fp,
        };

        let dispatcher = match backend.dispatcher.as_deref() {
            Some("mediatr") => Dispatcher::Mediatr,
            Some("mediatornet") => Dispatcher::MediatorNet,
            _ => Dispatcher::Native,
        };

        let api_style = match backend.api_style.as_deref() {
            Some("controller") => ApiStyle::Controller,
            _ => ApiStyle::Minimal,
        };

        let pattern = match backend.pattern.as_deref() {
            Some("separated") => PatternStyle::Separated,
            Some("basic") => PatternStyle::Basic,
            Some("flat") | Some("vertical-slice") | _ => PatternStyle::Flat,
        };

        let folder_name = backend.structure.as_ref().and_then(|s| s.folder_name.clone());
        let structure_mode = backend.structure.as_ref().and_then(|s| s.mode.clone());

        let declared_modules = raw.modules.into_iter().map(|m| {
            DeclarativeModule {
                name: m.name,
                id_type: m.id_type,
                generate_ui: m.generate_ui,
                properties: m.properties,
                backend: m.backend,
                ui: m.ui,
            }
        }).collect();

        let ui_config = raw.ui.map(|u| UiConfig {
            r#type: u.ui_type,
            path: u.path,
            base_url: u.base_url,
            http_client: u.http_client,
            state_strategy: u.state.and_then(|s| s.strategy),
        });

        Ok(Self {
            project: ProjectConfig {
                name: project_name,
                lang: Lang::CSharp,
                description: project_desc,
                src_path: backend.path,
                arch,
                deployment: DeploymentMode::Monolith,
                host_type: HostType::Api,
                dispatcher,
                paradigm,
                api_style,
                pattern,
                db_context: backend.db_context,
                namespace: backend.namespace,
                structure_mode,
                folder_name,
                secured: backend.secured.unwrap_or(false),
            },
            modules: ModulesConfig::default(),
            declared_modules,
            ui: ui_config,
        })
    }

    /// Load a manifest from a codegen.toml file path.
    pub fn load_from_codegen_toml(path: &Path) -> Result<Self> {
        let content = std::fs::read_to_string(path)
            .with_context(|| format!("Failed to read {}", path.display()))?;
        Self::from_codegen_toml(&content)
    }

    /// Discover and load a manifest by searching the given directory and its parents.
    /// Looks for `clrinf.toml` or `codegen.toml` in the directory hierarchy.
    pub fn discover(start_dir: &Path) -> Result<(Self, PathBuf)> {
        let mut current = Some(start_dir);
        while let Some(dir) = current {
            let clrinf_candidate = dir.join("clrinf.toml");
            if clrinf_candidate.is_file() {
                let manifest = Self::load(&clrinf_candidate)?;
                return Ok((manifest, clrinf_candidate));
            }
            let codegen_candidate = dir.join("codegen.toml");
            if codegen_candidate.is_file() {
                let manifest = Self::load_from_codegen_toml(&codegen_candidate)?;
                return Ok((manifest, codegen_candidate));
            }
            current = dir.parent();
        }
        anyhow::bail!(
            "clrinf.toml or codegen.toml not found in {} or any parent directory. Run 'clrinf init' to create one.",
            start_dir.display()
        );
    }

    /// Returns the list of module names that are enabled.
    pub fn enabled_modules(&self) -> Vec<&'static str> {
        let mut result = Vec::new();
        if self.modules.caching.enabled {
            result.push("caching");
        }
        if self.modules.logging.enabled {
            result.push("logging");
        }
        if self.modules.transaction.enabled {
            result.push("transaction");
        }
        if self.modules.authentication.enabled {
            result.push("authentication");
        }
        if self.modules.authorization.enabled {
            result.push("authorization");
        }
        if self.modules.error_handling.enabled {
            result.push("error_handling");
        }
        if self.modules.validation.enabled {
            result.push("validation");
        }
        if self.modules.idempotency.enabled {
            result.push("idempotency");
        }
        if self.modules.outbox.enabled {
            result.push("outbox");
        }
        result
    }

    /// Serialize the manifest back to TOML format.
    pub fn to_toml(&self) -> Result<String> {
        toml::to_string_pretty(self).context("Failed to serialize manifest to TOML")
    }

    /// Save the manifest to a file.
    pub fn save(&self, path: &Path) -> Result<()> {
        let content = self.to_toml()?;
        std::fs::write(path, content)
            .with_context(|| format!("Failed to write manifest to {}", path.display()))
    }
}

// ─── Brownfield Project Detection ──────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProjectDetection {
    pub lang: Lang,
    pub name: Option<String>,
    pub source_file: String,
}

/// Automatically inspects a directory to detect an existing project's language and name.
/// Searches for Cargo.toml (Rust), package.json (TypeScript), or *.csproj / *.sln (C#).
pub fn detect_project(dir: &Path) -> Option<ProjectDetection> {
    // 1. Rust: Cargo.toml
    let cargo_toml = dir.join("Cargo.toml");
    if cargo_toml.is_file() {
        let name = std::fs::read_to_string(&cargo_toml).ok().and_then(|content| {
            let mut in_package = false;
            for line in content.lines() {
                let trimmed = line.trim();
                if trimmed.starts_with('[') && trimmed.ends_with(']') {
                    in_package = trimmed == "[package]";
                    continue;
                }
                if in_package && trimmed.starts_with("name") && trimmed.contains('=') {
                    let parts: Vec<&str> = trimmed.split('=').collect();
                    if parts.len() == 2 {
                        let val = parts[1].trim().trim_matches('"').trim_matches('\'').trim();
                        if !val.is_empty() {
                            return Some(val.to_string());
                        }
                    }
                }
            }
            None
        });
        return Some(ProjectDetection {
            lang: Lang::Rust,
            name,
            source_file: "Cargo.toml".to_string(),
        });
    }

    // 2. TypeScript / JavaScript: package.json
    let package_json = dir.join("package.json");
    if package_json.is_file() {
        let name = std::fs::read_to_string(&package_json).ok().and_then(|content| {
            serde_json::from_str::<serde_json::Value>(&content).ok().and_then(|v| {
                v.get("name").and_then(|n| n.as_str()).map(|s| {
                    s.trim_start_matches('@').replace('/', "-")
                })
            })
        });
        return Some(ProjectDetection {
            lang: Lang::TypeScript,
            name,
            source_file: "package.json".to_string(),
        });
    }

    // 3. C#: *.csproj or *.sln
    if let Ok(entries) = std::fs::read_dir(dir) {
        for entry in entries.flatten() {
            let p = entry.path();
            if p.is_file() {
                if let Some(ext) = p.extension().and_then(|e| e.to_str()) {
                    if ext.eq_ignore_ascii_case("csproj") || ext.eq_ignore_ascii_case("sln") {
                        let name = p.file_stem().and_then(|s| s.to_str()).map(|s| s.to_string());
                        return Some(ProjectDetection {
                            lang: Lang::CSharp,
                            name,
                            source_file: p.file_name().unwrap().to_string_lossy().to_string(),
                        });
                    }
                }
            }
        }
    }

    // 4. Elixir: mix.exs
    let mix_exs = dir.join("mix.exs");
    if mix_exs.is_file() {
        let name = std::fs::read_to_string(&mix_exs).ok().and_then(|content| {
            for line in content.lines() {
                let trimmed = line.trim().trim_start_matches('[').trim();
                if let Some(rest) = trimmed.strip_prefix("app:") {
                    let app_part = rest
                        .trim()
                        .trim_start_matches(':')
                        .split(',')
                        .next()
                        .unwrap_or("");
                    let app_name = app_part
                        .trim()
                        .trim_matches('"')
                        .trim_matches('\'')
                        .trim();
                    if !app_name.is_empty() {
                        return Some(app_name.to_string());
                    }
                }
            }
            None
        });
        return Some(ProjectDetection {
            lang: Lang::Elixir,
            name,
            source_file: "mix.exs".to_string(),
        });
    }

    None
}

// ─── Tests ──────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_minimal_manifest() {
        let toml = r#"
[project]
name = "MyApp"
lang = "csharp"
"#;
        let manifest = ProjectManifest::from_toml(toml).unwrap();
        assert_eq!(manifest.project.name, "MyApp");
        assert_eq!(manifest.project.lang, Lang::CSharp);
        assert_eq!(manifest.project.arch, ArchStyle::CleanCqrs);
        assert_eq!(manifest.project.deployment, DeploymentMode::Monolith);
        // Default modules
        assert!(!manifest.modules.caching.enabled);
        assert!(manifest.modules.logging.enabled);
        assert!(manifest.modules.transaction.enabled);
        assert!(!manifest.modules.authentication.enabled);
        assert!(manifest.modules.error_handling.enabled);
        assert!(manifest.modules.validation.enabled);
    }

    #[test]
    fn parse_full_manifest() {
        let toml = r#"
[project]
name = "ECommerceApp"
lang = "rust"
arch = "layered"
deployment = "distributed"
host_type = "worker"
dispatcher = "native"

[modules.caching]
enabled = true
provider = "redis"
default_ttl_seconds = 600

[modules.logging]
enabled = true
provider = "opentelemetry"

[modules.transaction]
enabled = true
provider = "saga"

[modules.authentication]
enabled = true
provider = "jwt"

[modules.authentication.jwt]
issuer = "my-issuer"
audience = "my-audience"
expiration_minutes = 120

[modules.authorization]
enabled = true
provider = "rbac"

[modules.idempotency]
enabled = true
provider = "redis"

[modules.outbox]
enabled = true
provider = "database"
"#;
        let manifest = ProjectManifest::from_toml(toml).unwrap();
        assert_eq!(manifest.project.name, "ECommerceApp");
        assert_eq!(manifest.project.lang, Lang::Rust);
        assert_eq!(manifest.project.arch, ArchStyle::Layered);
        assert_eq!(manifest.project.deployment, DeploymentMode::Distributed);
        assert_eq!(manifest.project.host_type, HostType::Worker);
        assert!(manifest.modules.caching.enabled);
        assert_eq!(manifest.modules.caching.provider, CachingProvider::Redis);
        assert_eq!(manifest.modules.caching.default_ttl_seconds, 600);
        assert!(manifest.modules.authentication.enabled);
        assert_eq!(
            manifest.modules.authentication.jwt.issuer,
            "my-issuer"
        );
        assert_eq!(manifest.modules.authentication.jwt.expiration_minutes, 120);
        assert!(manifest.modules.authorization.enabled);
        assert_eq!(manifest.modules.authorization.provider, AuthzProvider::Rbac);
        assert!(manifest.modules.idempotency.enabled);
        assert_eq!(
            manifest.modules.idempotency.provider,
            IdempotencyProvider::Redis
        );
        assert!(manifest.modules.outbox.enabled);
    }

    #[test]
    fn enabled_modules_list() {
        let toml = r#"
[project]
name = "Test"
lang = "typescript"

[modules.caching]
enabled = true

[modules.authentication]
enabled = true

[modules.idempotency]
enabled = true
"#;
        let manifest = ProjectManifest::from_toml(toml).unwrap();
        let enabled = manifest.enabled_modules();
        assert!(enabled.contains(&"caching"));
        assert!(enabled.contains(&"authentication"));
        assert!(enabled.contains(&"idempotency"));
        assert!(enabled.contains(&"logging")); // default true
        assert!(enabled.contains(&"transaction")); // default true
        assert!(!enabled.contains(&"authorization")); // default false
        assert!(!enabled.contains(&"outbox")); // default false
    }

    #[test]
    fn roundtrip_toml() {
        let toml = r#"
[project]
name = "RoundTrip"
lang = "csharp"
arch = "flat"
"#;
        let manifest = ProjectManifest::from_toml(toml).unwrap();
        let serialized = manifest.to_toml().unwrap();
        let reparsed = ProjectManifest::from_toml(&serialized).unwrap();
        assert_eq!(manifest.project.name, reparsed.project.name);
        assert_eq!(manifest.project.lang, reparsed.project.lang);
        assert_eq!(manifest.project.arch, reparsed.project.arch);
    }

    #[test]
    fn invalid_lang_rejected() {
        let toml = r#"
[project]
name = "Bad"
lang = "python"
"#;
        assert!(ProjectManifest::from_toml(toml).is_err());
    }

    #[test]
    fn invalid_arch_rejected() {
        let toml = r#"
[project]
name = "Bad"
lang = "csharp"
arch = "microservices"
"#;
        assert!(ProjectManifest::from_toml(toml).is_err());
    }

    #[test]
    fn test_detect_project_rust() {
        let tmp = tempfile::tempdir().unwrap();
        let cargo_toml = tmp.path().join("Cargo.toml");
        std::fs::write(&cargo_toml, "[package]\nname = \"my_rust_service\"\nversion = \"0.1.0\"\n").unwrap();

        let detection = detect_project(tmp.path()).unwrap();
        assert_eq!(detection.lang, Lang::Rust);
        assert_eq!(detection.name.as_deref(), Some("my_rust_service"));
        assert_eq!(detection.source_file, "Cargo.toml");
    }

    #[test]
    fn test_detect_project_typescript() {
        let tmp = tempfile::tempdir().unwrap();
        let pkg_json = tmp.path().join("package.json");
        std::fs::write(&pkg_json, r#"{"name": "@myorg/cart-service", "version": "1.0.0"}"#).unwrap();

        let detection = detect_project(tmp.path()).unwrap();
        assert_eq!(detection.lang, Lang::TypeScript);
        assert_eq!(detection.name.as_deref(), Some("myorg-cart-service"));
        assert_eq!(detection.source_file, "package.json");
    }

    #[test]
    fn test_detect_project_csharp() {
        let tmp = tempfile::tempdir().unwrap();
        let csproj = tmp.path().join("OrderProcessing.csproj");
        std::fs::write(&csproj, "<Project Sdk=\"Microsoft.NET.Sdk\"></Project>").unwrap();

        let detection = detect_project(tmp.path()).unwrap();
        assert_eq!(detection.lang, Lang::CSharp);
        assert_eq!(detection.name.as_deref(), Some("OrderProcessing"));
        assert_eq!(detection.source_file, "OrderProcessing.csproj");
    }

    #[test]
    fn test_detect_project_elixir() {
        let tmp = tempfile::tempdir().unwrap();
        let mix_exs = tmp.path().join("mix.exs");
        std::fs::write(&mix_exs, "defmodule MyApp.MixProject do\n  def project do\n    [app: :my_elixir_app, version: \"0.1.0\"]\n  end\nend").unwrap();

        let detection = detect_project(tmp.path()).unwrap();
        assert_eq!(detection.lang, Lang::Elixir);
        assert_eq!(detection.name.as_deref(), Some("my_elixir_app"));
        assert_eq!(detection.source_file, "mix.exs");
    }

    #[test]
    fn test_detect_project_none() {
        let tmp = tempfile::tempdir().unwrap();
        assert!(detect_project(tmp.path()).is_none());
    }

    #[test]
    fn test_from_codegen_toml() {
        let toml_str = r#"
[project]
name        = "CrmMonolith"
description = "CRM Modular Monolith Reference Application"

[backend]
secured     = true
arch_style  = "flat"
paradigm    = "fp"
dispatcher  = "native"
api_style   = "minimal"
path        = "./src"
db_context  = "CrmDbContext"
namespace   = "CrmMonolith"

[backend.structure]
mode        = "module"
folder_name = "Modules"

[[modules]]
name        = "Deal"
id_type     = "string"
generate_ui = true

  [modules.backend]
  caching     = true
  logging     = true
  transaction = true
  secured     = true

  [[modules.properties]]
  name     = "Title"
  type     = "string"
  required = true
"#;
        let manifest = ProjectManifest::from_codegen_toml(toml_str).unwrap();
        assert_eq!(manifest.project.name, "CrmMonolith");
        assert_eq!(manifest.project.description.as_deref(), Some("CRM Modular Monolith Reference Application"));
        assert_eq!(manifest.project.src_path.as_deref(), Some("./src"));
        assert_eq!(manifest.project.structure_mode.as_deref(), Some("module"));
        assert_eq!(manifest.project.lang, Lang::CSharp);
        assert_eq!(manifest.project.arch, ArchStyle::Flat);
        assert_eq!(manifest.project.paradigm, Paradigm::Fp);
        assert_eq!(manifest.project.dispatcher, Dispatcher::Native);
        assert_eq!(manifest.project.db_context.as_deref(), Some("CrmDbContext"));
        assert_eq!(manifest.project.namespace.as_deref(), Some("CrmMonolith"));
        assert_eq!(manifest.project.folder_name.as_deref(), Some("Modules"));
        assert!(manifest.project.secured);

        assert_eq!(manifest.declared_modules.len(), 1);
        let deal = &manifest.declared_modules[0];
        assert_eq!(deal.name, "Deal");
        assert_eq!(deal.id_type, "string");
        assert_eq!(deal.properties.len(), 1);
        assert_eq!(deal.properties[0].name, "Title");
        assert_eq!(deal.properties[0].prop_type, "string");
        assert!(deal.properties[0].required);
    }
}


