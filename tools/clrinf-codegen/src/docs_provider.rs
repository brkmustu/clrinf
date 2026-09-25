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
- **ARCH001 (Zero Leakage):** Domain katmanında EF Core, ASP.NET Core, Web veya JSON kütüphaneleri YASAKTIR. Saf C# olmalıdır.
- **ARCH002 (Business Rules):** İş kuralları `IBusinessRule<T>` uygulamalı, `EvaluateAsync(entity, ctx)` sunmalıdır. Asla çıplak `throw new Exception()` fırlatma; `RuleResult.Success()` veya `RuleResult.Failure(code, message)` dön.
- **ARCH003 (Controller Isolation):** Web API Controller'larına `DbContext` doğrudan enjekte edilemez; CQRS Handler zorunludur.
- **ARCH004 (CQRS Contract):** Komut ve sorgular `IRequest<T>` veya `IRequireOperationClaim` sözleşmelerine uymalıdır.

## 2. Idiomatic Code Patterns
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
- Her komutta tenant izolasyonu: `[CedarAuthorize("orders.create", "Orders")]`.
- Cross-tenant veri erişimi `forbid` kuralıyla derhal engellenir.

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
- **RUST_ARCH001 (Zero-Panic Policy):** Domain modelleri ve iş kuralı fonksiyonlarında `.unwrap()`, `.expect()`, `panic!()` kullanımı KESİNLİKLE YASAKTIR. Derleme hatası verir.
- Tüm operasyonlar `Result<T, DomainError>` monadı ile dönmelidir.

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
- Ham modda (`--mode raw`): Modül bağımsız kural ve struct'ları içerir, harici crate bağımlılığı eklemez.
- Kablolanmış modda (`--mode wired`): `src/lib.rs` veya `src/main.rs` içine otomatik `pub mod <modul>;` eklenir.

## 4. Useful clrinf Commands
- `clrinf module adopt <crm|deals|authz|caching> --to-project ./Cargo.toml --mode raw|wired`
- `clrinf lint --lang rust`
"#;

const TYPESCRIPT_DOCS: &str = r#"# clrinf TypeScript Development & Architecture Guide

## 1. Architectural Guardrails (TS Compiler API Enforced)
- **ARCH_TS_001 (No Naked Throws):** İş kurallarında çıplak `throw new Error()` yasaktır. Fonksiyonel `Result.err()` ve `ruleFailed()` kullanılmalıdır.
- **ARCH_TS_003 (Functional Domain):** C# MediatR taklidi hantal sınıf hiyerarşileri yerine saf fonksiyonlar ve kompozisyon boru hatları (`pipeRules`) tercih edilmelidir.

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
- Ham modda (`--mode raw`): Bağımsız saf TypeScript fonksiyonları ve Result monadı.
- Kablolanmış modda (`--mode wired`): `src/index.ts` dosyasına `export * as <module> from './<module>';` barrel export eklenir.

## 4. Useful clrinf Commands
- `clrinf module adopt <crm|deals|authz|caching> --to-project ./package.json --mode raw|wired`
- `clrinf lint --lang typescript`
"#;

const ELIXIR_DOCS: &str = r#"# clrinf Elixir (OTP) Development & Architecture Guide

## 1. Architectural Guardrails (AST Linter Enforced: ARCH_EX_*)
- **ARCH_EX_001 (Explicit OTP Callbacks):** `GenServer` callback fonksiyonlarında (`init/1`, `handle_call/3`, `handle_cast/2`, `handle_info/2`, `terminate/2`) `@impl true` zorunludur.
- **ARCH_EX_002 (Pure Domain Result Monad):** Domain ve Handler fonksiyonları `{:ok, term()}` veya `{:error, term()}` dönmelidir. Pure domain içinde `raise` veya `throw` yasaktır.
- **ARCH_EX_003 (Supervised Concurrency):** Domain mantığı içinde ad-hoc `spawn/1`, `spawn_link/1` veya `Task.start/1` yasaktır. Eşzamanlılık bir Supervisor veya GenServer ile yönetilmelidir.
- **ARCH_EX_004 (Side-Effect Isolation):** Pure domain/handler modüllerinde `IO.puts`, `Process.sleep`, `:timer.sleep`, `System.cmd` yasaktır; yan etkiler adaptörlerde veya GenServer sınırında olmalıdır.

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

clrinf ekosisteminde yetkilendirme kod içine `if (user.TenantId == res.TenantId)` şeklinde gömülmez; bağımsız `policies/<module>.cedar` dosyalarında yönetilir.

## 3 Altın Kural:
1. **Varsayılan Olarak Çok Kiracılı (Multi-Tenant by Default):**
```cedar
permit (
    principal in Role::"TenantAdmin",
    action,
    resource in ResourceType::"Deals"
) when {
    resource.tenant_id == principal.tenant_id
};
```

2. **Forbid Kuralının Mutlak Önceliği (Overriding Forbid Guard):**
Cedar'da `forbid` tüm `permit` kurallarını ezer. Başka bir kiracının verisine erişim koşulsuz reddedilir:
```cedar
forbid (
    principal,
    action,
    resource in ResourceType::"Deals"
) when {
    resource.tenant_id != principal.tenant_id
};
```

3. **PlatformAdmin (Süper Kullanıcı) İstisnası:**
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
- `clrinf catalog`: Yerleşik modül kataloğunu listeler (crm, deals, contacts, activities, authz, caching, logging, transaction, idempotency, outbox).
- `clrinf module adopt <MODUL> --to-project <PROJE> [--mode raw|wired] [--as <ALIAS>]`: Modülü hedef projeye sıfır sürtünmeyle aktarır.
- `clrinf init --profile <minimal|standard|full> --dispatcher <native|mediatr>`: Yeni projede clrinf.toml manifesti oluşturur.
- `clrinf scaffold otp <MODULE> [--app <APP>]`: Elixir OTP Worker, Supervisor ve Handler üçlüsünü üretir.
- `clrinf rule new <RULE> --lang <csharp|rust|typescript|elixir>`: İzole saf iş kuralı üretir.
- `clrinf lint --lang <csharp|rust|typescript|elixir|all>`: AST mimari denetleyicilerini çalıştırır.
- `clrinf mcp`: Yapay zeka ajanları için JSON-RPC 2.0 MCP sunucusunu başlatır.
"#;

const ALL_DOCS: &str = r#"# clrinf Multi-Language Architecture & Guidelines

clrinf; C#, Rust, TypeScript ve Elixir dillerinde Clean Architecture, CQRS / OTP, monadik Result hata yönetimi ve Amazon Cedar çok kiracılı yetkilendirme sağlayan sözleşme-öncelikli bir framework'tür.

Dile özel kurallar için `lang` parametresiyle çağırın:
- `csharp`: Roslyn ARCH001-ARCH004, IBusinessRule, Native Dispatcher
- `rust`: Syn RUST_ARCH001 zero-panic, Result<T, DomainError>
- `typescript`: TS AST ARCH_TS_001, pipeRules, Result.err
- `elixir`: AST ARCH_EX_001-004, OTP GenServer/Supervisor, Result monad
- `cedar`: Çok kiracılı Amazon Cedar politikaları
- `cli`: clrinf-codegen komutları ve modül adaptasyonu
"#;
