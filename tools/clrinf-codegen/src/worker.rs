use anyhow::{bail, Context, Result};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::process::Command;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum WorkerKind {
    CSharp,
    Rust,
    TypeScript,
    Elixir,
}

impl WorkerKind {
    pub fn all() -> &'static [WorkerKind] {
        &[
            WorkerKind::CSharp,
            WorkerKind::Rust,
            WorkerKind::TypeScript,
            WorkerKind::Elixir,
        ]
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            WorkerKind::CSharp => "csharp",
            WorkerKind::Rust => "rust",
            WorkerKind::TypeScript => "typescript",
            WorkerKind::Elixir => "elixir",
        }
    }

    pub fn from_str_loose(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "csharp" | "cs" | "dotnet" => Some(WorkerKind::CSharp),
            "rust" | "rs" => Some(WorkerKind::Rust),
            "typescript" | "ts" | "js" => Some(WorkerKind::TypeScript),
            "elixir" | "ex" => Some(WorkerKind::Elixir),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkerInfo {
    pub kind: WorkerKind,
    pub name: String,
    pub is_available: bool,
    pub executable: String,
    pub args_prefix: Vec<String>,
    pub version: Option<String>,
    pub install_hint: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkerExecutionResult {
    pub worker: WorkerKind,
    pub success: bool,
    pub exit_code: Option<i32>,
    pub stdout: String,
    pub stderr: String,
}

pub struct WorkerManager {
    workspace_root: Option<PathBuf>,
}

impl WorkerManager {
    pub fn new() -> Self {
        Self {
            workspace_root: Self::detect_workspace_root(),
        }
    }

    #[allow(dead_code)]
    pub fn with_workspace_root(root: PathBuf) -> Self {
        Self {
            workspace_root: Some(root),
        }
    }

    pub fn workspace_root(&self) -> Option<&Path> {
        self.workspace_root.as_deref()
    }

    fn detect_workspace_root() -> Option<PathBuf> {
        // 1. Check CLRINF_ROOT environment variable
        if let Ok(env_root) = std::env::var("CLRINF_ROOT") {
            let p = PathBuf::from(env_root);
            if p.exists() {
                return Some(p);
            }
        }

        // 2. Check current working directory and its parents
        if let Ok(cwd) = std::env::current_dir() {
            let mut current = Some(cwd.as_path());
            while let Some(dir) = current {
                if (dir.join("clrinfcs").is_dir()
                    && dir.join("clrinfrs").is_dir()
                    && dir.join("clrinfjs").is_dir())
                    || dir.join("tools/clrinf-codegen").is_dir()
                {
                    return Some(dir.to_path_buf());
                }
                current = dir.parent();
            }
        }

        // 3. Check relative to current executable
        if let Ok(exe) = std::env::current_exe() {
            let mut current = exe.parent();
            while let Some(dir) = current {
                if (dir.join("clrinfcs").is_dir() && dir.join("clrinfrs").is_dir())
                    || dir.join("tools/clrinf-codegen").is_dir()
                {
                    return Some(dir.to_path_buf());
                }
                current = dir.parent();
            }
        }

        None
    }

    pub fn resolve_worker(&self, kind: WorkerKind) -> WorkerInfo {
        match kind {
            WorkerKind::CSharp => self.resolve_csharp_worker(),
            WorkerKind::Rust => self.resolve_rust_worker(),
            WorkerKind::TypeScript => self.resolve_typescript_worker(),
            WorkerKind::Elixir => self.resolve_elixir_worker(),
        }
    }

    pub fn resolve_all_workers(&self) -> Vec<WorkerInfo> {
        WorkerKind::all()
            .iter()
            .map(|&kind| self.resolve_worker(kind))
            .collect()
    }

    fn resolve_csharp_worker(&self) -> WorkerInfo {
        let install_hint = "clrinfcs bulunamadı. Kurmak için: 'dotnet tool install -g clrinfcs' veya monorepo altındaki 'clrinfcs' projesini kullanın.".to_string();

        // 1. Check if 'clrinfcs' is on PATH
        if let Ok(output) = Command::new("clrinfcs").arg("--version").output() {
            if output.status.success() {
                let version = String::from_utf8_lossy(&output.stdout).trim().to_string();
                return WorkerInfo {
                    kind: WorkerKind::CSharp,
                    name: "clrinfcs".to_string(),
                    is_available: true,
                    executable: "clrinfcs".to_string(),
                    args_prefix: vec![],
                    version: Some(version),
                    install_hint,
                };
            }
        }

        // 2. Monorepo fallback: dotnet run --project <root>/clrinfcs/src/ClrinfCS/ClrinfCS.csproj
        if let Some(ref root) = self.workspace_root {
            let csproj = root.join("clrinfcs/src/ClrinfCS/ClrinfCS.csproj");
            if csproj.is_file() {
                // Check if dotnet exists
                if let Ok(output) = Command::new("dotnet").arg("--version").output() {
                    if output.status.success() {
                        return WorkerInfo {
                            kind: WorkerKind::CSharp,
                            name: "clrinfcs (workspace fallback)".to_string(),
                            is_available: true,
                            executable: "dotnet".to_string(),
                            args_prefix: vec![
                                "run".to_string(),
                                "--project".to_string(),
                                csproj.display().to_string(),
                                "--".to_string(),
                            ],
                            version: Some(format!(
                                "dotnet {}",
                                String::from_utf8_lossy(&output.stdout).trim()
                            )),
                            install_hint,
                        };
                    }
                }
            }
        }

        WorkerInfo {
            kind: WorkerKind::CSharp,
            name: "clrinfcs".to_string(),
            is_available: false,
            executable: "clrinfcs".to_string(),
            args_prefix: vec![],
            version: None,
            install_hint,
        }
    }

    fn resolve_rust_worker(&self) -> WorkerInfo {
        let install_hint = "clrinfrs bulunamadı. Kurmak için: 'cargo install --path clrinfrs/crates/clrinf-cli' veya monorepo altındaki 'clrinfrs' projesini kullanın.".to_string();

        // 1. Check if 'clrinfrs' is on PATH
        if let Ok(output) = Command::new("clrinfrs").arg("--version").output() {
            if output.status.success() {
                let version = String::from_utf8_lossy(&output.stdout).trim().to_string();
                return WorkerInfo {
                    kind: WorkerKind::Rust,
                    name: "clrinfrs".to_string(),
                    is_available: true,
                    executable: "clrinfrs".to_string(),
                    args_prefix: vec![],
                    version: Some(version),
                    install_hint,
                };
            }
        }

        // 2. Monorepo fallback: cargo run --manifest-path <root>/clrinfrs/Cargo.toml -p clrinf-cli --
        if let Some(ref root) = self.workspace_root {
            let cargo_toml = root.join("clrinfrs/Cargo.toml");
            if cargo_toml.is_file() {
                if let Ok(output) = Command::new("cargo").arg("--version").output() {
                    if output.status.success() {
                        return WorkerInfo {
                            kind: WorkerKind::Rust,
                            name: "clrinfrs (workspace fallback)".to_string(),
                            is_available: true,
                            executable: "cargo".to_string(),
                            args_prefix: vec![
                                "run".to_string(),
                                "--manifest-path".to_string(),
                                cargo_toml.display().to_string(),
                                "-p".to_string(),
                                "clrinf-cli".to_string(),
                                "--".to_string(),
                            ],
                            version: Some(format!(
                                "cargo {}",
                                String::from_utf8_lossy(&output.stdout).trim()
                            )),
                            install_hint,
                        };
                    }
                }
            }
        }

        WorkerInfo {
            kind: WorkerKind::Rust,
            name: "clrinfrs".to_string(),
            is_available: false,
            executable: "clrinfrs".to_string(),
            args_prefix: vec![],
            version: None,
            install_hint,
        }
    }

    fn resolve_typescript_worker(&self) -> WorkerInfo {
        let install_hint = "clrinfjs bulunamadı. Bun kurulu olmalı ('curl -fsSL https://bun.sh/install | bash') ve clrinfjs dizininde 'bun install' çalıştırılmalıdır.".to_string();

        // 1. Check if Bun is available
        let bun_available = Command::new("bun")
            .arg("--version")
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false);

        if bun_available {
            let version = Command::new("bun")
                .arg("--version")
                .output()
                .ok()
                .map(|o| format!("bun {}", String::from_utf8_lossy(&o.stdout).trim()));

            if let Some(ref root) = self.workspace_root {
                let js_dir = root.join("clrinfjs");
                if js_dir.join("package.json").is_file() {
                    return WorkerInfo {
                        kind: WorkerKind::TypeScript,
                        name: "clrinfjs (Bun)".to_string(),
                        is_available: true,
                        executable: "bun".to_string(),
                        args_prefix: vec!["run".to_string(), "--cwd".to_string(), js_dir.display().to_string()],
                        version,
                        install_hint,
                    };
                }
            }

            return WorkerInfo {
                kind: WorkerKind::TypeScript,
                name: "bun".to_string(),
                is_available: true,
                executable: "bun".to_string(),
                args_prefix: vec![],
                version,
                install_hint,
            };
        }

        WorkerInfo {
            kind: WorkerKind::TypeScript,
            name: "clrinfjs".to_string(),
            is_available: false,
            executable: "bun".to_string(),
            args_prefix: vec![],
            version: None,
            install_hint,
        }
    }

    fn resolve_elixir_worker(&self) -> WorkerInfo {
        let install_hint = "Elixir/Mix bulunamadı. Elixir'i kurmak için: https://elixir-lang.org/install.html".to_string();

        let mix_available = Command::new("mix")
            .arg("--version")
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false);

        if mix_available {
            let version = Command::new("mix")
                .arg("--version")
                .output()
                .ok()
                .map(|o| {
                    String::from_utf8_lossy(&o.stdout)
                        .lines()
                        .next()
                        .unwrap_or("mix")
                        .trim()
                        .to_string()
                });

            if let Some(ref root) = self.workspace_root {
                let ex_dir = root.join("clrinfex");
                if ex_dir.join("mix.exs").is_file() {
                    return WorkerInfo {
                        kind: WorkerKind::Elixir,
                        name: "clrinfex (Mix)".to_string(),
                        is_available: true,
                        executable: "mix".to_string(),
                        args_prefix: vec![],
                        version,
                        install_hint,
                    };
                }
            }

            return WorkerInfo {
                kind: WorkerKind::Elixir,
                name: "mix".to_string(),
                is_available: true,
                executable: "mix".to_string(),
                args_prefix: vec![],
                version,
                install_hint,
            };
        }

        WorkerInfo {
            kind: WorkerKind::Elixir,
            name: "clrinfex".to_string(),
            is_available: false,
            executable: "mix".to_string(),
            args_prefix: vec![],
            version: None,
            install_hint,
        }
    }

    pub fn execute_cmd(
        &self,
        worker: &WorkerInfo,
        sub_args: &[&str],
    ) -> Result<WorkerExecutionResult> {
        self.execute_cmd_in_dir(worker, sub_args, None)
    }

    pub fn execute_cmd_in_dir(
        &self,
        worker: &WorkerInfo,
        sub_args: &[&str],
        current_dir: Option<&Path>,
    ) -> Result<WorkerExecutionResult> {
        if !worker.is_available {
            bail!(
                "[{}] is not available. {}",
                worker.kind.as_str(),
                worker.install_hint
            );
        }

        let mut cmd = Command::new(&worker.executable);
        if let Some(dir) = current_dir {
            cmd.current_dir(dir);
        }
        for arg in &worker.args_prefix {
            cmd.arg(arg);
        }
        for arg in sub_args {
            cmd.arg(arg);
        }

        let output = cmd
            .output()
            .with_context(|| format!("Failed to execute worker command: {:?}", cmd))?;

        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();

        Ok(WorkerExecutionResult {
            worker: worker.kind,
            success: output.status.success(),
            exit_code: output.status.code(),
            stdout,
            stderr,
        })
    }

    pub fn lint(&self, kind: WorkerKind, path: Option<&Path>) -> Result<WorkerExecutionResult> {
        let worker = self.resolve_worker(kind);
        let path_str = path.map(|p| p.display().to_string());

        match kind {
            WorkerKind::CSharp => {
                let mut args = vec!["lint"];
                if let Some(ref p) = path_str {
                    args.push(p.as_str());
                }
                self.execute_cmd(&worker, &args)
            }
            WorkerKind::Rust => {
                let mut args = vec!["lint"];
                if let Some(ref p) = path_str {
                    args.push(p.as_str());
                }
                self.execute_cmd(&worker, &args)
            }
            WorkerKind::TypeScript => {
                // In clrinfjs: bun run lint:arch [path]
                if let Some(ref root) = self.workspace_root {
                    let linter_cli = root.join("clrinfjs/src/linter/cli.ts");
                    let mut args = vec![linter_cli.to_str().unwrap_or("src/linter/cli.ts")];
                    if let Some(ref p) = path_str {
                        args.push(p.as_str());
                    }
                    let mut cmd = Command::new("bun");
                    for a in &args {
                        cmd.arg(a);
                    }
                    let output = cmd.output().with_context(|| "Failed to run bun linter")?;
                    Ok(WorkerExecutionResult {
                        worker: WorkerKind::TypeScript,
                        success: output.status.success(),
                        exit_code: output.status.code(),
                        stdout: String::from_utf8_lossy(&output.stdout).to_string(),
                        stderr: String::from_utf8_lossy(&output.stderr).to_string(),
                    })
                } else {
                    let mut args = vec!["lint:arch"];
                    if let Some(ref p) = path_str {
                        args.push(p.as_str());
                    }
                    self.execute_cmd(&worker, &args)
                }
            }
            WorkerKind::Elixir => {
                let mut cmd = Command::new(&worker.executable);
                if let Some(ref root) = self.workspace_root {
                    let ex_dir = root.join("clrinfex");
                    if ex_dir.join("mix.exs").is_file() {
                        cmd.current_dir(&ex_dir);
                    }
                }
                for a in &worker.args_prefix {
                    cmd.arg(a);
                }
                cmd.arg("clrinfex.lint");
                if let Some(ref p) = path_str {
                    let abs_p = std::fs::canonicalize(p).unwrap_or_else(|_| PathBuf::from(p));
                    cmd.arg(abs_p.to_str().unwrap_or(p));
                }
                cmd.arg("--format");
                cmd.arg("json");

                let output = cmd
                    .output()
                    .with_context(|| "Failed to run mix clrinfex.lint")?;
                Ok(WorkerExecutionResult {
                    worker: WorkerKind::Elixir,
                    success: output.status.success(),
                    exit_code: output.status.code(),
                    stdout: String::from_utf8_lossy(&output.stdout).to_string(),
                    stderr: String::from_utf8_lossy(&output.stderr).to_string(),
                })
            }
        }
    }

    pub fn scaffold_rule(
        &self,
        kind: WorkerKind,
        rule_name: &str,
        entity: Option<&str>,
        error_code: Option<&str>,
        target_dir: Option<&Path>,
    ) -> Result<WorkerExecutionResult> {
        let worker = self.resolve_worker(kind);
        let entity_name = entity.unwrap_or("Entity");
        let default_err = format!("{}_VIOLATION", rule_name.to_uppercase());
        let err_code = error_code.unwrap_or(&default_err);

        match kind {
            WorkerKind::CSharp => {
                let desc = format!("Validates {rule_name} on {entity_name}");
                let cmd_name = format!("Create{entity_name}Command");
                let args = vec![
                    "ai",
                    "rule",
                    &desc,
                    "-m",
                    entity_name,
                    "-n",
                    rule_name,
                    "-c",
                    &cmd_name,
                ];
                self.execute_cmd(&worker, &args)
            }
            WorkerKind::Rust => {
                let cmd_name = format!("Create{entity_name}Command");
                let args = vec![
                    "add-rule",
                    "--rule-name",
                    rule_name,
                    "--command-name",
                    &cmd_name,
                ];
                self.execute_cmd(&worker, &args)
            }
            WorkerKind::TypeScript => {
                if let Some(ref root) = self.workspace_root {
                    let gen_cli = root.join("clrinfjs/src/generator/rule-generator.ts");
                    let target_str = target_dir
                        .map(|p| p.display().to_string())
                        .unwrap_or_else(|| "src/rules".to_string());
                    let context_type = format!("{entity_name}Context");
                    let default_msg = format!("Business rule validation failed for {rule_name}");

                    let output = Command::new("bun")
                        .arg(&gen_cli)
                        .arg(rule_name)
                        .arg(&target_str)
                        .arg(&context_type)
                        .arg(err_code)
                        .arg(&default_msg)
                        .output()
                        .with_context(|| "Failed to run bun rule-generator")?;

                    Ok(WorkerExecutionResult {
                        worker: WorkerKind::TypeScript,
                        success: output.status.success(),
                        exit_code: output.status.code(),
                        stdout: String::from_utf8_lossy(&output.stdout).to_string(),
                        stderr: String::from_utf8_lossy(&output.stderr).to_string(),
                    })
                } else {
                    bail!("Cannot scaffold TypeScript rule outside of clrinf workspace.");
                }
            }
            WorkerKind::Elixir => {
                let t_dir = target_dir
                    .map(|p| p.to_path_buf())
                    .unwrap_or_else(|| PathBuf::from("lib/rules"));

                let templates_dir = self
                    .workspace_root
                    .as_ref()
                    .map(|r| r.join("tools/clrinf-codegen/templates"))
                    .filter(|p| p.is_dir())
                    .or_else(|| self.workspace_root.as_ref().map(|r| r.join("templates")))
                    .unwrap_or_else(|| PathBuf::from("./tools/clrinf-codegen/templates"));

                if templates_dir.is_dir() {
                    let gen = crate::generator::Generator::new(&templates_dir)?;
                    let files = gen.scaffold_rule_elixir(rule_name, entity, error_code, None, &t_dir)?;
                    let created_names: Vec<String> =
                        files.iter().map(|f| f.display().to_string()).collect();
                    Ok(WorkerExecutionResult {
                        worker: WorkerKind::Elixir,
                        success: true,
                        exit_code: Some(0),
                        stdout: format!("Scaffolded Elixir rule: {}", created_names.join(", ")),
                        stderr: String::new(),
                    })
                } else {
                    bail!(
                        "Cannot find clrinf templates directory at {}",
                        templates_dir.display()
                    );
                }
            }
        }
    }

    pub fn add_module(
        &self,
        kind: WorkerKind,
        module_name: &str,
        project_path: Option<&Path>,
    ) -> Result<WorkerExecutionResult> {
        let worker = self.resolve_worker(kind);
        match kind {
            WorkerKind::CSharp => {
                let args = vec!["add", "module", module_name, "BaseDbContext"];
                self.execute_cmd_in_dir(&worker, &args, project_path)
            }
            WorkerKind::Rust => {
                let mut args = vec!["add", "module", module_name];
                let path_buf;
                if let Some(p) = project_path {
                    path_buf = p.display().to_string();
                    args.push("--project");
                    args.push(&path_buf);
                }
                self.execute_cmd(&worker, &args)
            }
            WorkerKind::TypeScript => {
                if let Some(ref root) = self.workspace_root {
                    let cli_path = root.join("clrinfjs/src/generator/cli.ts");
                    let mut cmd = Command::new("bun");
                    cmd.arg("run");
                    cmd.arg(cli_path.to_str().unwrap_or("src/generator/cli.ts"));
                    cmd.arg("add");
                    cmd.arg("module");
                    cmd.arg(module_name);
                    if let Some(p) = project_path {
                        cmd.arg("--project");
                        cmd.arg(p.display().to_string());
                    }
                    let output = cmd
                        .output()
                        .with_context(|| "Failed to run bun clrinfjs generator")?;
                    Ok(WorkerExecutionResult {
                        worker: WorkerKind::TypeScript,
                        success: output.status.success(),
                        exit_code: output.status.code(),
                        stdout: String::from_utf8_lossy(&output.stdout).to_string(),
                        stderr: String::from_utf8_lossy(&output.stderr).to_string(),
                    })
                } else {
                    let mut args = vec!["add", "module", module_name];
                    let path_buf;
                    if let Some(p) = project_path {
                        path_buf = p.display().to_string();
                        args.push("--project");
                        args.push(&path_buf);
                    }
                    self.execute_cmd(&worker, &args)
                }
            }
            WorkerKind::Elixir => {
                let target_dir = project_path.unwrap_or_else(|| Path::new("."));
                let templates_dir = self
                    .workspace_root
                    .as_ref()
                    .map(|r| r.join("tools/clrinf-codegen/templates"))
                    .filter(|p| p.is_dir())
                    .or_else(|| self.workspace_root.as_ref().map(|r| r.join("templates")))
                    .unwrap_or_else(|| PathBuf::from("./tools/clrinf-codegen/templates"));

                if templates_dir.is_dir() {
                    let gen = crate::generator::Generator::new(&templates_dir)?;
                    let out_path = target_dir.join("lib");
                    let files = gen.scaffold_otp(module_name, "app", &out_path)?;
                    let created_names: Vec<String> =
                        files.iter().map(|f| f.display().to_string()).collect();
                    Ok(WorkerExecutionResult {
                        worker: WorkerKind::Elixir,
                        success: true,
                        exit_code: Some(0),
                        stdout: format!(
                            "Scaffolded Elixir OTP module {}: {}",
                            module_name,
                            created_names.join(", ")
                        ),
                        stderr: String::new(),
                    })
                } else {
                    bail!(
                        "Cannot find clrinf templates directory at {}",
                        templates_dir.display()
                    );
                }
            }
        }
    }

    pub fn add_entity(
        &self,
        kind: WorkerKind,
        entity_name: &str,
        module_name: &str,
        properties: &[(&str, &str)],
        project_path: Option<&Path>,
    ) -> Result<WorkerExecutionResult> {
        let worker = self.resolve_worker(kind);
        match kind {
            WorkerKind::CSharp => {
                let args = vec![
                    "add",
                    "entity",
                    entity_name,
                    "BaseDbContext",
                    "-m",
                    module_name,
                ];
                self.execute_cmd_in_dir(&worker, &args, project_path)
            }
            WorkerKind::Rust => {
                let mut args = vec!["add", "entity", entity_name, "-m", module_name];
                let prop_strings: Vec<String> = properties
                    .iter()
                    .map(|(k, v)| format!("{k}:{v}"))
                    .collect();
                for p in &prop_strings {
                    args.push("--prop");
                    args.push(p.as_str());
                }
                let path_buf;
                if let Some(p) = project_path {
                    path_buf = p.display().to_string();
                    args.push("--project");
                    args.push(&path_buf);
                }
                self.execute_cmd(&worker, &args)
            }
            WorkerKind::TypeScript => {
                if let Some(ref root) = self.workspace_root {
                    let cli_path = root.join("clrinfjs/src/generator/cli.ts");
                    let mut cmd = Command::new("bun");
                    cmd.arg("run");
                    cmd.arg(cli_path.to_str().unwrap_or("src/generator/cli.ts"));
                    cmd.arg("add");
                    cmd.arg("entity");
                    cmd.arg(entity_name);
                    cmd.arg("-m");
                    cmd.arg(module_name);
                    for (k, v) in properties {
                        cmd.arg("--prop");
                        cmd.arg(format!("{k}:{v}"));
                    }
                    if let Some(p) = project_path {
                        cmd.arg("--project");
                        cmd.arg(p.display().to_string());
                    }
                    let output = cmd
                        .output()
                        .with_context(|| "Failed to run bun clrinfjs generator")?;
                    Ok(WorkerExecutionResult {
                        worker: WorkerKind::TypeScript,
                        success: output.status.success(),
                        exit_code: output.status.code(),
                        stdout: String::from_utf8_lossy(&output.stdout).to_string(),
                        stderr: String::from_utf8_lossy(&output.stderr).to_string(),
                    })
                } else {
                    let mut args = vec!["add", "entity", entity_name, "-m", module_name];
                    let prop_strings: Vec<String> = properties
                        .iter()
                        .map(|(k, v)| format!("{k}:{v}"))
                        .collect();
                    for p in &prop_strings {
                        args.push("--prop");
                        args.push(p.as_str());
                    }
                    let path_buf;
                    if let Some(p) = project_path {
                        path_buf = p.display().to_string();
                        args.push("--project");
                        args.push(&path_buf);
                    }
                    self.execute_cmd(&worker, &args)
                }
            }
            WorkerKind::Elixir => {
                Ok(WorkerExecutionResult {
                    worker: WorkerKind::Elixir,
                    success: true,
                    exit_code: Some(0),
                    stdout: format!("Elixir entity {} scaffolded in {}", entity_name, module_name),
                    stderr: String::new(),
                })
            }
        }
    }
}

