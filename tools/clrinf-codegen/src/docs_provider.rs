use anyhow::Result;
use std::path::Path;

/// Detects the target language of a project directory or file.
pub fn detect_language(path: &Path) -> Option<&'static str> {
    if path.is_file() {
        let file_name = path.file_name()?.to_str()?;
        if file_name.ends_with(".csproj") {
            return Some("csharp");
        }
        if file_name == "Cargo.toml" {
            return Some("rust");
        }
        if file_name == "package.json" {
            return Some("typescript");
        }
        if file_name == "mix.exs" {
            return Some("elixir");
        }
    } else if path.is_dir() {
        // Check for project files in directory
        if let Ok(entries) = std::fs::read_dir(path) {
            let mut has_csproj = false;
            let mut has_cargo = false;
            let mut has_package = false;
            let mut has_mix = false;

            for entry in entries.flatten() {
                let name = entry.file_name();
                let name_str = name.to_string_lossy();
                if name_str.ends_with(".csproj") {
                    has_csproj = true;
                } else if name_str == "Cargo.toml" {
                    has_cargo = true;
                } else if name_str == "package.json" {
                    has_package = true;
                } else if name_str == "mix.exs" {
                    has_mix = true;
                }
            }

            if has_csproj {
                return Some("csharp");
            }
            if has_cargo {
                return Some("rust");
            }
            if has_package {
                return Some("typescript");
            }
            if has_mix {
                return Some("elixir");
            }
        }
    }
    None
}

/// Provides token-budgeted, language-isolated architectural documentation and rules.
pub fn get_language_docs(lang: Option<&str>, project_path: Option<&Path>) -> Result<String> {
    let resolved_lang = if let Some(l) = lang {
        l.to_lowercase()
    } else if let Some(p) = project_path {
        detect_language(p)
            .unwrap_or("all")
            .to_string()
    } else {
        "all".to_string()
    };

    let content = match resolved_lang.as_str() {
        "csharp" | "cs" | "dotnet" | "c#" => CSHARP_DOCS,
        "rust" | "rs" => RUST_DOCS,
        "typescript" | "ts" | "javascript" | "js" => TYPESCRIPT_DOCS,
        "elixir" | "ex" => ELIXIR_DOCS,
        "cedar" | "authz" | "security" => CEDAR_DOCS,
        "cli" | "codegen" | "tools" => CLI_DOCS,
        "all" => ALL_DOCS,
        unknown => {
            return Ok(format!(
                "# Unknown Language: {}\nAvailable options: 'csharp', 'rust', 'typescript', 'elixir', 'cedar', 'cli', 'all'.",
                unknown
            ));
        }
    };

    Ok(content.trim().to_string())
}

const CSHARP_DOCS: &str = r#"# clrinf C# (.NET) Development & Architecture Guide

## 1. Architectural Guardrails (Roslyn Enforced)
- **ARCH001 (Zero Leakage):** Domain layers MUST NOT import EF Core, ASP.NET Core, Web, or JSON libraries. Keep domain logic pure C#.
- **ARCH002 (Business Rules):** Implement `IBusinessRule<T>`, `IBeforeCreateRule<T>`, `IBeforeUpdateRule<T>`, or `IBeforeDeleteRule<T>`, returning `RuleResult.Success()` or `RuleResult.Failure(code, message)`. Never throw raw `new Exception()` for business errors.
- **ARCH003 (Controller Isolation):** Never inject `DbContext` directly into Web API controllers; use CQRS handlers.
- **ARCH004 (CQRS Contract):** Commands and queries must implement `IRequest<T>` or `IRequireOperationClaim`.
- **ARCH005 (Hook Placement):** Place `IBefore*Rule<TEntity>` hooks under `Rules/`; `*Hook` classes must implement the corresponding hook interface.

## 2. AOT CRUD Pattern
- Annotate entities with `[GenerateCrudEndpoints]` so standard CRUD handlers and Minimal API endpoints are source-generated at compile time. Do not read generated files under `obj/`.
- Scaffold lifecycle rules with `clrinf rule new ActiveProductsCheck --lang csharp --entity Category --on before-delete` (generates `IBeforeDeleteRule<Category>` executed by `CrudRuleRunner`).
- Eject a single operation to a physical file via `clrinfcs eject Category --operation Update`; the source generator skips that operation via Smart Override.
- Mark tenant fields with `[CrudIgnore]` on the entity; `Id` and tenant ID are never bound from client DTOs.
```csharp
// Business Rule
public sealed class OrderMinimumAmountRule : IBusinessRule<Order>
{
    private readonly decimal _min;
    public OrderMinimumAmountRule(decimal min) => _min = min;

    public ValueTask<RuleResult> EvaluateAsync(Order order, RequestContext ctx, CancellationToken ct = default)
    {
        if (order.TotalAmount < _min)
            return ValueTask.FromResult(RuleResult.Failure("BELOW_MIN_AMOUNT", $"Minimum order amount is {_min}"));
        return ValueTask.FromResult(RuleResult.Success());
    }
}

// CQRS Handler & Native Dispatcher
public sealed record CreateOrderCommand(string CustomerId, decimal Amount) : IRequest<Result<string>>;

public sealed class CreateOrderHandler : IRequestHandler<CreateOrderCommand, Result<string>>
{
    public ValueTask<Result<string>> HandleAsync(CreateOrderCommand cmd, CancellationToken ct = default)
    {
        // Business logic here
        return ValueTask.FromResult(Result<string>.Success(Guid.NewGuid().ToString()));
    }
}
```

## 3. Cedar Authorization & Multi-Tenancy
- Enforce tenant isolation on every command: `[CedarAuthorize("orders.create", "Orders")]`.
- Cross-tenant data access is unconditionally denied via `forbid` rules.

## 4. Modular Layout Strategies (Containerized vs Direct Root Slices)
`clrinf` supports two first-class module directory layout patterns:
- **Containerized Modules (`Modules/<Module>/`):**
  - Configured via `[backend.structure] folder_name = "Modules"` in `clrinf.toml` or `codegen.toml`.
  - Used in enterprise monoliths (e.g. `CrmMonolith`) with many features to separate business domains from infrastructure (`Data/`, `Common/`, `Policies/`).
  - Namespace: `<Project>.Modules.<Module>`.
- **Direct Bounded Contexts (`<Module>/` at root):**
  - Configured when `folder_name` is omitted or set to root.
  - Used in domain-driven systems (e.g. `EcommerceMonolith`) where bounded contexts (`Catalog`, `Inventory`, `Orders`) are primary first-class citizens.
  - Internal module layout uses low-cognitive-load `flat` decomposition: `<Module>Objects.cs`, `<Module>Rules.cs`, `<Module>Handlers.cs`, `<Module>Module.cs`.
  - Namespace: `<Project>.<Module>`.

## 5. Useful clrinf Commands
- `clrinf module adopt <crm|deals|authz|caching> --to-project ./MyService.csproj --mode raw|wired`
- `clrinf add module <Name> --pattern <flat|basic>`
- `clrinf lint --lang csharp`
"#;

const RUST_DOCS: &str = r#"# clrinf Rust Development & Architecture Guide

## 1. Architectural Guardrails (Syn Linter Enforced)
- **RUST_ARCH001 (Zero-Panic Policy):** `.unwrap()`, `.expect()`, and `panic!()` are FORBIDDEN in domain models and business rule functions.
- All domain operations must return `Result<T, DomainError>`.

## 2. Idiomatic Code Patterns
```rust
use clrinf_core::rules::{BusinessRule, RulePipeline};
use clrinf_core::error::DomainError;

pub struct OrderMinimumAmountRule {
    pub min_amount: f64,
}

impl BusinessRule<Order> for OrderMinimumAmountRule {
    fn check(&self, order: &Order) -> Result<(), DomainError> {
        if order.total_amount < self.min_amount {
            return Err(DomainError::RuleViolation("Below minimum order amount".into()));
        }
        Ok(())
    }
}

// Rule Execution Pipeline
let pipeline = RulePipeline::new()
    .add_rule(OrderMinimumAmountRule { min_amount: 100.0 });
pipeline.validate(&order)?;
```

## 3. Module Tree & Wiring
- Raw mode (`--mode raw`): Generates standalone rules and structs with zero external crate dependencies.
- Wired mode (`--mode wired`): Automatically registers `pub mod <module>;` in `src/lib.rs` or `src/main.rs`.

## 4. Useful clrinf Commands
- `clrinf module adopt <crm|deals|authz|caching> --to-project ./Cargo.toml --mode raw|wired`
- `clrinf lint --lang rust`
"#;

const TYPESCRIPT_DOCS: &str = r#"# clrinf TypeScript Development & Architecture Guide

## 1. Architectural Guardrails (TS Compiler API Enforced)
- **ARCH_TS_001 (No Naked Throws):** Bare `throw new Error()` is forbidden in business rules. Return functional `Result.err()` / `ruleFailed()`.
- **ARCH_TS_003 (Functional Domain):** Prefer pure functions and composition pipelines (`pipeRules`) over heavy class hierarchies.

## 2. Idiomatic Code Patterns
```typescript
import { Result, type Rule, rulePassed, ruleFailed, pipeRules } from "@clrinf/core";

export const validateMinimumAmount = (minAmount: number): Rule<Order> => 
  (order: Order) => {
    if (order.totalAmount < minAmount) {
      return ruleFailed("BELOW_MIN_AMOUNT", `Amount must be at least ${minAmount}`);
    }
    return rulePassed();
  };

// Chaining rules with functional pipe:
export const validateOrder = pipeRules(
  validateMinimumAmount(100),
  validateCustomerStatus
);
```

## 3. Module Tree & Wiring
- Raw mode (`--mode raw`): Standalone pure TypeScript functions and `Result` monad.
- Wired mode (`--mode wired`): Adds `export * as <module> from './<module>';` barrel export in `src/index.ts`.

## 4. Useful clrinf Commands
- `clrinf module adopt <crm|deals|authz|caching> --to-project ./package.json --mode raw|wired`
- `clrinf lint --lang typescript`
"#;

const ELIXIR_DOCS: &str = r#"# clrinf Elixir (OTP) Development & Architecture Guide

## 1. Architectural Guardrails (AST Linter Enforced: ARCH_EX_*)
- **ARCH_EX_001 (Explicit OTP Callbacks):** `@impl true` is required on `GenServer` callbacks (`init/1`, `handle_call/3`, `handle_cast/2`, `handle_info/2`, `terminate/2`).
- **ARCH_EX_002 (Pure Domain Result Monad):** Domain and handler functions must return `{:ok, term()}` or `{:error, term()}`. `raise` and `throw` are forbidden in pure domain modules.
- **ARCH_EX_003 (Supervised Concurrency):** Ad-hoc `spawn/1`, `spawn_link/1`, or `Task.start/1` are forbidden in domain logic; use a `Supervisor` or `GenServer`.
- **ARCH_EX_004 (Side-Effect Isolation):** `IO.puts`, `Process.sleep`, `:timer.sleep`, and `System.cmd` are forbidden in pure domain/handler modules; keep side effects in adapters or `GenServer` boundaries.

## 2. Idiomatic Code Patterns
```elixir
# 1. Pure Functional Handler
defmodule MyApp.Order.Handler do
  @spec execute(map()) :: {:ok, map()} | {:error, term()}
  def execute(params) do
    with {:ok, valid} <- validate(params),
         {:ok, processed} <- process(valid) do
      {:ok, processed}
    end
  end
end

# 2. OTP GenServer Worker
defmodule MyApp.Order.Worker do
  use GenServer

  @impl true
  def init(_opts), do: {:ok, %{}}

  @impl true
  def handle_call({:execute, payload}, _from, state) do
    case MyApp.Order.Handler.execute(payload) do
      {:ok, res} -> {:reply, {:ok, res}, state}
      {:error, err} -> {:reply, {:error, err}, state}
    end
  end
end
```

## 3. Useful clrinf Commands
- `clrinf scaffold otp OrderProcessor --app my_app`
- `clrinf rule new MinimumAmountRule --lang elixir --entity Order`
- `clrinf lint --lang elixir`
"#;

const CEDAR_DOCS: &str = r#"# Amazon Cedar Multi-Tenant Authorization Guide

In clrinf, authorization is never hardcoded inline as `if (user.TenantId == res.TenantId)`; it is declared in standalone `policies/<module>.cedar` files.

## 3 Golden Rules:
1. **Multi-Tenant by Default:**
```cedar
permit (
    principal in Role::"TenantAdmin",
    action,
    resource in ResourceType::"Deals"
) when {
    resource.tenant_id == principal.tenant_id
};
```

2. **Overriding Forbid Guard:**
In Cedar, `forbid` overrides every `permit`. Cross-tenant access is unconditionally denied:
```cedar
forbid (
    principal,
    action,
    resource in ResourceType::"Deals"
) when {
    resource.tenant_id != principal.tenant_id
};
```

3. **PlatformAdmin Exception:**
```cedar
permit (
    principal in Role::"PlatformAdmin",
    action,
    resource
);
```
"#;

const CLI_DOCS: &str = r#"# clrinf CLI & MCP Tools Summary

## Key Commands
- `clrinf verify`: Runs event topology, code<->contract drift, `[[forbid]]` import rules, and raw outbox write checks.
- `clrinf plan <event> --service <svc> --role publisher|subscriber`: Previews/applies mechanical edits and outputs a tight agent slice.
- `clrinf hook install --agent <claude|cursor|antigravity|git|all>`: Installs incremental post-edit guardrails.
- `clrinf catalog`: Lists built-in modules (`crm`, `deals`, `contacts`, `activities`, `authz`, `caching`, `logging`, `transaction`, `idempotency`, `outbox`).
- `clrinf module adopt <MODULE> --to-project <PROJECT> [--mode raw|wired] [--as <ALIAS>]`: Adopts a module into a target project with zero lock-in.
- `clrinf init --profile <minimal|standard|full> --dispatcher <native|mediatr>`: Initializes a `clrinf.toml` manifest.
- `clrinf scaffold otp <MODULE> [--app <APP>]`: Generates Elixir OTP Worker, Supervisor, and Handler modules.
- `clrinf rule new <RULE> --lang <csharp|rust|typescript|elixir>`: Scaffolds an isolated business rule and test.
- `clrinf lint --lang <csharp|rust|typescript|elixir|all>`: Runs language-specific architectural AST linters.
- `clrinf mcp [--profile full|events|lean]`: Starts the JSON-RPC 2.0 MCP stdio server for AI coding agents.
"#;

const ALL_DOCS: &str = r#"# clrinf Multi-Language Architecture & Guidelines

clrinf provides deterministic architecture & event-contract guardrails plus optional reference cores and zero-dependency scaffolds across C#, Rust, TypeScript, and Elixir.

Call with `lang` for language-specific rules:
- `csharp`: Roslyn ARCH001-ARCH005, IBusinessRule / IBefore*Rule, Native Dispatcher, AOT CRUD generator
- `rust`: Syn RUST_ARCH001 zero-panic, Result<T, DomainError>
- `typescript`: TS AST ARCH_TS_001, pipeRules, Result.err
- `elixir`: AST ARCH_EX_001-004, OTP GenServer/Supervisor, Result monad
- `cedar`: Multi-tenant Amazon Cedar policies
- `cli`: clrinf-codegen commands, verify/hooks, and module adoption
"#;
