use crate::manifest::{ArchStyle, Dispatcher, HostType};
use anyhow::{Context, Result};
use heck::ToUpperCamelCase;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

#[allow(dead_code)]
pub struct CreateProjectOptions<'a> {
    pub name: &'a str,
    pub target_path: &'a Path,
    pub arch: ArchStyle,
    pub host_type: HostType,
    pub dispatcher: Dispatcher,
    pub paradigm: &'a str,
    pub profile: &'a str,
    pub db_context: Option<&'a str>,
    pub skip_dotnet_exec: bool,
}

#[allow(dead_code)]
pub struct CreateProjectResult {
    pub project_dir: PathBuf,
    pub files_created: Vec<PathBuf>,
}

/// Creates a new C# project according to the chosen architecture, paradigm, and dispatcher.
#[allow(dead_code)]
pub fn create_project(opts: &CreateProjectOptions) -> Result<CreateProjectResult> {
    let project_name = opts.name.to_upper_camel_case();
    let project_dir = opts.target_path.join(&project_name);
    fs::create_dir_all(&project_dir)
        .with_context(|| format!("Failed to create project directory: {}", project_dir.display()))?;

    let mut files_created = Vec::new();

    // 1. Solution-level Directory.Build.props
    let props_path = project_dir.join("Directory.Build.props");
    let props_content = r#"<Project>
  <PropertyGroup>
    <TargetFramework>net9.0</TargetFramework>
    <Nullable>enable</Nullable>
    <ImplicitUsings>enable</ImplicitUsings>
  </PropertyGroup>
</Project>
"#;
    fs::write(&props_path, props_content)?;
    files_created.push(props_path);

    // 2. Generate clrinf.toml
    let clrinf_toml_path = project_dir.join("clrinf.toml");
    let arch_str = match opts.arch {
        ArchStyle::Flat => "flat",
        ArchStyle::Layered => "layered",
        ArchStyle::CleanCqrs => "clean-cqrs",
    };
    let db_ctx = opts.db_context.unwrap_or("BaseDbContext");
    let toml_content = format!(
        r#"[project]
name = "{project_name}"
lang = "csharp"
arch = "{arch_str}"
paradigm = "{}"
dispatcher = "{}"
db_context = "{db_ctx}"
profile = "{}"
"#,
        opts.paradigm,
        opts.dispatcher.as_str(),
        opts.profile
    );
    fs::write(&clrinf_toml_path, &toml_content)
        .with_context(|| format!("Failed to write {}", clrinf_toml_path.display()))?;
    files_created.push(clrinf_toml_path);

    // 3. Scaffolding structure
    match opts.arch {
        ArchStyle::Flat => {
            scaffold_flat(&project_dir, &project_name, opts, &mut files_created)?;
        }
        ArchStyle::Layered => {
            scaffold_layered(&project_dir, &project_name, opts, &mut files_created)?;
        }
        ArchStyle::CleanCqrs => {
            scaffold_clean_cqrs(&project_dir, &project_name, opts, &mut files_created)?;
        }
    }

    Ok(CreateProjectResult {
        project_dir,
        files_created,
    })
}

#[allow(dead_code)]
fn scaffold_clean_cqrs(
    project_dir: &Path,
    project_name: &str,
    opts: &CreateProjectOptions,
    files: &mut Vec<PathBuf>,
) -> Result<()> {
    let src_dir = project_dir.join("src");
    let api_dir = src_dir.join("Api");
    let app_dir = src_dir.join("Application");
    let domain_dir = src_dir.join("Domain");
    let persistence_dir = src_dir.join("Persistence");
    let infra_dir = src_dir.join("Infrastructure");

    fs::create_dir_all(&api_dir)?;
    fs::create_dir_all(app_dir.join("Modules"))?;
    fs::create_dir_all(&domain_dir)?;
    fs::create_dir_all(persistence_dir.join("Contexts"))?;
    fs::create_dir_all(&infra_dir)?;

    // If dotnet CLI available and not skipped, run dotnet new commands
    if !opts.skip_dotnet_exec && Command::new("dotnet").arg("--version").output().is_ok() {
        let _ = Command::new("dotnet")
            .args(["new", "sln", "-n", project_name])
            .current_dir(project_dir)
            .output();

        let _ = Command::new("dotnet")
            .args(["new", "webapi", "-n", &format!("{}.Api", project_name), "-o", "src/Api"])
            .current_dir(project_dir)
            .output();

        let _ = Command::new("dotnet")
            .args(["new", "classlib", "-n", &format!("{}.Application", project_name), "-o", "src/Application"])
            .current_dir(project_dir)
            .output();

        let _ = Command::new("dotnet")
            .args(["new", "classlib", "-n", &format!("{}.Domain", project_name), "-o", "src/Domain"])
            .current_dir(project_dir)
            .output();

        let _ = Command::new("dotnet")
            .args(["new", "classlib", "-n", &format!("{}.Persistence", project_name), "-o", "src/Persistence"])
            .current_dir(project_dir)
            .output();

        let _ = Command::new("dotnet")
            .args(["new", "classlib", "-n", &format!("{}.Infrastructure", project_name), "-o", "src/Infrastructure"])
            .current_dir(project_dir)
            .output();

        // Add to sln
        let _ = Command::new("dotnet")
            .args(["sln", "add", "src/Api", "src/Application", "src/Domain", "src/Persistence", "src/Infrastructure"])
            .current_dir(project_dir)
            .output();

        // Add project references
        let _ = Command::new("dotnet")
            .args(["add", "src/Application", "reference", "src/Domain"])
            .current_dir(project_dir)
            .output();
        let _ = Command::new("dotnet")
            .args(["add", "src/Persistence", "reference", "src/Application"])
            .current_dir(project_dir)
            .output();
        let _ = Command::new("dotnet")
            .args(["add", "src/Infrastructure", "reference", "src/Application"])
            .current_dir(project_dir)
            .output();
        let _ = Command::new("dotnet")
            .args(["add", "src/Api", "reference", "src/Application", "src/Persistence", "src/Infrastructure"])
            .current_dir(project_dir)
            .output();
    } else {
        // Fallback: create .csproj files directly with cross-project references
        write_csproj(
            &api_dir.join(format!("{}.Api.csproj", project_name)),
            true,
            &["Microsoft.EntityFrameworkCore", "Microsoft.EntityFrameworkCore.Sqlite"],
            &["../Application/Application.csproj", "../Persistence/Persistence.csproj", "../Infrastructure/Infrastructure.csproj"],
        )?;
        write_csproj(
            &app_dir.join(format!("{}.Application.csproj", project_name)),
            false,
            &[],
            &["../Domain/Domain.csproj"],
        )?;
        write_csproj(
            &domain_dir.join(format!("{}.Domain.csproj", project_name)),
            false,
            &[],
            &[],
        )?;
        write_csproj(
            &persistence_dir.join(format!("{}.Persistence.csproj", project_name)),
            false,
            &["Microsoft.EntityFrameworkCore", "Microsoft.EntityFrameworkCore.Sqlite"],
            &["../Application/Application.csproj"],
        )?;
        write_csproj(
            &infra_dir.join(format!("{}.Infrastructure.csproj", project_name)),
            false,
            &[],
            &["../Application/Application.csproj"],
        )?;
    }

    // FP primitives if paradigm is fp
    if opts.paradigm.eq_ignore_ascii_case("fp") {
        scaffold_fp_primitives(&app_dir, &domain_dir, project_name, files)?;
    }

    // BaseDbContext.cs
    let db_context_name = opts.db_context.unwrap_or("BaseDbContext");
    let db_context_path = persistence_dir.join("Contexts").join(format!("{}.cs", db_context_name));
    let db_context_content = format!(
        r#"namespace {project_name}.Persistence.Contexts;

using Microsoft.EntityFrameworkCore;
using System.Reflection;

public class {db_context_name} : DbContext
{{
    public {db_context_name}(DbContextOptions<{db_context_name}> options) : base(options)
    {{
    }}

    protected override void OnModelCreating(ModelBuilder modelBuilder)
    {{
        modelBuilder.ApplyConfigurationsFromAssembly(Assembly.GetExecutingAssembly());
        base.OnModelCreating(modelBuilder);
    }}
}}
"#
    );
    fs::write(&db_context_path, db_context_content)?;
    files.push(db_context_path);

    // Program.cs
    let program_path = api_dir.join("Program.cs");
    let program_content = format!(
        r#"using {project_name}.Persistence.Contexts;
using Microsoft.EntityFrameworkCore;

var builder = WebApplication.CreateBuilder(args);

builder.Services.AddControllers();
builder.Services.AddEndpointsApiExplorer();

// Database configuration (SQLite default)
builder.Services.AddDbContext<{db_context_name}>(options =>
    options.UseSqlite(builder.Configuration.GetConnectionString("DefaultConnection") ?? "Data Source=app.db"));

var app = builder.Build();

using (var scope = app.Services.CreateScope())
{{
    var db = scope.ServiceProvider.GetRequiredService<{db_context_name}>();
    db.Database.EnsureCreated();
}}

app.UseHttpsRedirection();
app.UseAuthorization();
app.MapControllers();

app.Run();
"#
    );
    fs::write(&program_path, program_content)?;
    files.push(program_path);

    // appsettings.json
    let appsettings_path = api_dir.join("appsettings.json");
    let appsettings_content = r#"{
  "Logging": {
    "LogLevel": {
      "Default": "Information",
      "Microsoft.AspNetCore": "Warning"
    }
  },
  "ConnectionStrings": {
    "DefaultConnection": "Data Source=app.db"
  },
  "AllowedHosts": "*"
}
"#;
    fs::write(&appsettings_path, appsettings_content)?;
    files.push(appsettings_path);

    Ok(())
}

#[allow(dead_code)]
fn scaffold_flat(
    project_dir: &Path,
    project_name: &str,
    opts: &CreateProjectOptions,
    files: &mut Vec<PathBuf>,
) -> Result<()> {
    let src_dir = project_dir.join("src").join(project_name);
    fs::create_dir_all(src_dir.join("Modules"))?;
    fs::create_dir_all(src_dir.join("Data"))?;

    if !opts.skip_dotnet_exec && Command::new("dotnet").arg("--version").output().is_ok() {
        let _ = Command::new("dotnet")
            .args(["new", "webapi", "-n", project_name, "-o", &format!("src/{}", project_name)])
            .current_dir(project_dir)
            .output();
    } else {
        write_csproj(&src_dir.join(format!("{}.csproj", project_name)), true, &["Microsoft.EntityFrameworkCore", "Microsoft.EntityFrameworkCore.Sqlite"], &[])?;
    }

    if opts.paradigm.eq_ignore_ascii_case("fp") {
        scaffold_fp_primitives(&src_dir, &src_dir, project_name, files)?;
    }

    let db_context_name = opts.db_context.unwrap_or("BaseDbContext");
    let db_context_path = src_dir.join("Data").join(format!("{}.cs", db_context_name));
    let db_context_content = format!(
        r#"namespace {project_name}.Data;

using Microsoft.EntityFrameworkCore;

public class {db_context_name} : DbContext
{{
    public {db_context_name}(DbContextOptions<{db_context_name}> options) : base(options)
    {{
    }}
}}
"#
    );
    fs::write(&db_context_path, db_context_content)?;
    files.push(db_context_path);

    let program_path = src_dir.join("Program.cs");
    let program_content = format!(
        r#"using {project_name}.Data;
using Microsoft.EntityFrameworkCore;

var builder = WebApplication.CreateBuilder(args);
builder.Services.AddControllers();
builder.Services.AddDbContext<{db_context_name}>(opt => opt.UseSqlite("Data Source=app.db"));

var app = builder.Build();
app.MapControllers();
app.Run();
"#
    );
    fs::write(&program_path, program_content)?;
    files.push(program_path);

    Ok(())
}

#[allow(dead_code)]
fn scaffold_layered(
    project_dir: &Path,
    project_name: &str,
    opts: &CreateProjectOptions,
    files: &mut Vec<PathBuf>,
) -> Result<()> {
    let src_dir = project_dir.join("src");
    let api_dir = src_dir.join("Api");
    let core_dir = src_dir.join(format!("{}.Core", project_name));

    fs::create_dir_all(&api_dir)?;
    fs::create_dir_all(core_dir.join("Modules"))?;
    fs::create_dir_all(core_dir.join("Data"))?;

    if !opts.skip_dotnet_exec && Command::new("dotnet").arg("--version").output().is_ok() {
        let _ = Command::new("dotnet")
            .args(["new", "sln", "-n", project_name])
            .current_dir(project_dir)
            .output();
    } else {
        write_csproj(&api_dir.join(format!("{}.Api.csproj", project_name)), true, &[], &[&format!("../{0}.Core/{0}.Core.csproj", project_name)])?;
        write_csproj(&core_dir.join(format!("{}.Core.csproj", project_name)), false, &["Microsoft.EntityFrameworkCore", "Microsoft.EntityFrameworkCore.Sqlite"], &[])?;
    }

    if opts.paradigm.eq_ignore_ascii_case("fp") {
        scaffold_fp_primitives(&core_dir, &core_dir, project_name, files)?;
    }

    let db_context_name = opts.db_context.unwrap_or("BaseDbContext");
    let db_context_path = core_dir.join("Data").join(format!("{}.cs", db_context_name));
    let db_context_content = format!(
        r#"namespace {project_name}.Core.Data;

using Microsoft.EntityFrameworkCore;

public class {db_context_name} : DbContext
{{
    public {db_context_name}(DbContextOptions<{db_context_name}> options) : base(options)
    {{
    }}
}}
"#
    );
    fs::write(&db_context_path, db_context_content)?;
    files.push(db_context_path);

    Ok(())
}

fn scaffold_fp_primitives(
    app_dir: &Path,
    domain_dir: &Path,
    project_name: &str,
    files: &mut Vec<PathBuf>,
) -> Result<()> {
    // 1. Result<T, TE>
    let functional_dir = app_dir.join("Common").join("Functional");
    fs::create_dir_all(&functional_dir)?;
    let result_path = functional_dir.join("Result.cs");
    let result_content = format!(
        r#"namespace {project_name}.Common.Functional;

using System;
using System.Threading.Tasks;

public readonly record struct Result<T, TE>
{{
    public bool IsSuccess {{ get; }}
    public bool IsFailure => !IsSuccess;

    private readonly T? _value;
    private readonly TE? _error;

    private Result(T value)
    {{
        IsSuccess = true;
        _value = value;
        _error = default;
    }}

    private Result(TE error)
    {{
        IsSuccess = false;
        _value = default;
        _error = error;
    }}

    public T Value => IsSuccess ? _value! : throw new InvalidOperationException($"Cannot access Value of failed result: {{_error}}");
    public TE Error => IsFailure ? _error! : throw new InvalidOperationException("Cannot access Error of successful result.");

    public static Result<T, TE> Success(T value) => new(value);
    public static Result<T, TE> Failure(TE error) => new(error);

    public static implicit operator Result<T, TE>(T value) => new(value);
    public static implicit operator Result<T, TE>(TE error) => new(error);

    public TResult Match<TResult>(Func<T, TResult> onSuccess, Func<TE, TResult> onFailure) =>
        IsSuccess ? onSuccess(_value!) : onFailure(_error!);

    public async Task<TResult> MatchAsync<TResult>(Func<T, Task<TResult>> onSuccess, Func<TE, Task<TResult>> onFailure) =>
        IsSuccess ? await onSuccess(_value!) : await onFailure(_error!);

    public Result<TOut, TE> Map<TOut>(Func<T, TOut> mapper) =>
        IsSuccess ? Result<TOut, TE>.Success(mapper(_value!)) : Result<TOut, TE>.Failure(_error!);

    public Result<TOut, TE> Bind<TOut>(Func<T, Result<TOut, TE>> binder) =>
        IsSuccess ? binder(_value!) : Result<TOut, TE>.Failure(_error!);
}}
"#
    );
    fs::write(&result_path, result_content)?;
    files.push(result_path);

    // 2. ICommand / IQuery pipeline
    let pipeline_dir = app_dir.join("Common").join("Pipeline");
    fs::create_dir_all(&pipeline_dir)?;
    let pipeline_path = pipeline_dir.join("ICommand.cs");
    let pipeline_content = format!(
        r#"namespace {project_name}.Common.Pipeline;

using System.Threading;
using System.Threading.Tasks;

public interface ICommand<out TResponse> {{ }}
public interface IQuery<out TResponse> {{ }}

public interface ICommandHandler<in TCommand, TResponse> where TCommand : ICommand<TResponse>
{{
    ValueTask<TResponse> HandleAsync(TCommand command, CancellationToken ct = default);
}}

public interface IQueryHandler<in TQuery, TResponse> where TQuery : IQuery<TResponse>
{{
    ValueTask<TResponse> HandleAsync(TQuery query, CancellationToken ct = default);
}}
"#
    );
    fs::write(&pipeline_path, pipeline_content)?;
    files.push(pipeline_path);

    // 3. DomainError
    let exceptions_dir = domain_dir.join("Domain").join("Common").join("Exceptions");
    let exc_dir = if domain_dir.ends_with("Domain") {
        domain_dir.join("Common").join("Exceptions")
    } else {
        exceptions_dir
    };
    fs::create_dir_all(&exc_dir)?;
    let domain_error_path = exc_dir.join("DomainError.cs");
    let domain_error_content = format!(
        r#"namespace {project_name}.Domain.Common.Exceptions;

public record DomainError(string Code, string Message)
{{
    public sealed record NotFound(string EntityName, string Key)
        : DomainError("NOT_FOUND", $"{{EntityName}} with key '{{Key}}' was not found.");

    public sealed record Validation(string Field, string Message)
        : DomainError("VALIDATION_ERROR", $"{{Field}}: {{Message}}");

    public sealed record Conflict(string Message)
        : DomainError("CONFLICT", Message);

    public sealed record BusinessRuleViolation(string RuleCode, string Reason)
        : DomainError(RuleCode, Reason);

    public sealed record Unauthorized(string Reason = "Authentication is required.")
        : DomainError("UNAUTHORIZED", Reason);

    public sealed record Forbidden(string RequiredPermission)
        : DomainError("FORBIDDEN", $"Action requires permission: {{RequiredPermission}}");
}}
"#
    );
    fs::write(&domain_error_path, domain_error_content)?;
    files.push(domain_error_path);

    Ok(())
}

#[allow(dead_code)]
fn write_csproj(path: &Path, is_executable: bool, packages: &[&str], project_refs: &[&str]) -> Result<()> {
    let output_type = if is_executable { "\n    <OutputType>Exe</OutputType>" } else { "" };
    let sdk = if is_executable { "Microsoft.NET.Sdk.Web" } else { "Microsoft.NET.Sdk" };

    let mut pkg_lines = String::new();
    if !packages.is_empty() {
        pkg_lines.push_str("  <ItemGroup>\n");
        for pkg in packages {
            pkg_lines.push_str(&format!("    <PackageReference Include=\"{}\" Version=\"9.0.0\" />\n", pkg));
        }
        pkg_lines.push_str("  </ItemGroup>\n");
    }

    let mut ref_lines = String::new();
    if !project_refs.is_empty() {
        ref_lines.push_str("  <ItemGroup>\n");
        for r in project_refs {
            ref_lines.push_str(&format!("    <ProjectReference Include=\"{}\" />\n", r));
        }
        ref_lines.push_str("  </ItemGroup>\n");
    }

    let content = format!(
        r#"<Project Sdk="{}">
  <PropertyGroup>
    <TargetFramework>net9.0</TargetFramework>
    <Nullable>enable</Nullable>
    <ImplicitUsings>enable</ImplicitUsings>{}
  </PropertyGroup>
{}{}
</Project>
"#,
        sdk, output_type, pkg_lines, ref_lines
    );

    fs::write(path, content)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_create_project_clean_cqrs() -> Result<()> {
        let dir = tempdir()?;
        let opts = CreateProjectOptions {
            name: "OrderService",
            target_path: dir.path(),
            arch: ArchStyle::CleanCqrs,
            host_type: HostType::Api,
            dispatcher: Dispatcher::Native,
            paradigm: "fp",
            profile: "standard",
            db_context: None,
            skip_dotnet_exec: true,
        };

        let res = create_project(&opts)?;
        assert!(res.project_dir.join("clrinf.toml").exists());
        assert!(res.project_dir.join("Directory.Build.props").exists());
        assert!(res.project_dir.join("src").join("Api").join("Program.cs").exists());
        assert!(res.project_dir.join("src").join("Persistence").join("Contexts").join("BaseDbContext.cs").exists());
        assert!(res.project_dir.join("src").join("Application").join("Common").join("Functional").join("Result.cs").exists());
        assert!(res.project_dir.join("src").join("Application").join("Common").join("Pipeline").join("ICommand.cs").exists());

        let toml_str = fs::read_to_string(res.project_dir.join("clrinf.toml"))?;
        assert!(toml_str.contains("OrderService"));
        assert!(toml_str.contains("clean-cqrs"));
        assert!(toml_str.contains("fp"));
        assert!(toml_str.contains("native"));

        Ok(())
    }
}
