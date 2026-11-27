# clrinf — AI-Native Ekipler İçin Mimari & Olay Sözleşmesi Koruyucusu

[![CI](https://github.com/brkmustu/clrinf/actions/workflows/ci.yml/badge.svg)](https://github.com/brkmustu/clrinf/actions/workflows/ci.yml)
[![License: Apache-2.0](https://img.shields.io/badge/License-Apache_2.0-blue.svg)](LICENSE)
[English README](README.md)

**clrinf**, yapay zeka kodlama ajanlarının (Claude Code, Cursor, Antigravity) ve dağıtık ekiplerin **olay sözleşmelerini kırmasını, beyan edilen topolojiden sapmasını (drift) ve domain katmanına altyapı bağımlılığı sızdırmasını deterministik olarak engelleyen** tek binary'lik bir Rust CLI ve Meta MCP sunucusudur — düşük parametreli veya yerel modellerde (SLM) dahi mimari tutarlılığı korur.

Varsayılan olarak **Modüler Monolith** ve yerel ACID işlemlerle başlarsınız; mevcut kodunuza **5 dakikada, sıfır çalışma zamanı kilitlenmesiyle (zero lock-in)** eklenir ve yalnızca ihtiyaç duyduğunuzda çok servisli olay topolojisine ölçeklenir.

---

## Neden clrinf?

`AGENTS.md` veya `CLAUDE.md` içindeki metin kuralları uzun oturumlarda unutulur:
- Ajanlar servisler arası olay şemalarını sessizce kırar veya `x-published-by` / `x-subscribed-by` beyanıyla çelişir.
- Domain katmanına `Infrastructure` import'ları sızdırılır.
- Üretilen publisher atlanıp doğrudan outbox yazımı yapılır.
- Ajanlar basit bir CRUD veya tek bir olay değişikliği için tüm repo boyunca onlarca gereksiz dosyayı okuyup token tüketir.

`clrinf`, aynı kural kaynağından (`clrinf.rules.toml`) ajan talimatlarını, **post-edit hook'larını**, dar bağlam planlarını, derleme anı CRUD üretimini ve **CI `verify` kapılarını** çalıştırır.

---

## 5 Dakikada Kurulum (Mevcut Repoda Sıfır Kilitlenme)

**Linux & macOS:**
```bash
curl -fsSL https://raw.githubusercontent.com/brkmustu/clrinf/master/install.sh | bash
```

**Windows (PowerShell):**
```powershell
irm https://raw.githubusercontent.com/brkmustu/clrinf/master/install.ps1 | iex
```

**Projeye ekleme:**
```bash
clrinf agents sync                 # AGENTS.md / CLAUDE.md / GEMINI.md / Cursor kurallarını üret
clrinf hook install --agent all    # Claude Code + Cursor + Antigravity + git pre-commit
clrinf verify                      # Topoloji + kod<->sözleşme drift + import kuralları + outbox
```

---

## Temel Yetenekler

### 1. Deterministik AI Ajan Korumaları (`clrinf.rules.toml`)
- **Dil-farkında `[[forbid]]` kuralları:** `scope = "imports"` ile yorum satırları veya string sabitleri yerine yalnızca gerçek `use` / `using` / `import` / `alias` deyimlerini denetler (C#, TypeScript, Rust, Elixir).
- **Düzenleme sonrası anlık geri bildirim (`clrinf hook install`):** Ajan dosya düzenlediğinde yalnızca etkilenen kontroller çalışır; ihlal ajana geri bildirilir ve kendi kendini düzeltir.
- **Boşluk doldurma planı (`clrinf plan`):** Mekanik şema/kayıt/kabuk işlerini yapar, ajana yalnızca doldurulacak sembolleri ve okunacak dar dosya dilimini verir.
- **Dar MCP yüzeyi (`clrinf mcp --profile lean`):** Oturum başına sabit araç şeması maliyetini düşürür (`lean`: `clrinf_crud_map` ve `clrinf_crud_eject` dahil 10 araç, `events`: 14 araç, `full`: 27 araç).
- **Ayrıntılar:** [Ajan Korumaları](docs/guides/agent-guardrails.md), [Benchmark Protokolü](docs/benchmarks/README.md).

### 2. Sıfır Şablon Maliyetli Derleme Anı Native AOT CRUD (.NET 10)
Diskte yalnızca entity ve yaşam döngüsü kuralları tutularak ajan bağlamındaki dosya karmaşasını azaltır:
- Entity'lere `[GenerateCrudEndpoints]` eklenir (`ClrinfCS.Generators` derleme anında DTO'ları, operasyon başına handler ve Minimal API `RequestDelegate` endpoint'lerini, sıfır-yansımalı `Utf8JsonWriter`/`JsonDocument` serileştirmesini ve AOT uyumlu `ClrinfCS.OpenApi` bilgisini üretir; `PublishAot=true` altında **0 uyarı / 0 hata**).
- Üretilen handler'lara dokunmadan iş kuralı eklenir: `clrinf rule new ActiveProductsCheck --lang csharp --entity Category --on before-delete` (`IBefore{Create|Update|Delete}Rule<T>` + `CrudRuleRunner`).
- Tek operasyonu Smart Override ile inceleme/çıkarma: `clrinf crud-map --json` ve `clrinf eject Category --operation Update`.
- Roslyn `ARCH001–ARCH005` denetimleri, `ClrinfCS.Adapters.EfCore` (`EfCrudStore`) ve `ClrinfCS.Adapters.Sqlite` ile desteklenir. Bkz. [`core/csharp/examples/AotCrud/`](core/csharp/examples/AotCrud/README.md).

### 3. Çok Dilli Olay Sözleşmesi & Topoloji Yönetişimi (İsteğe Bağlı Ölçeklenme)
- **Kod ↔ Sözleşme Drift Tespiti (`topology drift`):** Şemadaki `x-published-by` / `x-subscribed-by` beyanlarını gerçek kaynak kodu ile karşılaştırır ve doğrudan outbox yazımlarını (`DRIFT_RAW_OUTBOX_WRITE`) yakalar.
- **Kırıcı Değişiklik & Upcaster Analizi (`topology check` / `report`):** Ölü olayları (`TOPOLOGY_DEAD_EVENT`), yetim aboneleri (`TOPOLOGY_ORPHAN_SUBSCRIBER`) ve eksik upcaster'ları (`TOPOLOGY_UPCASTER_MISSING`) bulur; PR'lar için `--fail-on-breaking` raporu üretir.
- **Dar Bağlam Etki Analizi (`topology impact`):** Bir olayı değiştirmeden önce yalnızca etkilenen servisleri, sürümleri ve ilgili el yazımı dosyaları listeler.
- **Kod Üretimi (`generate`, `generate-pubsub`, `generate-upcasters`, `event subscribe|publish`):** JSON Schema (Draft 2020-12) ve CloudEvents 1.0 uyumlu DTO'lar, publisher'lar ve idempotent subscriber kabukları üretir.

### 4. Sıfır Bağımlılıklı Modül İskeletleri (`clrinf catalog` / `clrinf module adopt`)
Yerleşik modülleri (CRM Deals, Contacts, Activities, CRM Suite, Authz/Cedar, Caching, Outbox, Idempotency) mevcut `.csproj`, `Cargo.toml` veya `package.json` projelerine **iki felsefede** aktarabilirsiniz:
- `--mode raw`: Sıfır harici paket bağımlılığıyla saf kaynak kod.
- `--mode wired`: Hedef dilin DI / modül ağacına ve çok kiracılı Amazon Cedar politikalarına bağlı olarak.

### 5. İsteğe Bağlı Referans Çekirdekler & Dil Linter'ları
`core/` dizini zorunlu runtime değildir; dağıtık sistem ve modüler monolith referansları sunar:
- **Tier-1:** [`core/rust/`](core/rust/README.md) (Syn `RUST_ARCH001`), [`core/csharp/`](core/csharp/README.md) (.NET 10, Roslyn `ARCH001–ARCH005`, `ClrinfCS.Generators` AOT CRUD, `EfCrudStore` / SQLite), [`core/typescript/`](core/typescript/README.md) (Bun, TS AST `ARCH_TS_001`, Event Inspector).
- **Deneysel / Akış Köprüsü:** [`core/elixir/`](core/elixir/README.md) (OTP çekirdek, AST `ARCH_EX_001-004`, NATS canlı akış köprüsü).

---

## Depo Yapısı

| Dizin | Kapsam |
|---|---|
| [`tools/clrinf-codegen/`](tools/clrinf-codegen/README.md) | Tek-binary Rust CLI, AST/import analizörü, topoloji & drift motoru, modül adaptörü ve Meta MCP sunucusu |
| [`core/`](docs/architecture/core-and-adapters.md) | İsteğe bağlı Rust, C#, TypeScript ve Elixir referans çekirdekleri, AOT CRUD üreteci ve adaptörler |
| [`templates/`](templates/) | Minimal başlangıç şablonları ve `eticaret-showcase` referans monoliti |
| [`examples/guardrails-benchmark/`](examples/guardrails-benchmark/) | Tekrarlanabilir çok-dilli guardrail ve bağlam maliyeti benchmark örneği |
| [`tests/conformance/`](tests/conformance/README.md) | Tüm dillerin paylaştığı kanonik wire-format uyumluluk test verileri |

---

## Lisans

Bu proje [Apache License 2.0](LICENSE) altında lisanslanmıştır.
