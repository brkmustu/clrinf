use std::fs;

#[test]
fn test_flat_project_generation() {
    let temp_dir = std::env::temp_dir().join("test_flat_rust_proj");
    let _ = fs::remove_dir_all(&temp_dir);
    fs::create_dir_all(&temp_dir).unwrap();

    // Since generator is internal to clrinf-cli, let's run the binary via Command or test
    let output = std::process::Command::new("cargo")
        .args([
            "run",
            "-p",
            "clrinf-cli",
            "--",
            "new",
            "MyFlatService",
            "--arch",
            "flat",
            "--path",
            temp_dir.to_str().unwrap(),
        ])
        .output()
        .expect("Failed to execute clrinfrs new");

    assert!(output.status.success());
    let proj_root = temp_dir.join("MyFlatService");
    assert!(proj_root.join("Cargo.toml").exists());
    assert!(proj_root.join("src/main.rs").exists());
    assert!(proj_root.join("src/models.rs").exists());
    assert!(proj_root.join("src/routes.rs").exists());

    let _ = fs::remove_dir_all(&temp_dir);
}

#[test]
fn test_clean_cqrs_project_generation() {
    let temp_dir = std::env::temp_dir().join("test_clean_cqrs_rust_proj");
    let _ = fs::remove_dir_all(&temp_dir);
    fs::create_dir_all(&temp_dir).unwrap();

    let output = std::process::Command::new("cargo")
        .args([
            "run",
            "-p",
            "clrinf-cli",
            "--",
            "new",
            "EnterpriseApp",
            "--arch",
            "clean-cqrs",
            "--path",
            temp_dir.to_str().unwrap(),
        ])
        .output()
        .expect("Failed to execute clrinfrs new");

    assert!(output.status.success());
    let proj_root = temp_dir.join("EnterpriseApp");
    assert!(proj_root.join("Cargo.toml").exists());
    assert!(proj_root.join("crates/EnterpriseApp-domain/src/lib.rs").exists());
    assert!(proj_root.join("crates/EnterpriseApp-application/src/lib.rs").exists());
    assert!(proj_root.join("crates/EnterpriseApp-infra/src/lib.rs").exists());
    assert!(proj_root.join("crates/EnterpriseApp-api/src/main.rs").exists());

    // Test linter on this freshly generated clean CQRS project
    let lint_output = std::process::Command::new("cargo")
        .args([
            "run",
            "-p",
            "clrinf-cli",
            "--",
            "lint",
            proj_root.to_str().unwrap(),
            "--strict",
        ])
        .output()
        .expect("Failed to execute clrinfrs lint");

    assert!(lint_output.status.success());

    let _ = fs::remove_dir_all(&temp_dir);
}

#[test]
fn test_ai_rule_generation() {
    let temp_dir = std::env::temp_dir().join("test_ai_rule_rust");
    let _ = fs::remove_dir_all(&temp_dir);
    fs::create_dir_all(&temp_dir).unwrap();

    let output = std::process::Command::new("cargo")
        .args([
            "run",
            "-p",
            "clrinf-cli",
            "--",
            "ai",
            "rule",
            "Fiyat sıfırdan küçük olamaz",
            "-m",
            "Products",
            "-n",
            "PriceMustBePositiveRule",
            "-c",
            "CreateProductCommand",
            "--project",
            temp_dir.to_str().unwrap(),
        ])
        .output()
        .expect("Failed to execute clrinfrs ai rule");

    assert!(output.status.success());
    assert!(temp_dir.join("src/rules/price_must_be_positive_rule.rs").exists());
    assert!(temp_dir.join("tests/price_must_be_positive_rule_test.rs").exists());

    let _ = fs::remove_dir_all(&temp_dir);
}

#[test]
fn test_add_module_and_entity_generation() {
    let temp_dir = std::env::temp_dir().join("test_module_entity_proj");
    let _ = fs::remove_dir_all(&temp_dir);
    fs::create_dir_all(&temp_dir).unwrap();

    // First generate a clean-cqrs project
    let status = std::process::Command::new("cargo")
        .args([
            "run",
            "-p",
            "clrinf-cli",
            "--",
            "new",
            "StoreApp",
            "--arch",
            "clean-cqrs",
            "--path",
            temp_dir.to_str().unwrap(),
        ])
        .status()
        .expect("Failed to new project");
    assert!(status.success());

    let proj_root = temp_dir.join("StoreApp");

    // Add module 'Orders'
    let mod_status = std::process::Command::new("cargo")
        .args([
            "run",
            "-p",
            "clrinf-cli",
            "--",
            "add",
            "module",
            "Orders",
            "--project",
            proj_root.to_str().unwrap(),
        ])
        .status()
        .expect("Failed to add module");
    assert!(mod_status.success());

    let module_mod_exists = proj_root.join("src/modules/orders/mod.rs").exists()
        || proj_root.join("crates/StoreApp-core/src/modules/orders/mod.rs").exists()
        || proj_root.join("modules/orders/mod.rs").exists();
    assert!(module_mod_exists, "orders/mod.rs should exist");
    assert!(proj_root.join("policies/orders.cedar").exists(), "orders.cedar should exist");

    // Add entity 'OrderItem' with props
    let ent_status = std::process::Command::new("cargo")
        .args([
            "run",
            "-p",
            "clrinf-cli",
            "--",
            "add",
            "entity",
            "OrderItem",
            "-m",
            "Orders",
            "--prop",
            "name:string",
            "--prop",
            "price:f64",
            "--prop",
            "quantity:i32",
            "--project",
            proj_root.to_str().unwrap(),
        ])
        .status()
        .expect("Failed to add entity");
    assert!(ent_status.success());

    let entity_file_exists = proj_root.join("src/modules/orders/orderitem.rs").exists()
        || proj_root.join("crates/StoreApp-core/src/modules/orders/orderitem.rs").exists()
        || proj_root.join("modules/orders/orderitem.rs").exists();
    assert!(entity_file_exists, "orderitem.rs should exist");

    let _ = fs::remove_dir_all(&temp_dir);
}

