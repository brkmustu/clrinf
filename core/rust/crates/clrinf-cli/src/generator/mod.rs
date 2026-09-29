use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ArchStyle {
    Flat,
    Layered,
    CleanCqrs,
}

impl ArchStyle {
    pub fn parse_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "flat" => Self::Flat,
            "layered" => Self::Layered,
            _ => Self::CleanCqrs,
        }
    }
}

pub mod entity;
pub mod module;

pub use entity::EntityGenerator;
pub use module::ModuleGenerator;

pub struct Generator;

impl Generator {
    pub fn add_module(project_root: &Path, module_name: &str) -> std::io::Result<Vec<String>> {
        ModuleGenerator::generate(project_root, module_name)
    }

    pub fn add_entity(
        project_root: &Path,
        module_name: &str,
        entity_name: &str,
        properties: &[(String, String)],
    ) -> std::io::Result<Vec<String>> {
        EntityGenerator::generate(project_root, module_name, entity_name, properties)
    }
    pub fn new_project(
        project_name: &str,
        target_dir: &Path,
        arch: ArchStyle,
        host: &str,
    ) -> std::io::Result<()> {
        let root = target_dir.join(project_name);
        fs::create_dir_all(&root)?;

        match arch {
            ArchStyle::Flat => Self::create_flat_project(project_name, &root, host)?,
            ArchStyle::Layered => Self::create_layered_project(project_name, &root, host)?,
            ArchStyle::CleanCqrs => Self::create_clean_cqrs_project(project_name, &root, host)?,
        }

        Ok(())
    }

    fn create_flat_project(name: &str, root: &Path, _host: &str) -> std::io::Result<()> {
        let cargo_toml = format!(
            r#"[package]
name = "{name}"
version = "0.1.0"
edition = "2021"

[dependencies]
tokio = {{ version = "1.0", features = ["full"] }}
serde = {{ version = "1.0", features = ["derive"] }}
serde_json = "1.0"
axum = "0.7"
async-trait = "0.1"
"#
        );
        fs::write(root.join("Cargo.toml"), cargo_toml)?;

        let src = root.join("src");
        fs::create_dir_all(&src)?;

        let main_rs = format!(
            r#"use axum::{{routing::get, Json, Router}};
use std::net::SocketAddr;

mod models;
mod routes;
mod rules;

#[tokio::main]
async fn main() {{
    let app = Router::new()
        .route("/health", get(|| async {{ Json(serde_json::json!({{"status": "healthy"}})) }}))
        .merge(routes::routes());

    let addr = SocketAddr::from(([0, 0, 0, 0], 8080));
    println!("Server running on http://{{}}", addr);
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}}
"#
        );
        fs::write(src.join("main.rs"), main_rs)?;

        let models_rs = r#"use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Product {
    pub id: String,
    pub name: String,
    pub price: f64,
}
"#;
        fs::write(src.join("models.rs"), models_rs)?;

        let routes_rs = r#"use axum::{routing::get, Json, Router};
use serde_json::{json, Value};

pub fn routes() -> Router {
    Router::new()
        .route("/api/products", get(list_products))
}

async fn list_products() -> Json<Value> {
    Json(json!([
        { "id": "1", "name": "Rust in Action", "price": 45.0 }
    ]))
}
"#;
        fs::write(src.join("routes.rs"), routes_rs)?;

        let rules_rs = r#"// In-process business rule definitions
pub trait BusinessRule<T> {
    fn check(&self, item: &T) -> Result<(), String>;
}
"#;
        fs::write(src.join("rules.rs"), rules_rs)?;

        Ok(())
    }

    fn create_layered_project(name: &str, root: &Path, _host: &str) -> std::io::Result<()> {
        let root_cargo = format!(
            r#"[workspace]
members = [
    "crates/{name}-core",
    "crates/{name}-server",
]
resolver = "2"
"#
        );
        fs::write(root.join("Cargo.toml"), root_cargo)?;

        let crates = root.join("crates");

        // Core crate
        let core_dir = crates.join(format!("{name}-core"));
        fs::create_dir_all(core_dir.join("src"))?;
        let core_cargo = format!(
            r#"[package]
name = "{name}-core"
version = "0.1.0"
edition = "2021"

[dependencies]
serde = {{ version = "1.0", features = ["derive"] }}
serde_json = "1.0"
async-trait = "0.1"
"#
        );
        fs::write(core_dir.join("Cargo.toml"), core_cargo)?;

        let core_lib = r#"pub mod models;
pub mod rules;
pub mod ports;

pub use models::*;
"#;
        fs::write(core_dir.join("src/lib.rs"), core_lib)?;

        let models_rs = r#"use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Product {
    pub id: String,
    pub name: String,
    pub price: f64,
}
"#;
        fs::write(core_dir.join("src/models.rs"), models_rs)?;

        let rules_rs = r#"pub trait BusinessRule<T>: Send + Sync {
    fn evaluate(&self, context: &T) -> Result<(), String>;
}
"#;
        fs::write(core_dir.join("src/rules.rs"), rules_rs)?;

        let ports_rs = r#"use async_trait::async_trait;
use crate::models::Product;

#[async_trait]
pub trait ProductRepository: Send + Sync {
    async fn find_by_id(&self, id: &str) -> Option<Product>;
}
"#;
        fs::write(core_dir.join("src/ports.rs"), ports_rs)?;

        // Server crate
        let server_dir = crates.join(format!("{name}-server"));
        fs::create_dir_all(server_dir.join("src"))?;
        let server_cargo = format!(
            r#"[package]
name = "{name}-server"
version = "0.1.0"
edition = "2021"

[dependencies]
{name}-core = {{ path = "../{name}-core" }}
tokio = {{ version = "1.0", features = ["full"] }}
axum = "0.7"
serde = {{ version = "1.0", features = ["derive"] }}
serde_json = "1.0"
"#
        );
        fs::write(server_dir.join("Cargo.toml"), server_cargo)?;

        let server_main = format!(
            r#"use axum::{{routing::get, Json, Router}};
use std::net::SocketAddr;
use {name}_core::Product;

#[tokio::main]
async fn main() {{
    let app = Router::new()
        .route("/health", get(|| async {{ Json(serde_json::json!({{"status": "healthy"}})) }}))
        .route("/api/products", get(list_products));

    let addr = SocketAddr::from(([0, 0, 0, 0], 8080));
    println!("Server running on http://{{}}", addr);
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}}

async fn list_products() -> Json<Vec<Product>> {{
    Json(vec![Product {{
        id: "1".into(),
        name: "Idiomatic Rust".into(),
        price: 55.0,
    }}])
}}
"#
        );
        fs::write(server_dir.join("src/main.rs"), server_main)?;

        Ok(())
    }

    fn create_clean_cqrs_project(name: &str, root: &Path, _host: &str) -> std::io::Result<()> {
        let root_cargo = format!(
            r#"[workspace]
members = [
    "crates/{name}-domain",
    "crates/{name}-application",
    "crates/{name}-infra",
    "crates/{name}-api",
]
resolver = "2"
"#
        );
        fs::write(root.join("Cargo.toml"), root_cargo)?;

        let crates = root.join("crates");

        // 1. Domain crate (Zero dependencies, pure Rust)
        let domain_dir = crates.join(format!("{name}-domain"));
        fs::create_dir_all(domain_dir.join("src"))?;
        let domain_cargo = format!(
            r#"[package]
name = "{name}-domain"
version = "0.1.0"
edition = "2021"

[dependencies]
# Strictly pure: no web, no database, no external framework dependencies!
"#
        );
        fs::write(domain_dir.join("Cargo.toml"), domain_cargo)?;
        let domain_lib = r#"pub mod entities;
pub mod events;

pub use entities::*;
"#;
        fs::write(domain_dir.join("src/lib.rs"), domain_lib)?;
        let entities_rs = r#"#[derive(Clone, Debug, PartialEq)]
pub struct Product {
    pub id: String,
    pub name: String,
    pub price: f64,
}

impl Product {
    pub fn new(id: impl Into<String>, name: impl Into<String>, price: f64) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            price,
        }
    }
}
"#;
        fs::write(domain_dir.join("src/entities.rs"), entities_rs)?;
        let events_rs = r#"pub enum ProductEvent {
    Created { id: String, name: String },
    PriceUpdated { id: String, new_price: f64 },
}
"#;
        fs::write(domain_dir.join("src/events.rs"), events_rs)?;

        // 2. Application crate (CQRS, Rules, Ports)
        let app_dir = crates.join(format!("{name}-application"));
        fs::create_dir_all(app_dir.join("src/commands"))?;
        fs::create_dir_all(app_dir.join("src/queries"))?;
        fs::create_dir_all(app_dir.join("src/rules"))?;
        let app_cargo = format!(
            r#"[package]
name = "{name}-application"
version = "0.1.0"
edition = "2021"

[dependencies]
{name}-domain = {{ path = "../{name}-domain" }}
serde = {{ version = "1.0", features = ["derive"] }}
serde_json = "1.0"
async-trait = "0.1"
"#
        );
        fs::write(app_dir.join("Cargo.toml"), app_cargo)?;
        let app_lib = r#"pub mod commands;
pub mod queries;
pub mod rules;
pub mod ports;
"#;
        fs::write(app_dir.join("src/lib.rs"), app_lib)?;
        let ports_rs = format!(
            r#"use async_trait::async_trait;
use {name}_domain::Product;

#[async_trait]
pub trait ProductRepository: Send + Sync {{
    async fn get_by_id(&self, id: &str) -> Option<Product>;
    async fn save(&self, product: &Product) -> Result<(), String>;
}}
"#
        );
        fs::write(app_dir.join("src/ports.rs"), ports_rs)?;

        // 3. Infrastructure crate
        let infra_dir = crates.join(format!("{name}-infra"));
        fs::create_dir_all(infra_dir.join("src"))?;
        let infra_cargo = format!(
            r#"[package]
name = "{name}-infra"
version = "0.1.0"
edition = "2021"

[dependencies]
{name}-domain = {{ path = "../{name}-domain" }}
{name}-application = {{ path = "../{name}-application" }}
async-trait = "0.1"
"#
        );
        fs::write(infra_dir.join("Cargo.toml"), infra_cargo)?;
        let infra_lib = format!(
            r#"use async_trait::async_trait;
use std::sync::RwLock;
use std::collections::HashMap;
use {name}_domain::Product;
use {name}_application::ports::ProductRepository;

#[derive(Default)]
pub struct InMemoryProductRepository {{
    items: RwLock<HashMap<String, Product>>,
}}

#[async_trait]
impl ProductRepository for InMemoryProductRepository {{
    async fn get_by_id(&self, id: &str) -> Option<Product> {{
        self.items.read().unwrap().get(id).cloned()
    }}

    async fn save(&self, product: &Product) -> Result<(), String> {{
        self.items.write().unwrap().insert(product.id.clone(), product.clone());
        Ok(())
    }}
}}
"#
        );
        fs::write(infra_dir.join("src/lib.rs"), infra_lib)?;

        // 4. API crate
        let api_dir = crates.join(format!("{name}-api"));
        fs::create_dir_all(api_dir.join("src"))?;
        let api_cargo = format!(
            r#"[package]
name = "{name}-api"
version = "0.1.0"
edition = "2021"

[dependencies]
{name}-domain = {{ path = "../{name}-domain" }}
{name}-application = {{ path = "../{name}-application" }}
{name}-infra = {{ path = "../{name}-infra" }}
tokio = {{ version = "1.0", features = ["full"] }}
axum = "0.7"
serde = {{ version = "1.0", features = ["derive"] }}
serde_json = "1.0"
"#
        );
        fs::write(api_dir.join("Cargo.toml"), api_cargo)?;
        let api_main = format!(
            r#"use axum::{{routing::get, Json, Router}};
use std::net::SocketAddr;
use std::sync::Arc;
use {name}_domain::Product;
use {name}_infra::InMemoryProductRepository;
use {name}_application::ports::ProductRepository;

#[tokio::main]
async fn main() {{
    let repo = Arc::new(InMemoryProductRepository::default());
    repo.save(&Product::new("1", "Clean Rust", 60.0)).await.unwrap();

    let app = Router::new()
        .route("/health", get(|| async {{ Json(serde_json::json!({{"status": "healthy"}})) }}))
        .route("/api/products/1", get({{
            let repo = repo.clone();
            move || {{
                let repo = repo.clone();
                async move {{
                    match repo.get_by_id("1").await {{
                        Some(p) => Json(serde_json::json!(p)),
                        None => Json(serde_json::json!({{"error": "not found"}})),
                    }}
                }}
            }}
        }}));

    let addr = SocketAddr::from(([0, 0, 0, 0], 8080));
    println!("Clean CQRS API running on http://{{}}", addr);
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}}
"#
        );
        fs::write(api_dir.join("src/main.rs"), api_main)?;

        Ok(())
    }

    pub fn add_rule(
        project_path: &Path,
        _module: &str,
        rule_name: &str,
        command_name: &str,
    ) -> std::io::Result<PathBuf> {
        let rules_dir = project_path.join("src").join("rules");
        let target_dir = if rules_dir.exists() {
            rules_dir
        } else {
            // Check crates/<name>-application/src/rules or crates/<name>-core/src/rules
            let crates_dir = project_path.join("crates");
            let mut found = project_path.join("src");
            if crates_dir.exists() {
                if let Ok(entries) = fs::read_dir(crates_dir) {
                    for entry in entries.flatten() {
                        let path = entry.path();
                        let dir_name = path.file_name().unwrap_or_default().to_string_lossy();
                        if dir_name.ends_with("-application") || dir_name.ends_with("-core") {
                            found = path.join("src").join("rules");
                            break;
                        }
                    }
                }
            }
            found
        };

        fs::create_dir_all(&target_dir)?;

        let file_name = format!("{}.rs", to_snake_case(rule_name));
        let file_path = target_dir.join(&file_name);

        let content = format!(
            r#"// Isolated Business Rule: {rule_name}
// Automatically registered or evaluated in the CQRS pipeline.

pub struct {rule_name};

impl {rule_name} {{
    pub fn priority(&self) -> i32 {{
        1
    }}

    pub async fn evaluate(&self, command: &{command_name}) -> Result<(), String> {{
        // TODO: Implement business rule logic
        // if command is invalid, return Err("Error message".into());
        Ok(())
    }}
}}
"#
        );

        fs::write(&file_path, content)?;
        Ok(file_path)
    }
}

fn to_snake_case(s: &str) -> String {
    let mut result = String::new();
    for (i, c) in s.chars().enumerate() {
        if c.is_uppercase() {
            if i > 0 {
                result.push('_');
            }
            result.push(c.to_ascii_lowercase());
        } else {
            result.push(c);
        }
    }
    result
}
