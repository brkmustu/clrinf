# clrinfex — Elixir Kod Üretim + Mimari Linter Desteği (Final Plan)

## Amaç ve Stratejik Bağlam

Hedefimiz: **yerelde çalışan veya bulut tabanlı LLM'lerin (SLM / local LLM) clrinf araçları üzerinden, mimari kalite standartlarını koruyarak Elixir kodu üretebilmesi.**

Elixir, fonksiyonel programlama ve BEAM/OTP (Supervision Tree, GenServer, hafif aktör modeli) üzerinde benzersiz bir güce sahiptir. Bu plan, Elixir'in en güçlü olduğu bu alanda:
1. **İdiomatik OTP Kod Üretimi** (Worker / Supervisor / Handler üçlüsü ve saf Business Rule scaffold'ları),
2. **Fonksiyonel Mimari Linter** (`ARCH_EX_*` — AST tabanlı, dilin doğasına uygun "olmazsa olmaz" 4 kural),
3. **Standart CLI & MCP Entegrasyonu** (C#, Rust ve TypeScript ile birebir aynı standarda sahip komut seti),
4. **Çift Geliştirici Deneyimi** (Hem sıfırdan proje başlatan **Greenfield**, hem var olan projeye kod ekleyen **Brownfield** desteği)
sağlar.

---

## Mimari Karar: Rust ↔ Elixir İş Bölümü (Temel Prensip)

> **Prensip:** Kod üretimi ve genel orkestrasyon Rust `clrinf-codegen` üzerinde toplanır. Dilin kendi AST'sini gerektiren mimari linter ise Elixir içinde bir `mix` task olarak yaşar ve Rust tarafından bir worker olarak delege edilir.

| Sorumluluk | Nerede | Neden / Standart |
|---|---|---|
| **Kod üretimi** (string templating) | Rust `clrinf-codegen` + Tera | Hızlı, tek merkez, tek CLI binary, sıfır çalışma zamanı bağımlılığı |
| **Mimari linter** (AST analizi) | Elixir `clrinfex` Mix task | Roslyn, Syn ve TS Compiler API gibi; `.ex` AST'si dilin kendi mekanizmasıyla (`Code.string_to_quoted!`, `Macro.prewalk`) analiz edilmelidir |
| **Orkestrasyon + MCP + Fix-Loop** | Rust `clrinf-codegen` | Tek MCP sunucusu (`clrinf mcp`); linter'ı bir `WorkerKind::Elixir` olarak yönetir |

---

## İki Temel Geliştirici Senaryosu (Greenfield & Brownfield)

Geliştiriciler iki farklı durumda clrinf'e başvurur. Komut setimiz her iki senaryoda da tam ve anlaşılır bir deneyim sunar:

### Senaryo A: Sıfırdan Proje (Greenfield)
Geliştirici yeni bir Elixir projesine clrinf standartlarıyla başlamak ister.

```bash
# 1. Hazır minimal-elixir şablonundan projeyi kur
clrinf new my_service --template minimal-elixir

# 2. Proje dizinine geç
cd my_service

# 3. İhtiyaç duyulan OTP modülünü ekle
clrinf scaffold otp OrderProcessor --app my_service

# 4. Mimari kontrolleri çalıştır
clrinf lint --lang elixir
```

### Senaryo B: Var Olan Projeye Kod Ekleme (Brownfield)
Geliştiricinin zaten var olan bir Elixir/Phoenix projesi vardır (`my_existing_app`).

```bash
cd my_existing_app

# 1. Projeyi clrinf ile tanıştır (mix.exs'i tespit eder, clrinf.toml oluşturur)
clrinf init --lang elixir

# 2. Var olan projeye yeni bir OTP servisi / modülü üret
clrinf scaffold otp PaymentGateway --app my_existing_app
# veya standart modül komutu:
clrinf add module PaymentGateway

# 3. İzole bir iş kuralı (business rule) üret
clrinf rule new ValidatePayment --lang elixir --entity Payment

# 4. Kodun mimariye uygunluğunu doğrula
clrinf lint --lang elixir
```

---

## Bileşen 1 — Rust: OTP & Rule Kod Üretimi (`clrinf-codegen`)

Diğer dillerdeki (C#, Rust, TypeScript) CLI standartlarıyla tam uyumlu, Tera tabanlı üretim.

### 1a. Yeni Tera Şablonları — `clrinf-contracts/templates/elixir/`

```text
clrinf-contracts/templates/elixir/
  otp_worker.tera      # use GenServer; init/1, handle_call/3, handle_cast/2, handle_info/2
  otp_supervisor.tera  # use Supervisor; :one_for_one; OTP ağacına bağlar
  otp_handler.tera     # Pure fonksiyon API'si (with pipeline + {:ok,val} | {:error,err})
  rule.tera            # Pure business rule fonksiyonu + ExUnit testi
```

#### Üretilen 3'lü OTP Mimarisi:
```
lib/<app>/<snake_name>/
  worker.ex         # GenServer — durum ve süreç yönetimi
  supervisor.ex     # Supervisor — OTP denetim ağacı
  handler.ex        # Dışa açık saf fonksiyonel API
```

- **`handler.ex`**: İş mantığını barındıran saf fonksiyonlar. Result disiplinine (`{:ok, val} | {:error, reason}`) uyar, GenServer'a doğrudan bağımlılığı izole eder, birim testleri hızlandırır.
- **`worker.ex`**: Süreç durumunu (state) tutan ve mesajlaşmayı yöneten GenServer. Tüm callback'lerde `@impl true` taşır.
- **`supervisor.ex`**: Worker'ı denetim ağacına bağlayan standart OTP Supervisor modülü (`:one_for_one`).

### 1b. `generator.rs` — Üretim Fonksiyonları

```rust
// tools/clrinf-codegen/src/generator.rs
pub fn scaffold_otp(&self, module: &str, app: &str, output_dir: &Path) -> Result<Vec<PathBuf>>;
pub fn scaffold_rule_elixir(&self, rule_name: &str, entity: Option<&str>, error_code: Option<&str>, target_dir: &Path) -> Result<Vec<PathBuf>>;
```

- Güvenli dosya yazımı (`ensure_safe_output`).
- `module` (örn: `OrderProcessor`) → PascalCase ve snake_case dönüşümleri (`order_processor`).

### 1c. Standartlaştırılmış CLI Komutları (`main.rs`)

Diğer dillerin CLI sözdizimi ve parametre standartlarına sadık kalınır:

```bash
# OTP modülü üretimi (Greenfield & Brownfield)
clrinf scaffold otp <ModuleName> [--app <app>] [--path <dir>]
# Alternatif genel modül komutu:
clrinf add module <ModuleName> [--path <dir>]

# İzole iş kuralı üretimi
clrinf rule new <RuleName> --lang elixir [--entity <Entity>] [--error-code <Code>] [--target-dir <Dir>]

# Mimari denetim
clrinf lint --lang elixir [--path <path>]

# Elixir mimari dokümantasyonu (LLM prompt optimizasyonu için)
clrinf docs --lang elixir
```

---

## Bileşen 2 — Elixir: Fonksiyonel Mimari Linter (`ARCH_EX_*`)

Kullanıcı geri bildirimi doğrultusunda: **Elixir fonksiyonel ve eşzamanlı bir dildir. Kurallar dilin doğasına uygun, sadece "olmazsa olmaz" 4 temel prensip üzerine kuruludur.**

### 2a. 4 Temel Fonksiyonel Kural (`ARCH_EX_*`)

| Kural Kodu | Prensip | Gerekçe / Dilin Doğası |
|---|---|---|
| **`ARCH_EX_001`** | **Explicit OTP Callback Contracts** | `GenServer` callback fonksiyonları (`init/1`, `handle_call/3`, `handle_cast/2`, `handle_info/2`, `terminate/2`) mutlaka `@impl true` (veya `@impl GenServer`) taşımalıdır. İmza kaymalarını ve sessiz hataları derleme aşamasında yakalar. |
| **`ARCH_EX_002`** | **Pure Domain Result Monad** | Domain ve Handler modüllerindeki iş fonksiyonları `{:ok, term()} \| {:error, term()}` tuple dönmelidir. Pure domain içinde kontrolsüz `raise` veya `throw` yasaktır (clrinf Result sözleşmesi). |
| **`ARCH_EX_003`** | **Supervised Concurrency (No Naked Spawns)** | Domain ve Handler iş mantığı içinde ad-hoc / denetimsiz `spawn/1`, `spawn_link/1` veya `Task.start/1` kullanımı yasaktır. Eşzamanlılık mutlaka bir Supervisor (`DynamicSupervisor`, `Task.Supervisor`) veya GenServer arkasında olmalıdır. |
| **`ARCH_EX_004`** | **Pure Domain Side-Effect Isolation** | Saf domain/handler modüllerinde yan etkiler izole edilmelidir (`IO.puts`, `Process.sleep`, `:timer.sleep`, `System.cmd` yasaktır). Yan etkiler adaptörlerde veya GenServer süreç sınırında yönetilmelidir. |

### 2b. Linter Motoru — `lib/clrinfex/linter/arch_linter.ex`

Harici hiçbir hex paketi gerekmez; Elixir'in yerleşik AST mekanizması kullanılır:
```elixir
Code.string_to_quoted(source, file: path)
# -> Macro.prewalk/2 ile AST analizi
```

### 2c. Mix Task — `lib/mix/tasks/clrinfex.lint.ex`

```bash
mix clrinfex.lint [path] [--format human|json]
```

- **Human format**: `[ARCH_EX_001] lib/app/worker.ex:14:3: GenServer callback handle_call/3 must specify @impl true`
- **JSON format** (Rust ve LLM fix-loop için):
```json
{
  "success": false,
  "violations": [
    {
      "code": "ARCH_EX_001",
      "file": "lib/my_app/worker.ex",
      "line": 14,
      "col": 3,
      "message": "GenServer callback handle_call/3 must specify @impl true"
    }
  ]
}
```
- Çıkış kodu: İhlal varsa `1`, temizse `0`.

---

## Bileşen 3 — Rust: Elixir Worker ve Manifest Entegrasyonu

### 3a. `worker.rs` — `WorkerKind::Elixir`

Elixir geliştiricisi makinesinde zaten Elixir/Mix bulundurur. Bu yüzden `mix` doğrudan çalıştırılır:

1. `WorkerKind` enum'ına `Elixir` eklenir (`as_str` -> `"elixir"`, `from_str_loose` -> `"elixir" | "ex"`).
2. `WorkerKind::all()` -> `[CSharp, Rust, TypeScript, Elixir]`.
3. `resolve_elixir_worker`:
   - `mix --version` kontrol edilir.
   - Proje kökünde veya `clrinfex` dizininde `mix` komutu çalıştırılır.
   - `install_hint`: `"Elixir/Mix bulunamadı. Elixir'i kurmak için: https://elixir-lang.org/install.html"`.
4. `lint()` metodu:
   - `mix clrinfex.lint <path> --format json` çalıştırılır.
5. `scaffold_rule()` metodu:
   - Elixir için doğrudan Rust `generator.rs` içindeki `scaffold_rule_elixir` çağrılır veya worker delegasyonu yapılır.

### 3b. `manifest.rs` — Proje Tespiti & `clrinf.toml`

- `Lang::Elixir` eklenir.
- Dizin taramasında `mix.exs` görüldüğünde dil otomatik olarak `Elixir` seçilir.
- `clrinf init` çalıştırıldığında Elixir için uygun profil ve ayarları üretir.

---

## Bileşen 4 — MCP (Model Context Protocol) & Fix-Loop Entegrasyonu

LLM (özellikle yerelde çalışan SLM / Local LLM) döngüsü:

```
┌──────────────────────────────────────────┐
│  LLM / AI Agent                         │
└────┬────────────────────────────────▲────┘
     │ 1. clrinf_scaffold_otp         │ 4. Hataları düzelt
     ▼                                │
┌─────────────────────────┐           │
│ Rust clrinf-codegen     │           │
│ (Şablonları üretir)     │           │
└────┬────────────────────┘           │
     │ 2. Kod yazılır/tamamlanır      │
     ▼                                │
┌─────────────────────────────────────┴────┐
│ 3. clrinf_lint_architecture (Elixir)    │
│    -> mix clrinfex.lint --format json    │
│    -> ARCH_EX_* ihlalleri raporlanır    │
└──────────────────────────────────────────┘
```

Mevcut `clrinf mcp` sunucusuna eklenen / güncellenen araçlar:
1. `clrinf_scaffold_otp`: Module adı, app adı ve hedef yol ile OTP 3'lüsünü üretir.
2. `clrinf_scaffold_rule`: `lang: "elixir"` desteğiyle saf kural ve test dosyası üretir.
3. `clrinf_lint_architecture`: `lang: "elixir"` veya `"all"` çağrıldığında `ARCH_EX_*` denetimini yürütür.
4. `clrinf_get_docs`: `lang: "elixir"` için OTP ve fonksiyonel mimari kılavuzunu döner.
5. `clrinf_inspect_ecosystem`: Ekosistem worker listesinde Elixir'i ve linter kurallarını raporlar.

---

## Proposed Changes (Dosya Düzeyi Özeti)

### Rust — `tools/clrinf-codegen/`
- `[NEW]` `clrinf-contracts/templates/elixir/otp_worker.tera`
- `[NEW]` `clrinf-contracts/templates/elixir/otp_supervisor.tera`
- `[NEW]` `clrinf-contracts/templates/elixir/otp_handler.tera`
- `[NEW]` `clrinf-contracts/templates/elixir/rule.tera`
- `[MODIFY]` `src/generator.rs`: `scaffold_otp` ve `scaffold_rule_elixir` fonksiyonları
- `[MODIFY]` `src/main.rs`: `clrinf scaffold otp`, `clrinf rule new --lang elixir`, `clrinf docs --lang elixir`
- `[MODIFY]` `src/worker.rs`: `WorkerKind::Elixir`, `resolve_elixir_worker`, `lint()` ve `scaffold_rule()` kolları
- `[MODIFY]` `src/manifest.rs`: `Lang::Elixir`, `mix.exs` auto-detection
- `[MODIFY]` `src/mcp.rs`: `clrinf_scaffold_otp` aracı, `clrinf_inspect_ecosystem` ve `all` listelerine Elixir eklenmesi
- `[MODIFY]` `src/docs_provider.rs`: `ELIXIR_DOCS` mimari kılavuzu

### Elixir — `clrinfex/`
- `[NEW]` `lib/clrinfex/linter/arch_linter.ex` (AST tabanlı `ARCH_EX_001` - `ARCH_EX_004` motoru)
- `[NEW]` `lib/mix/tasks/clrinfex.lint.ex` (`mix clrinfex.lint` task'ı, human ve JSON çıktı formatları)
- `[NEW]` `test/clrinfex/linter/arch_linter_test.exs` (Linter birim testleri)
- `[NEW]` `test/mix/tasks/clrinfex.lint_test.exs` (Mix task testleri)

---

## Doğrulama ve Test Planı

### 1. Otomatik Testler (Birim & Linter)
```bash
# Rust tarafı (generator ve worker testleri)
cargo test -p clrinf-codegen

# Elixir tarafı (linter ve task testleri)
cd clrinfex && mix test test/clrinfex/linter/arch_linter_test.exs
```

### 2. Greenfield Testi (Sıfırdan Proje Doğrulaması)
```bash
# 1. Yeni minimal Elixir projesi üret
target_dir=$(mktemp -d)
clrinf new test_elixir_app --template minimal-elixir

# 2. OTP modülü scaffold et
cd test_elixir_app
clrinf scaffold otp BillingEngine --app test_elixir_app

# 3. Dosyaların derlendiğini ve linter'dan temiz geçtiğini doğrula
mix compile
clrinf lint --lang elixir
```

### 3. Brownfield Testi (Var Olan Projeye Ekleme Doğrulaması)
```bash
# 1. Var olan proje simülasyonu
temp_proj=$(mktemp -d)
cd "$temp_proj" && mix new legacy_service --sup
cd legacy_service

# 2. clrinf ile init et
clrinf init --lang elixir

# 3. Kural ve OTP modülü ekle
clrinf rule new MinimumAmountRule --lang elixir --entity Order
clrinf scaffold otp OrderProcessor --app legacy_service

# 4. Derle ve linter ile kontrol et
mix compile
clrinf lint --lang elixir
```

### 4. MCP Sunucu Doğrulaması
```bash
# MCP üzerinden tool listeleme ve Elixir linter çağrısı
clrinf mcp
# -> clrinf_inspect_ecosystem yanıtında Elixir görülmeli
# -> clrinf_scaffold_otp çalıştırılabilmeli
# -> clrinf_lint_architecture(lang="elixir") JSON formatında sonuç dönmeli
```
