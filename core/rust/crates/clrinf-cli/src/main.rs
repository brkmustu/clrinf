use clap::{Args, Parser, Subcommand};
use colored::Colorize;
use std::path::PathBuf;

mod ai;
mod generator;
mod lint;
mod mcp;

use ai::AiRuleGenerator;
use generator::{ArchStyle, Generator};
use lint::Linter;
use mcp::McpServer;

#[derive(Parser)]
#[command(name = "clrinfrs", bin_name = "clrinfrs", author, version = "0.1.0")]
#[command(about = "Rust için sözleşme öncelikli mimari scaffold, kural motoru, Syn AST linter ve MCP sunucusu")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Yeni bir Rust projesi oluşturur (flat, layered veya clean-cqrs mimarisinde)
    New(NewArgs),

    /// Projeye yeni bir bileşen veya kural ekler
    #[command(subcommand)]
    Add(AddCommands),

    /// Syn AST ile mimari kuralları ve katman sınırlarını deterministik olarak denetler
    Lint(LintArgs),

    /// IDE ve AI ajanları için Model Context Protocol (MCP) JSON-RPC sunucusunu başlatır
    Mcp,

    /// Doğal dil ile kural tanımından izole kural ve test dosyaları üretir
    Ai(AiArgs),
}

#[derive(Args)]
struct NewArgs {
    /// Proje adı
    name: String,

    /// Mimari profili: flat, layered veya clean-cqrs (varsayılan: clean-cqrs)
    #[arg(long, default_value = "clean-cqrs")]
    arch: String,

    /// Host tipi: api, web veya console (varsayılan: api)
    #[arg(long, default_value = "api")]
    host: String,

    /// Projenin oluşturulacağı dizin (varsayılan: mevcut dizin)
    #[arg(long, default_value = ".")]
    path: PathBuf,
}

#[derive(Subcommand)]
enum AddCommands {
    /// Yeni bir izole iş kuralı (BusinessRule) ekler
    Rule {
        /// Kural adı (örn. MaxDiscountRule)
        name: String,

        /// Kuralın bağlı olduğu komut (örn. CreateProductCommand)
        #[arg(short, long)]
        command: String,

        /// Modül veya Entity adı (varsayılan: Products)
        #[arg(short, long, default_value = "Products")]
        module: String,

        /// Proje dizini (varsayılan: .)
        #[arg(short, long, default_value = ".")]
        project: PathBuf,
    },

    /// Yeni bir modül oluşturur (domain, handlers, rules ve Cedar policy ile)
    Module {
        /// Modül adı (örn. Orders)
        name: String,

        /// Proje dizini (varsayılan: .)
        #[arg(short, long, default_value = ".")]
        project: PathBuf,
    },

    /// Modüle yeni bir CRUD Entity ekler (model, repository ve multi-tenant claims ile)
    Entity {
        /// Entity adı (örn. OrderItem)
        name: String,

        /// Bağlı olduğu modül adı
        #[arg(short, long)]
        module: String,

        /// Özellikler (örn. --prop name:string --prop price:f64)
        #[arg(long = "prop")]
        property: Vec<String>,

        /// Proje dizini (varsayılan: .)
        #[arg(long, default_value = ".")]
        project: PathBuf,
    },
}

#[derive(Args)]
struct LintArgs {
    /// Denetlenecek proje dizini (varsayılan: .)
    #[arg(default_value = ".")]
    path: PathBuf,

    /// Sonuçları JSON formatında yazdırır
    #[arg(long)]
    json: bool,

    /// Uyarıları da hata sayar
    #[arg(long)]
    strict: bool,
}

#[derive(Args)]
struct AiArgs {
    #[command(subcommand)]
    command: AiCommands,
}

#[derive(Subcommand)]
enum AiCommands {
    /// Doğal dil kural tanımından izole struct ve test dosyası üretir
    Rule {
        /// Kural açıklaması (örn. 'Fiyat sıfırdan küçük olamaz')
        description: String,

        /// Modül adı
        #[arg(short, long)]
        module: String,

        /// Kural adı (opsiyonel)
        #[arg(short, long)]
        name: Option<String>,

        /// Komut adı (opsiyonel)
        #[arg(short, long)]
        command: Option<String>,

        /// Proje dizini
        #[arg(short, long, default_value = ".")]
        project: PathBuf,
    },
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Check if invoked via cargo: 'cargo clrinf ...' passes 'clrinf' as first argument
    let args: Vec<String> = std::env::args().collect();

    let cli = if args.len() > 1 && args[1] == "clrinf" {
        // Strip cargo custom command prefix
        let mut stripped = vec![args[0].clone()];
        stripped.extend_from_slice(&args[2..]);
        Cli::parse_from(stripped)
    } else {
        Cli::parse()
    };

    match cli.command {
        Commands::New(args) => {
            let arch = ArchStyle::parse_str(&args.arch);
            println!("{}", "=== clrinfrs Proje Üretim Motoru ===".blue().bold());
            println!("{}: {}", "Proje Adı".dimmed(), args.name.cyan());
            println!("{}: {}", "Mimari Profil".dimmed(), args.arch.yellow());
            println!("{}: {}", "Host Tipi".dimmed(), args.host.green());

            Generator::new_project(&args.name, &args.path, arch, &args.host)?;
            println!("{}", "✔ Proje başarıyla oluşturuldu!".green().bold());
        }

        Commands::Add(AddCommands::Rule {
            name,
            command,
            module,
            project,
        }) => {
            println!("{}", "=== clrinfrs İzole Kural Ekleme ===".blue().bold());
            let path = Generator::add_rule(&project, &module, &name, &command)?;
            println!("{} {}", "✔ Kural başarıyla eklendi:".green().bold(), path.display());
            println!("{}", "Not: Mevcut hiçbir dosyaya dokunulmadı (Sıfır regresyon).".dimmed());
        }

        Commands::Add(AddCommands::Module { name, project }) => {
            println!("{}", "=== clrinfrs Modül Oluşturma ===".blue().bold());
            let files = Generator::add_module(&project, &name)?;
            println!("{} '{}'", "✔ Modül başarıyla oluşturuldu:".green().bold(), name);
            for f in files {
                println!("  {} {}", "•".cyan(), f);
            }
        }

        Commands::Add(AddCommands::Entity {
            name,
            module,
            property,
            project,
        }) => {
            println!("{}", "=== clrinfrs CRUD Entity Ekleme ===".blue().bold());
            let parsed_props: Vec<(String, String)> = property
                .iter()
                .map(|p| {
                    let parts: Vec<&str> = p.splitn(2, ':').collect();
                    if parts.len() == 2 {
                        (parts[0].to_string(), parts[1].to_string())
                    } else {
                        (parts[0].to_string(), "String".to_string())
                    }
                })
                .collect();

            let files = Generator::add_entity(&project, &module, &name, &parsed_props)?;
            println!("{} '{}' (Modül: {})", "✔ Entity başarıyla eklendi:".green().bold(), name, module);
            for f in files {
                println!("  {} {}", "•".cyan(), f);
            }
        }

        Commands::Lint(args) => {
            let violations = Linter::lint_directory(&args.path);

            if args.json {
                println!("{}", serde_json::to_string_pretty(&violations)?);
                let has_err = if args.strict {
                    !violations.is_empty()
                } else {
                    violations.iter().any(|v| v.severity == "Error")
                };
                if has_err {
                    std::process::exit(1);
                }
                return Ok(());
            }

            println!("{}", "=== clrinfrs Syn AST Mimari Linter ===".blue().bold());
            println!("{}: {}\n", "Hedef Dizin".dimmed(), args.path.display());

            if violations.is_empty() {
                println!("{}", "✔ Tebrikler! Hiçbir mimari veya kural ihlali bulunamadı.".green().bold());
            } else {
                for v in &violations {
                    let color_sev = match v.severity.as_str() {
                        "Error" => v.severity.red().bold(),
                        _ => v.severity.yellow(),
                    };
                    println!(
                        "[{}] {}: {} ({}:{})",
                        v.rule_id.cyan(),
                        color_sev,
                        v.message,
                        v.file_path.dimmed(),
                        v.line_number
                    );
                }
                println!("\nToplam {} ihlal bulundu.", violations.len());
                let has_err = if args.strict {
                    !violations.is_empty()
                } else {
                    violations.iter().any(|v| v.severity == "Error")
                };
                if has_err {
                    std::process::exit(1);
                }
            }
        }

        Commands::Mcp => {
            McpServer::run().await?;
        }

        Commands::Ai(AiArgs {
            command: AiCommands::Rule {
                description,
                module,
                name,
                command,
                project,
            },
        }) => {
            let rule_name = name.unwrap_or_else(|| format!("{module}ValidationRule"));
            let cmd_name = command.unwrap_or_else(|| format!("Create{module}Command"));

            println!("{}", "=== clrinfrs AI Kural ve Test Sentezi ===".blue().bold());
            let res = AiRuleGenerator::generate_rule(
                &project,
                &module,
                &rule_name,
                &cmd_name,
                &description,
            )?;

            println!("{}", "✔ İzole kural ve test başarıyla üretildi:".green().bold());
            println!("  Kural: {}", res.rule_file.display());
            println!("  Test:  {}", res.test_file.display());
            println!("{}", "Not: Mevcut hiçbir dosyada değişiklik yapılmadı.".dimmed());
        }
    }

    Ok(())
}
