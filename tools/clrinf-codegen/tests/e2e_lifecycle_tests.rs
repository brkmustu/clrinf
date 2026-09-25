// @clrinf:generated — End-to-End Multi-Language Lifecycle Integration Tests
use std::fs;
use std::path::PathBuf;
use std::process::Command;
use tempfile::tempdir;

fn bin_path() -> PathBuf {
    PathBuf::from(env!("CARGO_BIN_EXE_clrinf-codegen"))
}

fn root_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

#[test]
fn test_e2e_rust_lifecycle() {
    let tmp = tempdir().unwrap();
    let project_dir = tmp.path();

    // 1. Setup mock Cargo.toml for brownfield auto-detection
    let cargo_toml = project_dir.join("Cargo.toml");
    fs::write(
        &cargo_toml,
        "[package]\nname = \"store_backend\"\nversion = \"0.1.0\"\nedition = \"2021\"\n",
    )
    .unwrap();

    // 2. clrinf init (auto-detects Rust & store_backend)
    let init_out = Command::new(bin_path())
        .arg("init")
        .arg("--path")
        .arg(project_dir)
        .current_dir(root_dir())
        .output()
        .expect("Failed to run clrinf init");
    assert!(init_out.status.success());
    let init_stdout = String::from_utf8_lossy(&init_out.stdout);
    assert!(init_stdout.contains("Mevcut proje algılandı (Cargo.toml): Dil = rust, Ad = Some(\"store_backend\")"));
    assert!(project_dir.join("clrinf.toml").is_file());

    // 3. clrinf module add caching
    let cache_out = Command::new(bin_path())
        .arg("module")
        .arg("add")
        .arg("caching")
        .arg("--path")
        .arg(project_dir)
        .current_dir(root_dir())
        .output()
        .expect("Failed to add caching");
    assert!(cache_out.status.success());
    assert!(project_dir.join("src/caching/mod.rs").is_file());

    // 4. clrinf module add authorization --provider cedar
    let authz_out = Command::new(bin_path())
        .arg("module")
        .arg("add")
        .arg("authorization")
        .arg("--provider")
        .arg("cedar")
        .arg("--path")
        .arg(project_dir)
        .current_dir(root_dir())
        .output()
        .expect("Failed to add authorization");
    assert!(authz_out.status.success());
    assert!(project_dir.join("src/authz/mod.rs").is_file());
    assert!(project_dir.join("policies/default.cedar").is_file());

    // Verify multi-tenant Cedar policy content
    let cedar_content = fs::read_to_string(project_dir.join("policies/default.cedar")).unwrap();
    assert!(cedar_content.contains("resource.tenant_id != principal.tenant_id"));
    assert!(cedar_content.contains("PlatformAdmin"));

    // Verify Rust module tree wiring in src/lib.rs
    let lib_rs = fs::read_to_string(project_dir.join("src/lib.rs")).unwrap();
    assert!(lib_rs.contains("pub mod caching;"));
    assert!(lib_rs.contains("pub mod authz;"));

    // 5. clrinf add module Orders (domain scaffolding via language worker)
    let mod_out = Command::new(bin_path())
        .arg("add")
        .arg("module")
        .arg("Orders")
        .arg("--path")
        .arg(project_dir)
        .current_dir(root_dir())
        .output()
        .expect("Failed to add domain module");
    assert!(mod_out.status.success());
    assert!(project_dir.join("src/modules/orders/mod.rs").is_file() || project_dir.join("modules/orders/mod.rs").is_file());
    assert!(project_dir.join("policies/orders.cedar").is_file());

    // 6. clrinf add entity OrderItem -m Orders --prop name:string --prop price:f64
    let entity_out = Command::new(bin_path())
        .arg("add")
        .arg("entity")
        .arg("OrderItem")
        .arg("-m")
        .arg("Orders")
        .arg("-p")
        .arg("name:string")
        .arg("-p")
        .arg("price:f64")
        .arg("--path")
        .arg(project_dir)
        .current_dir(root_dir())
        .output()
        .expect("Failed to add entity");
    assert!(entity_out.status.success());

    // 7. clrinf module remove caching
    let rm_cache = Command::new(bin_path())
        .arg("module")
        .arg("remove")
        .arg("caching")
        .arg("--path")
        .arg(project_dir)
        .current_dir(root_dir())
        .output()
        .expect("Failed to remove caching");
    assert!(rm_cache.status.success());
    assert!(!project_dir.join("src/caching/mod.rs").exists());

    // Verify unwired from src/lib.rs
    let lib_rs_after = fs::read_to_string(project_dir.join("src/lib.rs")).unwrap();
    assert!(!lib_rs_after.contains("pub mod caching;"));
    assert!(lib_rs_after.contains("pub mod authz;"));

    // 8. clrinf module sync
    let sync_out = Command::new(bin_path())
        .arg("module")
        .arg("sync")
        .arg("--path")
        .arg(project_dir)
        .current_dir(root_dir())
        .output()
        .expect("Failed to sync modules");
    assert!(sync_out.status.success());
}

#[test]
fn test_e2e_typescript_lifecycle() {
    let tmp = tempdir().unwrap();
    let project_dir = tmp.path();

    // 1. Setup mock package.json for brownfield auto-detection
    let pkg_json = project_dir.join("package.json");
    fs::write(
        &pkg_json,
        r#"{"name": "@mycorp/cart-service", "version": "1.0.0"}"#,
    )
    .unwrap();

    // 2. clrinf init (auto-detects TypeScript & mycorp-cart-service)
    let init_out = Command::new(bin_path())
        .arg("init")
        .arg("--path")
        .arg(project_dir)
        .current_dir(root_dir())
        .output()
        .expect("Failed to run clrinf init");
    assert!(init_out.status.success());
    let init_stdout = String::from_utf8_lossy(&init_out.stdout);
    assert!(init_stdout.contains("Mevcut proje algılandı (package.json): Dil = typescript, Ad = Some(\"mycorp-cart-service\")"));
    assert!(project_dir.join("clrinf.toml").is_file());

    // 3. clrinf module add logging
    let log_out = Command::new(bin_path())
        .arg("module")
        .arg("add")
        .arg("logging")
        .arg("--path")
        .arg(project_dir)
        .current_dir(root_dir())
        .output()
        .expect("Failed to add logging");
    assert!(log_out.status.success());
    assert!(project_dir.join("src/logging/index.ts").is_file());

    // 4. clrinf module add authorization --provider cedar
    let authz_out = Command::new(bin_path())
        .arg("module")
        .arg("add")
        .arg("authorization")
        .arg("--provider")
        .arg("cedar")
        .arg("--path")
        .arg(project_dir)
        .current_dir(root_dir())
        .output()
        .expect("Failed to add authorization");
    assert!(authz_out.status.success());
    assert!(project_dir.join("src/authz/index.ts").is_file());
    assert!(project_dir.join("policies/default.cedar").is_file());

    // Verify TypeScript barrel exports in src/index.ts
    let index_ts = fs::read_to_string(project_dir.join("src/index.ts")).unwrap();
    assert!(index_ts.contains("./logging/index.js"));
    assert!(index_ts.contains("./authz/index.js"));

    // 5. clrinf add module Cart (domain scaffolding via TS language worker)
    let mod_out = Command::new(bin_path())
        .arg("add")
        .arg("module")
        .arg("Cart")
        .arg("--path")
        .arg(project_dir)
        .current_dir(root_dir())
        .output()
        .expect("Failed to add domain module");
    if !mod_out.status.success() {
        panic!("Failed to add TS module: stdout: {}, stderr: {}", String::from_utf8_lossy(&mod_out.stdout), String::from_utf8_lossy(&mod_out.stderr));
    }
    assert!(project_dir.join("src/modules/cart/index.ts").is_file());
    assert!(project_dir.join("policies/cart.cedar").is_file());

    // 6. clrinf add entity CartItem -m Cart --prop sku:string --prop qty:number
    let entity_out = Command::new(bin_path())
        .arg("add")
        .arg("entity")
        .arg("CartItem")
        .arg("-m")
        .arg("Cart")
        .arg("-p")
        .arg("sku:string")
        .arg("-p")
        .arg("qty:number")
        .arg("--path")
        .arg(project_dir)
        .current_dir(root_dir())
        .output()
        .expect("Failed to add entity");
    assert!(entity_out.status.success());
    assert!(project_dir.join("src/modules/cart/cart-item.entity.ts").is_file());

    // 7. clrinf module remove logging
    let rm_log = Command::new(bin_path())
        .arg("module")
        .arg("remove")
        .arg("logging")
        .arg("--path")
        .arg(project_dir)
        .current_dir(root_dir())
        .output()
        .expect("Failed to remove logging");
    assert!(rm_log.status.success());
    assert!(!project_dir.join("src/logging/index.ts").exists());

    // Verify unwired from src/index.ts
    let index_ts_after = fs::read_to_string(project_dir.join("src/index.ts")).unwrap();
    assert!(!index_ts_after.contains("./logging/index.js"));
    assert!(index_ts_after.contains("./authz/index.js"));

    // 8. clrinf module sync
    let sync_out = Command::new(bin_path())
        .arg("module")
        .arg("sync")
        .arg("--path")
        .arg(project_dir)
        .current_dir(root_dir())
        .output()
        .expect("Failed to sync modules");
    assert!(sync_out.status.success());
}

#[test]
fn test_e2e_csharp_lifecycle() {
    let tmp = tempdir().unwrap();
    let project_dir = tmp.path();

    // 1. Setup mock .csproj for brownfield auto-detection
    let csproj = project_dir.join("PaymentService.csproj");
    fs::write(
        &csproj,
        r#"<Project Sdk="Microsoft.NET.Sdk"><PropertyGroup><TargetFramework>net10.0</TargetFramework></PropertyGroup></Project>"#,
    )
    .unwrap();

    // 2. clrinf init (auto-detects C# & PaymentService)
    let init_out = Command::new(bin_path())
        .arg("init")
        .arg("--path")
        .arg(project_dir)
        .current_dir(root_dir())
        .output()
        .expect("Failed to run clrinf init");
    assert!(init_out.status.success());
    let init_stdout = String::from_utf8_lossy(&init_out.stdout);
    assert!(init_stdout.contains("Mevcut proje algılandı (PaymentService.csproj): Dil = csharp, Ad = Some(\"PaymentService\")"));
    assert!(project_dir.join("clrinf.toml").is_file());

    // 3. clrinf module add caching
    let cache_out = Command::new(bin_path())
        .arg("module")
        .arg("add")
        .arg("caching")
        .arg("--path")
        .arg(project_dir)
        .current_dir(root_dir())
        .output()
        .expect("Failed to add caching");
    assert!(cache_out.status.success());
    assert!(project_dir.join("src/Common/Caching/MemoryCacheService.cs").is_file());

    // 4. clrinf module add authorization --provider cedar
    let authz_out = Command::new(bin_path())
        .arg("module")
        .arg("add")
        .arg("authorization")
        .arg("--provider")
        .arg("cedar")
        .arg("--path")
        .arg(project_dir)
        .current_dir(root_dir())
        .output()
        .expect("Failed to add authorization");
    assert!(authz_out.status.success());
    assert!(project_dir.join("src/Common/Authz/CedarPolicyService.cs").is_file());
    assert!(project_dir.join("policies/default.cedar").is_file());

    // Verify multi-tenant Cedar policy content
    let cedar_content = fs::read_to_string(project_dir.join("policies/default.cedar")).unwrap();
    assert!(cedar_content.contains("resource.tenant_id != principal.tenant_id"));
    assert!(cedar_content.contains("PlatformAdmin"));

    // Verify C# service registration in src/Common/ServiceRegistration.cs
    let reg_cs = fs::read_to_string(project_dir.join("src/Common/ServiceRegistration.cs")).unwrap();
    assert!(reg_cs.contains("services.AddClrinfCaching();"));
    assert!(reg_cs.contains("services.AddClrinfCedarAuthorization();"));

    // 5. clrinf module remove caching
    let rm_cache = Command::new(bin_path())
        .arg("module")
        .arg("remove")
        .arg("caching")
        .arg("--path")
        .arg(project_dir)
        .current_dir(root_dir())
        .output()
        .expect("Failed to remove caching");
    assert!(rm_cache.status.success());
    assert!(!project_dir.join("src/Common/Caching").exists());

    // Verify unwired from ServiceRegistration.cs
    let reg_cs_after = fs::read_to_string(project_dir.join("src/Common/ServiceRegistration.cs")).unwrap();
    assert!(!reg_cs_after.contains("services.AddClrinfCaching();"));
    assert!(reg_cs_after.contains("services.AddClrinfCedarAuthorization();"));

    // 6. clrinf module sync
    let sync_out = Command::new(bin_path())
        .arg("module")
        .arg("sync")
        .arg("--path")
        .arg(project_dir)
        .current_dir(root_dir())
        .output()
        .expect("Failed to sync modules");
    assert!(sync_out.status.success());
}

#[test]
fn test_pure_lean_mode_across_languages() {
    let tmp = tempdir().unwrap();
    let project_dir = tmp.path();

    // 1. Initialize with pure / minimal profile (zero cross-cutting concerns)
    let init_out = Command::new(bin_path())
        .arg("init")
        .arg("--path")
        .arg(project_dir)
        .arg("--lang")
        .arg("typescript")
        .arg("--name")
        .arg("LeanShop")
        .arg("--profile")
        .arg("minimal")
        .current_dir(root_dir())
        .output()
        .expect("Failed to run clrinf init --profile minimal");
    assert!(init_out.status.success());

    // Verify manifest has zero optional concerns enabled
    let toml_content = fs::read_to_string(project_dir.join("clrinf.toml")).unwrap();
    assert!(toml_content.contains("caching]\nenabled = false") || toml_content.contains("caching]\nprovider"));
    assert!(!toml_content.contains("caching]\nenabled = true"));
    assert!(!toml_content.contains("logging]\nenabled = true"));
    assert!(!toml_content.contains("authorization]\nenabled = true"));

    // 2. Add domain module Orders in pure mode
    let mod_out = Command::new(bin_path())
        .arg("add")
        .arg("module")
        .arg("Orders")
        .arg("--path")
        .arg(project_dir)
        .current_dir(root_dir())
        .output()
        .expect("Failed to add domain module in pure mode");
    assert!(mod_out.status.success());

    // 3. Add domain entity OrderItem in pure mode
    let entity_out = Command::new(bin_path())
        .arg("add")
        .arg("entity")
        .arg("OrderItem")
        .arg("-m")
        .arg("Orders")
        .arg("-p")
        .arg("sku:string")
        .arg("-p")
        .arg("price:number")
        .arg("--path")
        .arg(project_dir)
        .current_dir(root_dir())
        .output()
        .expect("Failed to add entity in pure mode");
    assert!(entity_out.status.success());

    // Verify that handlers and entities have NO external forced dependencies (@clrinf/core import removed)
    let handlers_ts = fs::read_to_string(project_dir.join("src/modules/orders/handlers.ts")).unwrap();
    assert!(!handlers_ts.contains("from \"@clrinf/core\""));
    assert!(handlers_ts.contains("export interface OperationClaim"));

    let entity_ts = fs::read_to_string(project_dir.join("src/modules/orders/order-item.entity.ts")).unwrap();
    assert!(!entity_ts.contains("from \"@clrinf/core\""));
    assert!(entity_ts.contains("from \"./handlers.js\""));
}

#[test]
fn test_full_batteries_mode_across_languages() {
    let tmp = tempdir().unwrap();
    let project_dir = tmp.path();

    // 1. Initialize with full profile (all 7 cross-cutting concerns enabled)
    let init_out = Command::new(bin_path())
        .arg("init")
        .arg("--path")
        .arg(project_dir)
        .arg("--lang")
        .arg("rust")
        .arg("--name")
        .arg("EnterpriseService")
        .arg("--profile")
        .arg("full")
        .current_dir(root_dir())
        .output()
        .expect("Failed to run clrinf init --profile full");
    assert!(init_out.status.success());

    // 2. Verify all 7 cross-cutting concern files were scaffolded
    assert!(project_dir.join("src/caching/mod.rs").is_file());
    assert!(project_dir.join("src/logging/mod.rs").is_file());
    assert!(project_dir.join("src/transaction/mod.rs").is_file());
    assert!(project_dir.join("src/auth/jwt_service.rs").is_file());
    assert!(project_dir.join("src/authz/mod.rs").is_file());
    assert!(project_dir.join("policies/default.cedar").is_file());

    // 3. Verify Rust compile-time module tree in src/lib.rs has all concerns wired
    let lib_rs = fs::read_to_string(project_dir.join("src/lib.rs")).unwrap();
    assert!(lib_rs.contains("pub mod caching;"));
    assert!(lib_rs.contains("pub mod logging;"));
    assert!(lib_rs.contains("pub mod transaction;"));
    assert!(lib_rs.contains("pub mod auth;"));
    assert!(lib_rs.contains("pub mod authz;"));

    // 4. Status reflects all enabled modules
    let status_out = Command::new(bin_path())
        .arg("status")
        .arg("--path")
        .arg(project_dir)
        .current_dir(root_dir())
        .output()
        .expect("Failed to run status");
    assert!(status_out.status.success());
    let status_str = String::from_utf8_lossy(&status_out.stdout);
    assert!(status_str.contains("caching"));
    assert!(status_str.contains("logging"));
    assert!(status_str.contains("transaction"));
    assert!(status_str.contains("authorization"));
}

#[test]
fn test_custom_provider_isolation() {
    let tmp = tempdir().unwrap();
    let project_dir = tmp.path();

    // 1. Setup mock C# project
    let csproj = project_dir.join("CoreBilling.csproj");
    fs::write(&csproj, "<Project Sdk=\"Microsoft.NET.Sdk\"></Project>").unwrap();

    let init_out = Command::new(bin_path())
        .arg("init")
        .arg("--path")
        .arg(project_dir)
        .current_dir(root_dir())
        .output()
        .expect("Failed to run clrinf init");
    assert!(init_out.status.success());

    // 2. Add caching with custom provider
    let add_custom = Command::new(bin_path())
        .arg("module")
        .arg("add")
        .arg("caching")
        .arg("--provider")
        .arg("custom")
        .arg("--path")
        .arg(project_dir)
        .current_dir(root_dir())
        .output()
        .expect("Failed to add custom caching");
    assert!(add_custom.status.success());

    // Verify manifest has provider = "custom"
    let toml = fs::read_to_string(project_dir.join("clrinf.toml")).unwrap();
    assert!(toml.contains("provider = \"custom\""));

    // 3. User writes their own custom caching code (no @clrinf:generated header)
    let custom_cache_dir = project_dir.join("src/Common/Caching");
    fs::create_dir_all(&custom_cache_dir).unwrap();
    let custom_file = custom_cache_dir.join("UserCustomCache.cs");
    fs::write(
        &custom_file,
        "// Custom user-written cache implementation\npublic class UserCustomCache {}\n",
    )
    .unwrap();

    // 4. Run sync — custom file must remain intact
    let sync_out = Command::new(bin_path())
        .arg("module")
        .arg("sync")
        .arg("--path")
        .arg(project_dir)
        .current_dir(root_dir())
        .output()
        .expect("Failed to sync modules");
    assert!(sync_out.status.success());
    assert!(custom_file.is_file());
    let custom_content = fs::read_to_string(&custom_file).unwrap();
    assert!(custom_content.contains("UserCustomCache"));

    // 5. Remove caching — manifest is disabled, but user's custom file is preserved!
    let rm_out = Command::new(bin_path())
        .arg("module")
        .arg("remove")
        .arg("caching")
        .arg("--path")
        .arg(project_dir)
        .current_dir(root_dir())
        .output()
        .expect("Failed to remove caching");
    assert!(rm_out.status.success());

    // Verify file STILL exists because it was custom!
    assert!(custom_file.is_file());
}

#[test]
fn test_adopt_module_raw_into_csharp_csproj() {
    let temp = tempdir().unwrap();
    let project_dir = temp.path();
    let csproj = project_dir.join("OrderService.csproj");
    fs::write(&csproj, "<Project Sdk=\"Microsoft.NET.Sdk\"></Project>").unwrap();

    let out = Command::new(bin_path())
        .arg("module")
        .arg("adopt")
        .arg("deals")
        .arg("--to-project")
        .arg(&csproj)
        .arg("--as")
        .arg("Sales")
        .arg("--mode")
        .arg("raw")
        .current_dir(root_dir())
        .output()
        .expect("Failed to run module adopt");

    if !out.status.success() {
        panic!(
            "Adopt command failed: stdout: {}, stderr: {}",
            String::from_utf8_lossy(&out.stdout),
            String::from_utf8_lossy(&out.stderr)
        );
    }

    let sales_file = project_dir.join("Features/Sales/SalesModule.cs");
    assert!(sales_file.is_file(), "SalesModule.cs should be created");
    let content = fs::read_to_string(&sales_file).unwrap();
    assert!(content.contains("public sealed record Sale("));
    assert!(content.contains("CreateSaleCommand"));
    assert!(content.contains("public sealed record OperationClaim"));
    assert!(content.contains("ISaleRepository"));
    assert!(content.contains("InMemorySaleRepository"));

    // Multi-tenant Cedar policy
    let cedar_file = project_dir.join("policies/sales.cedar");
    assert!(cedar_file.is_file(), "policies/sales.cedar should be created");
    let cedar_content = fs::read_to_string(&cedar_file).unwrap();
    assert!(cedar_content.contains("ResourceType::\"Sales\""));
    assert!(cedar_content.contains("Role::\"PlatformAdmin\""));
    assert!(cedar_content.contains("Role::\"TenantAdmin\""));
}

#[test]
fn test_adopt_module_raw_into_rust_crate() {
    let temp = tempdir().unwrap();
    let project_dir = temp.path();
    let cargo_toml = project_dir.join("Cargo.toml");
    fs::write(
        &cargo_toml,
        r#"[package]
name = "inventory-worker"
version = "0.1.0"
edition = "2021"
"#,
    )
    .unwrap();

    let out = Command::new(bin_path())
        .arg("module")
        .arg("adopt")
        .arg("contacts")
        .arg("--to-project")
        .arg(&cargo_toml)
        .arg("--mode")
        .arg("raw")
        .current_dir(root_dir())
        .output()
        .expect("Failed to run module adopt");

    assert!(out.status.success());
    let contacts_mod = project_dir.join("src/contacts/mod.rs");
    assert!(contacts_mod.is_file(), "src/contacts/mod.rs should be created");
    let content = fs::read_to_string(&contacts_mod).unwrap();
    assert!(content.contains("pub struct Contact"));
    assert!(content.contains("pub struct CreateContactCommand"));
    assert!(content.contains("pub struct InMemoryContactRepository"));

    let cedar_file = project_dir.join("policies/contacts.cedar");
    assert!(cedar_file.is_file(), "policies/contacts.cedar should be created");
}

#[test]
fn test_adopt_module_wired_into_typescript_package() {
    let temp = tempdir().unwrap();
    let project_dir = temp.path();
    let pkg_json = project_dir.join("package.json");
    fs::write(&pkg_json, r#"{"name": "my-storefront", "version": "1.0.0"}"#).unwrap();

    let src_dir = project_dir.join("src");
    fs::create_dir_all(&src_dir).unwrap();
    let index_ts = src_dir.join("index.ts");
    fs::write(&index_ts, "// Main entry\n").unwrap();

    let out = Command::new(bin_path())
        .arg("module")
        .arg("adopt")
        .arg("activities")
        .arg("--to-project")
        .arg(&pkg_json)
        .arg("--mode")
        .arg("wired")
        .current_dir(root_dir())
        .output()
        .expect("Failed to run module adopt");

    assert!(out.status.success());
    let act_file = project_dir.join("src/modules/activities/index.ts");
    assert!(act_file.is_file(), "activities/index.ts should be created");

    // In wired mode, src/index.ts should have barrel export
    let index_content = fs::read_to_string(&index_ts).unwrap();
    assert!(
        index_content.contains("export * as activities"),
        "index.ts must contain barrel export for activities: {}",
        index_content
    );
}

#[test]
fn test_adopt_module_custom_target_dir_and_caching() {
    let temp = tempdir().unwrap();
    let project_dir = temp.path();
    let csproj = project_dir.join("App.csproj");
    fs::write(&csproj, "<Project Sdk=\"Microsoft.NET.Sdk\"></Project>").unwrap();

    let out = Command::new(bin_path())
        .arg("module")
        .arg("adopt")
        .arg("caching")
        .arg("--to-project")
        .arg(&csproj)
        .arg("--target-dir")
        .arg("src/Infrastructure/Cache")
        .arg("--mode")
        .arg("raw")
        .current_dir(root_dir())
        .output()
        .expect("Failed to run module adopt");

    assert!(out.status.success());
    let cache_file = project_dir.join("src/Infrastructure/Cache/CacheService.cs");
    assert!(cache_file.is_file(), "Custom placed CacheService.cs must exist");
    let content = fs::read_to_string(&cache_file).unwrap();
    assert!(content.contains("public interface ICacheService"));
    assert!(content.contains("MemoryCacheService : ICacheService"));
}

#[test]
fn test_catalog_command() {
    // Test both top-level 'clrinf catalog' and 'clrinf module catalog'
    let out1 = Command::new(bin_path())
        .arg("catalog")
        .current_dir(root_dir())
        .output()
        .expect("Failed to run clrinf catalog");
    assert!(out1.status.success());
    let stdout1 = String::from_utf8_lossy(&out1.stdout);
    assert!(stdout1.contains("clrinf Yerleşik Modül Kataloğu"));
    assert!(stdout1.contains("crm"));
    assert!(stdout1.contains("deals"));
    assert!(stdout1.contains("contacts"));
    assert!(stdout1.contains("activities"));
    assert!(stdout1.contains("authz"));
    assert!(stdout1.contains("caching"));

    let out2 = Command::new(bin_path())
        .arg("module")
        .arg("catalog")
        .current_dir(root_dir())
        .output()
        .expect("Failed to run clrinf module catalog");
    assert!(out2.status.success());
    let stdout2 = String::from_utf8_lossy(&out2.stdout);
    assert!(stdout2.contains("Domain Modülleri"));
    assert!(stdout2.contains("Altyapı Modülleri"));
}

#[test]
fn test_adopt_builtin_crm_suite_raw_into_csharp() {
    let temp = tempdir().unwrap();
    let project_dir = temp.path();
    let csproj = project_dir.join("EnterpriseCrm.csproj");
    fs::write(&csproj, "<Project Sdk=\"Microsoft.NET.Sdk\"></Project>").unwrap();

    let out = Command::new(bin_path())
        .arg("module")
        .arg("adopt")
        .arg("crm")
        .arg("--to-project")
        .arg(&csproj)
        .arg("--target-dir")
        .arg("src/Features/Crm")
        .arg("--mode")
        .arg("raw")
        .current_dir(root_dir())
        .output()
        .expect("Failed to run module adopt crm");

    assert!(out.status.success());
    let deals_file = project_dir.join("src/Features/Crm/Deals/DealsModule.cs");
    let contacts_file = project_dir.join("src/Features/Crm/Contacts/ContactsModule.cs");
    let activities_file = project_dir.join("src/Features/Crm/Activities/ActivitiesModule.cs");
    let cedar_file = project_dir.join("policies/crm.cedar");

    assert!(deals_file.is_file(), "DealsModule.cs should exist in CRM suite");
    assert!(contacts_file.is_file(), "ContactsModule.cs should exist in CRM suite");
    assert!(activities_file.is_file(), "ActivitiesModule.cs should exist in CRM suite");
    assert!(cedar_file.is_file(), "policies/crm.cedar should exist for CRM suite");

    let cedar_content = fs::read_to_string(&cedar_file).unwrap();
    assert!(cedar_content.contains("Action::\"deals.read\""));
    assert!(cedar_content.contains("Action::\"contacts.read\""));
    assert!(cedar_content.contains("Action::\"activities.read\""));
}

#[test]
fn test_adopt_builtin_crm_suite_wired_into_rust() {
    let temp = tempdir().unwrap();
    let project_dir = temp.path();
    let cargo = project_dir.join("Cargo.toml");
    fs::write(
        &cargo,
        r#"[package]
name = "crm-service"
version = "0.1.0"
edition = "2021"
"#,
    )
    .unwrap();

    let out = Command::new(bin_path())
        .arg("module")
        .arg("adopt")
        .arg("crm")
        .arg("--to-project")
        .arg(&cargo)
        .arg("--mode")
        .arg("wired")
        .current_dir(root_dir())
        .output()
        .expect("Failed to run module adopt crm wired");

    assert!(out.status.success());
    let crm_mod = project_dir.join("src/crm/mod.rs");
    let deals_mod = project_dir.join("src/crm/deals/mod.rs");
    let contacts_mod = project_dir.join("src/crm/contacts/mod.rs");
    let activities_mod = project_dir.join("src/crm/activities/mod.rs");
    let lib_rs = project_dir.join("src/lib.rs");
    let cedar_file = project_dir.join("policies/crm.cedar");

    assert!(crm_mod.is_file(), "src/crm/mod.rs must exist");
    assert!(deals_mod.is_file(), "src/crm/deals/mod.rs must exist");
    assert!(contacts_mod.is_file(), "src/crm/contacts/mod.rs must exist");
    assert!(activities_mod.is_file(), "src/crm/activities/mod.rs must exist");
    assert!(lib_rs.is_file(), "src/lib.rs must exist");
    assert!(cedar_file.is_file(), "policies/crm.cedar must exist");

    let lib_content = fs::read_to_string(&lib_rs).unwrap();
    assert!(lib_content.contains("pub mod crm;"), "src/lib.rs should declare pub mod crm;");
}

#[test]
fn test_e2e_clrinf_new_csharp_project() {
    let tmp = tempdir().unwrap();
    let target_dir = tmp.path();

    let out = Command::new(bin_path())
        .arg("new")
        .arg("SampleService")
        .arg("--lang")
        .arg("csharp")
        .arg("--arch")
        .arg("flat")
        .arg("--paradigm")
        .arg("fp")
        .arg("--dispatcher")
        .arg("native")
        .current_dir(target_dir)
        .output()
        .expect("Failed to run clrinf new");

    assert!(out.status.success());
    let proj_dir = target_dir.join("SampleService");
    assert!(proj_dir.join("clrinf.toml").is_file());
    assert!(proj_dir.join("Directory.Build.props").is_file());
    assert!(proj_dir.join("src/SampleService/Data/BaseDbContext.cs").is_file());
    assert!(proj_dir.join("src/SampleService/Program.cs").is_file());
    assert!(proj_dir.join("src/SampleService/Common/Functional/Result.cs").is_file());
    assert!(proj_dir.join("src/SampleService/Common/Pipeline/ICommand.cs").is_file());

    let toml_str = fs::read_to_string(proj_str(&proj_dir.join("clrinf.toml"))).unwrap();
    assert!(toml_str.contains("arch = \"flat\""));
    assert!(toml_str.contains("paradigm = \"fp\""));
}

#[test]
fn test_e2e_clrinf_generate_all_crm_monolith() {
    let tmp = tempdir().unwrap();
    let target_dir = tmp.path();

    let codegen_toml_source = root_dir().join("clrinfcs/codegen.toml");
    let templates_dir = root_dir().join("tools/clrinf-codegen/templates");

    // Pre-create the DbContext so DbSet injection can succeed
    let src_data = target_dir.join("src").join("Data");
    fs::create_dir_all(&src_data).unwrap();
    fs::write(
        src_data.join("CrmDbContext.cs"),
        "using Microsoft.EntityFrameworkCore;\n\nnamespace CrmMonolith.Data;\n\npublic class CrmDbContext : DbContext\n{\n}\n",
    ).unwrap();

    let out = Command::new(bin_path())
        .arg("generate-all")
        .arg("--config")
        .arg(&codegen_toml_source)
        .arg("--templates-dir")
        .arg(&templates_dir)
        .arg("--path")
        .arg(target_dir)
        .current_dir(root_dir())
        .output()
        .expect("Failed to run generate-all");

    let stdout = String::from_utf8_lossy(&out.stdout);
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(out.status.success(), "generate-all failed: stdout: {}, stderr: {}", stdout, stderr);

    // Verify Deal module
    let deal_dir = target_dir.join("src/Modules/Deal");
    assert!(deal_dir.join("Deal.cs").is_file(), "Deal.cs must exist");
    assert!(deal_dir.join("Create.cs").is_file(), "Create.cs must exist in Deal");
    assert!(deal_dir.join("Update.cs").is_file(), "Update.cs must exist in Deal");
    assert!(deal_dir.join("Delete.cs").is_file(), "Delete.cs must exist in Deal");
    assert!(deal_dir.join("GetById.cs").is_file(), "GetById.cs must exist in Deal");
    assert!(deal_dir.join("GetList.cs").is_file(), "GetList.cs must exist in Deal");

    // Verify Contact module
    let contact_dir = target_dir.join("src/Modules/Contact");
    assert!(contact_dir.join("Contact.cs").is_file(), "Contact.cs must exist");
    assert!(contact_dir.join("Create.cs").is_file(), "Create.cs must exist in Contact");

    // Verify Activity module
    let activity_dir = target_dir.join("src/Modules/Activity");
    assert!(activity_dir.join("Activity.cs").is_file(), "Activity.cs must exist");

    // Verify Security module
    assert!(target_dir.join("src/Domain/Entities/User.cs").is_file(), "User.cs must exist");
    assert!(target_dir.join("src/Application/Services/Repositories/IUserRepository.cs").is_file());

    // Verify DbSet injection into CrmDbContext.cs
    let db_content = fs::read_to_string(src_data.join("CrmDbContext.cs")).unwrap();
    assert!(db_content.contains("DbSet<Deal> Deals"), "Deals DbSet should be injected");
    assert!(db_content.contains("DbSet<Contact> Contacts"), "Contacts DbSet should be injected");
    assert!(db_content.contains("DbSet<Activity> Activities"), "Activities DbSet should be injected");
    assert!(db_content.contains("DbSet<User> Users"), "Users DbSet should be injected");

    // Verify that Deal.cs has string Id and id_default
    let deal_content = fs::read_to_string(deal_dir.join("Deal.cs")).unwrap();
    assert!(deal_content.contains("public string Id { get; set; } = Guid.NewGuid().ToString();"));
    assert!(!deal_content.contains("{%"), "No Tera template leakage in Deal.cs");
}

fn proj_str(path: &std::path::Path) -> std::path::PathBuf {
    path.to_path_buf()
}



