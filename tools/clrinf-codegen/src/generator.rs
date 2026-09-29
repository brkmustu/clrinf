use crate::schema::{parse_schema_file, ParsedSchema};
use crate::validator::{check_schemas, discover_files};
use anyhow::{ensure, Context, Result};
use heck::{ToKebabCase, ToSnakeCase, ToUpperCamelCase};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use tera::{Context as TeraContext, Tera};

pub struct Generator {
    tera: Tera,
}

pub fn ensure_safe_output(output: &Path) -> Result<()> {
    if output.exists() || output.is_symlink() {
        for entry in walkdir::WalkDir::new(output).sort_by_file_name() {
            let entry = entry?;
            ensure!(
                !entry.file_type().is_symlink(),
                "Refusing symlinked generation output: {}",
                entry.path().display()
            );
        }
    }
    Ok(())
}

impl Generator {
    pub fn new(templates_dir: &Path) -> Result<Self> {
        ensure!(templates_dir.is_dir(), "Code templates not found at {}. Pass --templates-dir /path/to/tools/clrinf-codegen/templates; installed CLIs require explicit asset paths outside a checkout.", templates_dir.display());
        let template_pattern = format!("{}/**/*.tera", templates_dir.display());
        let tera = Tera::new(&template_pattern).with_context(|| {
            format!(
                "Failed to initialize Tera with templates from: {}",
                templates_dir.display()
            )
        })?;

        Ok(Self { tera })
    }

    pub fn generate(&self, schema_dir: &Path, lang: &str, output_dir: &Path) -> Result<usize> {
        ensure!(
            matches!(
                lang.to_lowercase().as_str(),
                "rust" | "csharp" | "cs" | "dotnet" | "typescript" | "ts" | "elixir" | "ex"
            ),
            "Unsupported language: {lang}"
        );
        check_schemas(schema_dir)?;
        ensure_safe_output(output_dir)?;
        std::fs::create_dir_all(output_dir).with_context(|| {
            format!(
                "Failed to create output directory: {}",
                output_dir.display()
            )
        })?;

        let schema_files = self.find_schema_files(schema_dir)?;

        let mut count = 0;
        let mut rust_modules = Vec::new();
        let mut ts_modules = Vec::new();

        for file in &schema_files {
            let schema = parse_schema_file(file)?;
            let context = self.create_context(&schema);

            match lang.to_lowercase().as_str() {
                "rust" => {
                    let template_name = "rust/event.tera";
                    let rendered = self.tera.render(template_name, &context)
                        .with_context(|| format!("Failed rendering {} for Rust", schema.title))?;

                    let mod_name = schema.file_stem.to_snake_case();
                    let out_file = output_dir.join(format!("{}.rs", mod_name));
                    std::fs::write(&out_file, rendered)
                        .with_context(|| format!("Failed writing {}", out_file.display()))?;

                    rust_modules.push(mod_name);
                    println!("  [Rust] Generated: {}", out_file.display());
                    count += 1;
                }
                "csharp" | "cs" | "dotnet" => {
                    let template_name = "csharp/event.tera";
                    let rendered = self.tera.render(template_name, &context)
                        .with_context(|| format!("Failed rendering {} for C#", schema.title))?;

                    let out_file = output_dir.join(format!("{}.cs", schema.title));
                    std::fs::write(&out_file, rendered)
                        .with_context(|| format!("Failed writing {}", out_file.display()))?;

                    println!("  [C#] Generated: {}", out_file.display());
                    count += 1;
                }
                "typescript" | "ts" => {
                    let template_name = "typescript/event.tera";
                    let rendered = self.tera.render(template_name, &context)
                        .with_context(|| format!("Failed rendering {} for TypeScript", schema.title))?;

                    let file_name = schema.file_stem.to_kebab_case();
                    let out_file = output_dir.join(format!("{}.ts", file_name));
                    std::fs::write(&out_file, rendered)
                        .with_context(|| format!("Failed writing {}", out_file.display()))?;

                    ts_modules.push(file_name);
                    println!("  [TS] Generated: {}", out_file.display());
                    count += 1;
                }
                "elixir" | "ex" => {
                    let template_name = "elixir/event.tera";
                    let rendered = self.tera.render(template_name, &context)
                        .with_context(|| format!("Failed rendering {} for Elixir", schema.title))?;

                    let file_name = schema.file_stem.to_snake_case();
                    let out_file = output_dir.join(format!("{}.ex", file_name));
                    std::fs::write(&out_file, rendered)
                        .with_context(|| format!("Failed writing {}", out_file.display()))?;

                    println!("  [Elixir] Generated: {}", out_file.display());
                    count += 1;
                }
                other => anyhow::bail!("Unsupported language target: '{}'. Supported: rust, csharp, typescript, elixir", other),
            }
        }

        if lang.eq_ignore_ascii_case("rust") && !rust_modules.is_empty() {
            let mut mod_ctx = TeraContext::new();
            mod_ctx.insert("modules", &rust_modules);
            let mod_rendered = self.tera.render("rust/mod.tera", &mod_ctx)?;
            let mod_path = output_dir.join("mod.rs");
            std::fs::write(&mod_path, mod_rendered)?;
            println!("  [Rust] Generated module root: {}", mod_path.display());
        }

        if lang.eq_ignore_ascii_case("csharp")
            || lang.eq_ignore_ascii_case("cs")
            || lang.eq_ignore_ascii_case("dotnet")
        {
            let icloud_event_content = "// <auto-generated>\n// Code generated by clrinf-codegen. DO NOT EDIT.\n// </auto-generated>\n\nnamespace ClrInf.Contracts;\n\npublic interface ICloudEvent\n{\n    string EventType { get; }\n}\n";
            let icloud_event_path = output_dir.join("ICloudEvent.cs");
            std::fs::write(&icloud_event_path, icloud_event_content)?;
            println!(
                "  [C#] Generated interface: {}",
                icloud_event_path.display()
            );
        }

        if (lang.eq_ignore_ascii_case("typescript") || lang.eq_ignore_ascii_case("ts"))
            && !ts_modules.is_empty()
        {
            let mut index_content =
                String::from("// Code generated by clrinf-codegen. DO NOT EDIT.\n\n");
            for m in &ts_modules {
                index_content.push_str(&format!("export * from './{}';\n", m));
            }
            let index_path = output_dir.join("index.ts");
            std::fs::write(&index_path, index_content)?;
            println!("  [TS] Generated index root: {}", index_path.display());
        }

        Ok(count)
    }

    /// Generates full end-to-end CQRS Vertical Slice for Rust, C#, TypeScript & Cedar
    pub fn generate_slice(
        &self,
        schema_input: &Path,
        lang: &str,
        output_dir: &Path,
    ) -> Result<usize> {
        ensure!(
            matches!(
                lang.to_lowercase().as_str(),
                "all" | "csharp" | "cs" | "rust" | "rs"
            ),
            "Unsupported slice language: {lang}"
        );
        check_schemas(schema_input)?;
        ensure_safe_output(output_dir)?;
        let schema_files = if schema_input.is_dir() {
            self.find_schema_files(schema_input)?
        } else {
            vec![schema_input.to_path_buf()]
        };

        if schema_files.is_empty() {
            anyhow::bail!(
                "Belirtilen konumda geçerli JSON schema bulunamadı: {}",
                schema_input.display()
            );
        }

        let gen_csharp = lang.eq_ignore_ascii_case("all")
            || lang.eq_ignore_ascii_case("csharp")
            || lang.eq_ignore_ascii_case("cs");
        let gen_rust = lang.eq_ignore_ascii_case("all")
            || lang.eq_ignore_ascii_case("rust")
            || lang.eq_ignore_ascii_case("rs");

        let mut total_files = 0;
        let mut rust_module_groups: BTreeMap<PathBuf, Vec<String>> = BTreeMap::new();

        for file in &schema_files {
            let schema = parse_schema_file(file)?;
            let context = self.create_context(&schema);
            let domain_pascal = schema.domain.to_upper_camel_case();
            let domain_snake = schema.domain.to_snake_case();

            println!(
                "\n🧩 CQRS Dikey Dilimi Üretiliyor: \x1b[1;36m{}\x1b[0m (Etki Alanı: {})",
                schema.title, domain_pascal
            );

            // 1. C# .NET Vertical Slice
            if gen_csharp {
                let cs_contracts_dir = output_dir.join("csharp").join("Contracts");
                let cs_handlers_dir = output_dir.join("csharp").join("Handlers");
                let cs_controllers_dir = output_dir.join("csharp").join("Controllers");
                let cs_domain_dir = output_dir.join("csharp").join("Domain");

                std::fs::create_dir_all(&cs_contracts_dir)?;
                std::fs::create_dir_all(&cs_handlers_dir)?;
                std::fs::create_dir_all(&cs_controllers_dir)?;
                std::fs::create_dir_all(&cs_domain_dir)?;

                // C# Contract
                let rendered_contract = self.tera.render("csharp/event.tera", &context)?;
                let contract_file = cs_contracts_dir.join(format!("{}.cs", schema.title));
                std::fs::write(&contract_file, rendered_contract)?;
                println!("  ├── [C# Contract]      {}", contract_file.display());
                total_files += 1;

                // C# Handler
                let rendered_handler = self.tera.render("csharp/handler.tera", &context)?;
                let handler_file = cs_handlers_dir.join(format!("{}Handler.cs", schema.title));
                std::fs::write(&handler_file, rendered_handler)?;
                println!("  ├── [C# Handler]       {}", handler_file.display());
                total_files += 1;

                // C# Controller
                let rendered_controller = self.tera.render("csharp/controller.tera", &context)?;
                let controller_file =
                    cs_controllers_dir.join(format!("{}Controller.cs", schema.title));
                std::fs::write(&controller_file, rendered_controller)?;
                println!("  ├── [C# Controller]    {}", controller_file.display());
                total_files += 1;

                // C# Obek (Aggregate)
                let rendered_obek = self.tera.render("csharp/obek.tera", &context)?;
                let obek_file = cs_domain_dir.join(format!("{}Obegi.cs", schema.title));
                std::fs::write(&obek_file, rendered_obek)?;
                println!("  ├── [C# Kök Öbek]      {}", obek_file.display());
                total_files += 1;
            }

            // 2. Rust CQRS Slice (Contract, Async Handler Trait, Axum Service Endpoint, Mod)
            if gen_rust {
                let rust_module_dir = output_dir.join("rust").join(&domain_snake);
                std::fs::create_dir_all(&rust_module_dir)?;

                // Rust Model / Struct
                let rendered_model = self.tera.render("rust/event.tera", &context)?;
                let model_file =
                    rust_module_dir.join(format!("{}.rs", schema.file_stem.to_snake_case()));
                std::fs::write(&model_file, rendered_model)?;
                println!("  ├── [Rust Model]       {}", model_file.display());
                total_files += 1;

                // Rust CQRS Handler
                let rendered_handler = self.tera.render("rust/handler.tera", &context)?;
                let handler_file = rust_module_dir
                    .join(format!("{}_handler.rs", schema.file_stem.to_snake_case()));
                std::fs::write(&handler_file, rendered_handler)?;
                println!("  ├── [Rust CQRS Handler]{}", handler_file.display());
                total_files += 1;

                // Rust Service & Axum Handler
                let rendered_servis = self.tera.render("rust/servis.tera", &context)?;
                let servis_file =
                    rust_module_dir.join(format!("{}_servis.rs", schema.file_stem.to_snake_case()));
                std::fs::write(&servis_file, rendered_servis)?;
                println!("  ├── [Rust Axum Servis] {}", servis_file.display());
                total_files += 1;

                rust_module_groups
                    .entry(rust_module_dir)
                    .or_default()
                    .push(schema.file_stem.to_snake_case());
            }

            // 3. Cedar ABAC Policy
            let cedar_dir = output_dir.join("cedar");
            std::fs::create_dir_all(&cedar_dir)?;
            let rendered_cedar = self.tera.render("cedar/policy.tera", &context)?;
            let cedar_file = cedar_dir.join(format!("{}.cedar", schema.file_stem.to_kebab_case()));
            std::fs::write(&cedar_file, rendered_cedar)?;
            println!("  ├── [Cedar ABAC]       {}", cedar_file.display());
            total_files += 1;

            // 4. TypeScript Client & Models
            let ts_models_dir = output_dir.join("typescript").join("models");
            let ts_clients_dir = output_dir.join("typescript").join("clients");
            std::fs::create_dir_all(&ts_models_dir)?;
            std::fs::create_dir_all(&ts_clients_dir)?;

            let rendered_ts_model = self.tera.render("typescript/event.tera", &context)?;
            let ts_model_file =
                ts_models_dir.join(format!("{}.ts", schema.file_stem.to_kebab_case()));
            std::fs::write(&ts_model_file, rendered_ts_model)?;
            println!("  ├── [TS Model]         {}", ts_model_file.display());
            total_files += 1;

            let rendered_ts_client = self.tera.render("typescript/client.tera", &context)?;
            let ts_client_file =
                ts_clients_dir.join(format!("{}-client.ts", schema.file_stem.to_kebab_case()));
            std::fs::write(&ts_client_file, rendered_ts_client)?;
            println!("  └── [TS API Client]    {}", ts_client_file.display());
            total_files += 1;
        }

        for (directory, modules) in rust_module_groups {
            let mut content =
                String::from("pub trait CloudEvent { fn event_type(&self) -> &'static str; }\n");
            for name in modules {
                content.push_str(&format!(
                    "pub mod {name};\npub mod {name}_handler;\npub mod {name}_servis;\n"
                ));
            }
            std::fs::write(directory.join("mod.rs"), content)?;
            total_files += 1;
        }
        Ok(total_files)
    }

    /// Generates event upcasters across Rust, C#, TypeScript
    pub fn generate_upcasters(
        &self,
        schema_dir: &Path,
        lang: &str,
        output_dir: &Path,
    ) -> Result<usize> {
        let tera = crate::upcaster::templates()?;
        crate::upcaster::generate(&tera, schema_dir, lang, output_dir)
    }

    /// Generates CloudEvent publisher envelopes and subscriber shells with idempotency enforcement.
    pub fn generate_pubsub(
        &self,
        schema_dir: &Path,
        lang: &str,
        output_dir: &Path,
    ) -> Result<usize> {
        ensure!(
            matches!(
                lang.to_lowercase().as_str(),
                "rust" | "csharp" | "cs" | "dotnet" | "typescript" | "ts"
            ),
            "Unsupported language for pub/sub generation: {lang}. Supported: rust, csharp, typescript"
        );
        check_schemas(schema_dir)?;
        ensure_safe_output(output_dir)?;
        std::fs::create_dir_all(output_dir).with_context(|| {
            format!(
                "Failed to create output directory: {}",
                output_dir.display()
            )
        })?;

        let schema_files = self.find_schema_files(schema_dir)?;
        let mut count = 0;
        let mut rust_modules = Vec::new();
        let mut ts_exports = Vec::new();

        for file in &schema_files {
            let schema = parse_schema_file(file)?;
            // Only generate pub/sub if it has an event type or pub/sub metadata
            if schema.event_type.is_none() && schema.published_by.is_empty() && schema.subscribed_by.is_empty() {
                continue;
            }

            let context = self.create_context(&schema);

            match lang.to_lowercase().as_str() {
                "rust" => {
                    let pub_rendered = self.tera.render("rust/publisher.tera", &context)
                        .with_context(|| format!("Failed rendering Rust publisher for {}", schema.title))?;
                    let pub_name = format!("{}_publisher", schema.file_stem.to_snake_case());
                    let pub_file = output_dir.join(format!("{}.rs", pub_name));
                    std::fs::write(&pub_file, pub_rendered)?;
                    eprintln!("  [Rust Publisher]  {}", pub_file.display());
                    count += 1;
                    rust_modules.push(pub_name);

                    let sub_rendered = self.tera.render("rust/subscriber.tera", &context)
                        .with_context(|| format!("Failed rendering Rust subscriber for {}", schema.title))?;
                    let sub_name = format!("{}_subscriber", schema.file_stem.to_snake_case());
                    let sub_file = output_dir.join(format!("{}.rs", sub_name));
                    std::fs::write(&sub_file, sub_rendered)?;
                    eprintln!("  [Rust Subscriber] {}", sub_file.display());
                    count += 1;
                    rust_modules.push(sub_name);
                }
                "csharp" | "cs" | "dotnet" => {
                    let pub_rendered = self.tera.render("csharp/publisher.tera", &context)
                        .with_context(|| format!("Failed rendering C# publisher for {}", schema.title))?;
                    let pub_file = output_dir.join(format!("{}Publisher.cs", schema.title));
                    std::fs::write(&pub_file, pub_rendered)?;
                    eprintln!("  [C# Publisher]    {}", pub_file.display());
                    count += 1;

                    let sub_rendered = self.tera.render("csharp/subscriber.tera", &context)
                        .with_context(|| format!("Failed rendering C# subscriber for {}", schema.title))?;
                    let sub_file = output_dir.join(format!("{}Subscriber.cs", schema.title));
                    std::fs::write(&sub_file, sub_rendered)?;
                    eprintln!("  [C# Subscriber]   {}", sub_file.display());
                    count += 1;
                }
                "typescript" | "ts" => {
                    let pub_rendered = self.tera.render("typescript/publisher.tera", &context)
                        .with_context(|| format!("Failed rendering TypeScript publisher for {}", schema.title))?;
                    let pub_name = format!("{}-publisher", schema.file_stem.to_kebab_case());
                    let pub_file = output_dir.join(format!("{}.ts", pub_name));
                    std::fs::write(&pub_file, pub_rendered)?;
                    eprintln!("  [TS Publisher]    {}", pub_file.display());
                    count += 1;
                    ts_exports.push(pub_name);

                    let sub_rendered = self.tera.render("typescript/subscriber.tera", &context)
                        .with_context(|| format!("Failed rendering TypeScript subscriber for {}", schema.title))?;
                    let sub_name = format!("{}-subscriber", schema.file_stem.to_kebab_case());
                    let sub_file = output_dir.join(format!("{}.ts", sub_name));
                    std::fs::write(&sub_file, sub_rendered)?;
                    eprintln!("  [TS Subscriber]   {}", sub_file.display());
                    count += 1;
                    ts_exports.push(sub_name);
                }
                _ => unreachable!(),
            }
        }

        if !rust_modules.is_empty() {
            let mut mod_content = String::from("// Code generated by clrinf-codegen. DO NOT EDIT.\n");
            for m in rust_modules {
                mod_content.push_str(&format!("pub mod {m};\n"));
            }
            std::fs::write(output_dir.join("mod.rs"), mod_content)?;
            count += 1;
        }

        if !ts_exports.is_empty() {
            let mut index_content = String::from("// Code generated by clrinf-codegen. DO NOT EDIT.\n");
            for exp in ts_exports {
                index_content.push_str(&format!("export * from \"./{exp}\";\n"));
            }
            std::fs::write(output_dir.join("index.ts"), index_content)?;
            count += 1;
        }

        Ok(count)
    }

    pub fn scaffold_otp(&self, module: &str, app: &str, output_dir: &Path) -> Result<Vec<PathBuf>> {
        ensure_safe_output(output_dir)?;
        let module_pascal = module.to_upper_camel_case();
        let snake_name = module.to_snake_case();
        let app_module = app.to_upper_camel_case();

        let target_dir = output_dir.join(&snake_name);
        std::fs::create_dir_all(&target_dir)
            .with_context(|| format!("Failed to create directory: {}", target_dir.display()))?;

        let mut context = TeraContext::new();
        context.insert("module", &module_pascal);
        context.insert("snake_name", &snake_name);
        context.insert("app", &app.to_snake_case());
        context.insert("app_module", &app_module);

        let mut created = Vec::new();

        // 1. Worker
        let worker_rendered = self.tera.render("elixir/otp_worker.tera", &context)
            .context("Failed rendering elixir/otp_worker.tera")?;
        let worker_path = target_dir.join("worker.ex");
        std::fs::write(&worker_path, worker_rendered)?;
        created.push(worker_path);

        // 2. Supervisor
        let supervisor_rendered = self.tera.render("elixir/otp_supervisor.tera", &context)
            .context("Failed rendering elixir/otp_supervisor.tera")?;
        let supervisor_path = target_dir.join("supervisor.ex");
        std::fs::write(&supervisor_path, supervisor_rendered)?;
        created.push(supervisor_path);

        // 3. Handler
        let handler_rendered = self.tera.render("elixir/otp_handler.tera", &context)
            .context("Failed rendering elixir/otp_handler.tera")?;
        let handler_path = target_dir.join("handler.ex");
        std::fs::write(&handler_path, handler_rendered)?;
        created.push(handler_path);

        Ok(created)
    }

    pub fn scaffold_rule_elixir(
        &self,
        rule_name: &str,
        entity: Option<&str>,
        error_code: Option<&str>,
        app: Option<&str>,
        target_dir: &Path,
    ) -> Result<Vec<PathBuf>> {
        ensure_safe_output(target_dir)?;
        std::fs::create_dir_all(target_dir)
            .with_context(|| format!("Failed to create directory: {}", target_dir.display()))?;

        let rule_pascal = rule_name.to_upper_camel_case();
        let rule_snake = rule_name.to_snake_case();
        let entity_name = entity.unwrap_or("Entity");
        let default_err = format!("{}_VIOLATION", rule_pascal.to_snake_case().to_uppercase());
        let err_code = error_code.unwrap_or(&default_err);
        let app_name = app.unwrap_or("MyApp");
        let app_module = app_name.to_upper_camel_case();

        let mut context = TeraContext::new();
        context.insert("rule_name", &rule_pascal);
        context.insert("entity", entity_name);
        context.insert("error_code", err_code);
        context.insert("app_module", &app_module);

        let mut created = Vec::new();

        let rule_rendered = self.tera.render("elixir/rule.tera", &context)
            .context("Failed rendering elixir/rule.tera")?;
        let rule_path = target_dir.join(format!("{}.ex", rule_snake));
        std::fs::write(&rule_path, rule_rendered)?;
        created.push(rule_path);

        if self.tera.get_template_names().any(|t| t == "elixir/rule_test.tera") {
            let test_rendered = self.tera.render("elixir/rule_test.tera", &context)
                .context("Failed rendering elixir/rule_test.tera")?;
            let test_path = target_dir.join(format!("{}_test.exs", rule_snake));
            std::fs::write(&test_path, test_rendered)?;
            created.push(test_path);
        }

        Ok(created)
    }

    fn find_schema_files(&self, dir: &Path) -> Result<Vec<PathBuf>> {
        discover_files(dir, &["json"])
    }

    fn create_context(&self, schema: &ParsedSchema) -> TeraContext {
        let mut context = TeraContext::new();
        context.insert("title", &schema.title);
        context.insert("description", &schema.description);
        context.insert("domain", &schema.domain.to_upper_camel_case());
        context.insert("wire_domain", &schema.domain);
        context.insert("file_stem", &schema.file_stem);
        context.insert("event_type", &schema.event_type);
        context.insert("command_type", &schema.command_type);
        context.insert("query_type", &schema.query_type);
        context.insert("fields", &schema.fields);
        context.insert("published_by", &schema.published_by);
        context.insert("subscribed_by", &schema.subscribed_by);
        context.insert("version", &schema.version);
        context
    }
}
