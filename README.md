# clrinf — Yapay Zeka Ajanları için Mimari Koruma, Token Verimliliği ve Kod Üretim Altyapısı

[![CI](https://github.com/brkmustu/clrinf/actions/workflows/ci.yml/badge.svg)](https://github.com/brkmustu/clrinf/actions/workflows/ci.yml)
[![License: BSL 1.1](https://img.shields.io/badge/License-BSL%201.1-purple.svg)](LICENSE)

> **Modüler Monolith'ten Çok Dilli Dağıtık Sistemlere**: Yapay zeka ajanlarını mimari sınırlar içinde kalmaya zorlayan, hazır şablonlarla token tüketimini düşürmeye yardımcı olan ve karmaşıklaşan projelerde kontrolü korumaya destek sunan açık kaynaklı geliştirme altyapısı.

**clrinf**, yapay zeka kodlama ajanlarıyla (Claude Code, Cursor, Antigravity, Copilot vb.) yazılım geliştirirken; **mimari anlamda daha stabil kod üretimine katkı sunmak**, **hazır şablonlarla token tüketimini düşürmeye yardımcı olmak** ve **daha düşük parametreli dil modellerinde (SLM/Local LLM) bile ajanları belirli mimari sınırlar dahilinde tutabilmek** amacıyla geliştirilmektedir.

Geliştiriciye başlangıçta peşinen bir mikroservis ya da dağıtık sistem mimarisi **dayatmaz**. İster tek bir süreçte (in-process) çalışan yalın bir **modüler monolith** ile başlayın, ister projeniz zamanla **çok dilli (Rust, C#, TypeScript, Elixir) dağıtık sistemlere** evrilsin; `clrinf`, yapay zeka ajanlarını önceden tanımlanmış mimari kurallar ve sözleşmeler içinde kalmaya zorlayarak, özellikle karmaşıklaşan projelerde kontrolü kaybetmeden geliştirme yapabilmenize destek olmayı hedefler.

---

## Neden clrinf?

Yapay zeka ajanları hızlı kod üretebilir; ancak proje büyüdükçe mimari sınırları ihlal etme, katmanları birbirine karıştırma ve kontrolsüz token tüketme eğilimindedir. Üstelik bu durum, yüksek maliyetli "frontier" modeller yerine daha ekonomik veya yerel modeller kullanıldığında çok daha belirgin hale gelir. `clrinf`, bu zorlukları hafifletmek ve geliştiriciye yardımcı olmak için üç temel hedefe odaklanır:

1. **Daha Düşük Modellerde Bile Mimari Sınırları Koruma Desteği**
   - Yapay zeka modelleri büyüdükçe yetenekleri artsa da mimari katmanları ihlal etme riski devam eder. `clrinf` bir kalite garantisi sunmaz; ancak ajanı deterministik kurallarla sınırlandırarak, daha düşük parametreli veya yerel modellerde (SLM / Local LLM) bile mimari açıdan daha tutarlı kod üretilmesine katkı sağlamaya çalışır.
   - Roslyn linter'ları (`ARCH001-ARCH004`), Syn AST denetleyicileri (`RUST_ARCH001`) ve TypeScript AST linter'ları (`ARCH_TS_001`) ile mimari sınırları derleme seviyesinde denetler.
   - Ajan mimari bir kuralı atladığında IDE ve Git hook'ları (`clrinf hook install`) geri bildirim üreterek ajanın belirlenen sınırlar içinde kalmasını hatırlatır; mimari sapmaların erkenden fark edilmesine yardımcı olur.

2. **Hazır Kod Üretim Şablonlarıyla Token Tüketimini Düşürme**
   - Ajanların sıfırdan mimari iskelet veya DTO kurgulamaya çalışırken yüksek miktarda token tüketmesinin önüne geçmeye yardımcı olur.
   - CLI iskeletleyicileri (`clrinf rule new`, `clrinf add module/entity`), yerleşik modül kataloğu (`clrinf catalog` / `clrinf module adopt`) ve boşluk doldurma planları (`clrinf plan`) ile mekanik işlerin şablonlar üzerinden ilerlemesini sağlar.
   - Ajanın tüm projeyi baştan sona okuması yerine; `clrinf plan` ve Graft bağlam grafiği ile yalnızca doldurması gereken iş mantığına ve dar dosya bağlamına odaklanması hedeflenir. `--profile lean` Meta MCP sunucusu ile oturum başına araç şeması token maliyeti düşürülür.

3. **Karmaşık ve Dağıtık Projelerde Kontrolü Elde Tutma**
   - Dağıtık sistem bir **önkoşul veya ilk adım değil**, proje karmaşıklaştığında devreye giren bir **ölçeklenme seçeneğidir**.
   - Proje büyüyüp çok servisli veya dağıtık bir yapıya evrildiğinde; sözleşme öncelikli yapı (JSON Schema, CloudEvents), drift tespiti (`clrinf topology drift`), etki analizi (`clrinf topology impact`) ve Outbox/Idempotency şablonları sayesinde, yapay zeka ajanları ile çalışırken mimari kontrolün kaybedilmemesine destek olur.

---

## Temel Mimari Prensipleri

1. **Ajan Korumaları ve Dar Bağlam (Agent Guardrails & Context Efficiency)**: Tek kural dosyasından (`clrinf.rules.toml`) beslenen otomatik koruma. `clrinf hook install` (Claude Code, Cursor, Antigravity, git pre-commit) ajanın her dosya düzenlemesinden sonra kuralları denetler; `clrinf plan` mekanik işleri yapıp ajana yalnızca doldurulacak iş mantığını ve dar dosya bağlamını bırakır; `clrinf agents sync` aynı kuralları `AGENTS.md` / `CLAUDE.md` / Cursor kurallarına yazar. Ayrıntılar: [Ajan Korumaları](docs/guides/agent-guardrails.md), [Benchmark Protokolü](docs/benchmarks/README.md).
2. **Dile Özgü İdiomatik İş Kuralları & Derleyici Seviyesinde Denetim**:
   - **C#**: `IBusinessRule<T>` arayüzü, `RulePipeline` ve Roslyn tabanlı mimari denetleyiciler (`ARCH001` - `ARCH004`).
   - **Rust**: `BusinessRule<T>` trait'i, `RulePipeline` ve Syn tabanlı AST linter (`RUST_ARCH001`).
   - **TypeScript**: Saf fonksiyonel `Rule<TCtx>`, `pipeRules` zinciri, `Result<T, E>` monadı ve TS AST linter (`ARCH_TS_001`).
3. **Sözleşme Öncelikli (Contract-First) Bütünlük**: Tüm veri modelleri ve servisler arası iletişim, dilden bağımsız kanonik JSON Schema (Draft 2020-12) ve CloudEvents 1.0 standartları üzerinden yapılandırılır; modeller otomatik üretilir.
4. **Karmaşık ve Dağıtık Sistemlerde Olay Topolojisi Koruması**: Sistem büyüdüğünde derleyicinin göremediği servisler arası bağımlılıkları koruma altına alır:
   - **Drift Tespiti**: `topology drift` ile şemadaki `x-published-by` / `x-subscribed-by` beyanları gerçek servis kodu ile karşılaştırılır; beyan edilmemiş publish/handle ve doğrudan outbox yazımları (`DRIFT_*`) yakalanır.
   - **Şema Evrimi Analizi**: Sürümler arası alan farkları kırıcı / kırıcı olmayan olarak sınıflandırılır; eksik upcaster (`TOPOLOGY_UPCASTER_MISSING`) tanısı neyin kırıldığını bildirir.
   - **Etki Analizi**: `topology impact` bir olayı değiştirmeden önce etkilenen servisleri, sürümleri ve yalnızca ilgili kaynak dosyalarını listeler (ajana tüm repo yerine dar bağlam verir).
   - **Olay Öncelikli İskelet**: `event subscribe|publish` şemayı günceller, CloudEvent publisher ve idempotent subscriber kabuklarını üretir. Kabuk, `IdempotencyStore` (`claim`/`complete`/`release`) ile **etkin-tek-sefer (effectively-once)** işleme sağlar.
   - **PR Raporu & Topoloji Doğrulama**: `topology report` PR'da topoloji farklarını özetler; ölü olayları (`TOPOLOGY_DEAD_EVENT`) ve yetim aboneleri (`TOPOLOGY_ORPHAN_SUBSCRIBER`) derleme/CI öncesinde saptar.
5. **Merkezi Orkestratör & Meta MCP Gateway**: Otonom yapay zeka kodlama ajanları (Claude, Cursor, Antigravity) için JSON-RPC 2.0 stdio protokolü üzerinden ekosistem bileşenlerini merkezi araçlarla denetleme, kural üretme ve doğrulama yeteneği. `--profile lean` ile oturum başına sabit araç şeması token maliyeti düşürülür.

---

## Kullanım Biçimleri

| Senaryo | Kullanılan Bileşenler | Gereksinim Dışı Unsurlar |
|---|---|---|
| **Modüler Monolith (Önerilen Başlangıç)** | İlgili dilin çekirdeği, süreç-içi dispatch, bellek/SQLite adaptörü, izole iş kuralları | Mikroservis bölünmesi, dağıtık kilit, mesaj brokerı, Docker |
| **Yapay Zeka ile Güvenli Geliştirme** | `clrinf hook`, `clrinf plan`, `clrinf catalog/adopt`, Meta MCP Gateway | Token israfı, mimari sapmalar, devasa dosya bağlamı yükleme |
| **Kanonik Veri & DTO Üretimi** | JSON Schema + `clrinf generate` | Çalışma zamanı servisleri, NATS, Docker |
| **Çok Dilli Dağıtık Sistem (İhtiyaç Halinde Ölçeklenme)** | Kanonik sözleşmeler, Outbox/Inbox adaptörleri, HTTP/mesajlaşma, topoloji denetimi | Tek dil mecburiyeti, tek veri tabanı zorunluluğu, kontrolsüz drift |
| **Çapraz Servis Olay Doğrulama** | `clrinf topology check` + `generate-pubsub` | Taşıma katmanı bağımlılıkları (Kafka/NATS zorunluluğu yoktur) |

> [!NOTE]
> **Mimari Esneklik ve Ölçeklenme:** Varsayılan olarak en yalın başlangıç olan **Modüler Monolith** yaklaşımı ve yerel ACID veritabanı işlemleri desteklenir. Dağıtık sistemler veya mikroservisler bir zorunluluk değil; projeniz büyüyüp servis sınırları aşıldığında Outbox, Inbox, Idempotency ve telafi edici işlemler (compensation) gibi açık kalıplarla güvenle devreye alınabilecek bir yetenektir. `correlation_id` telemetri izleme içindir; tek başına idempotency anahtarı olarak kullanılamaz.

---

## Depo Bileşenleri ve Sorumluluk Alanları

| Dizin | Sorumluluk ve Kapsam |
|---|---|
| [`tools/clrinf-codegen/`](tools/clrinf-codegen/README.md) | Çok dilli kod üretimi, şablonlar (`templates/`), Federated Language Worker orkestrasyonu, Topoloji Doğrulama ve Meta MCP stdio sunucusu |
| [`core/csharp/`](core/csharp/README.md) | .NET 10 / C# 13 çekirdeği, C# sözleşmeleri (`contracts/`), Roslyn linterları (`ARCH001-ARCH004`) ve SQLite referans adaptörü |
| [`core/rust/`](core/rust/README.md) | Rust çekirdeği, Rust sözleşmeleri (`contracts/`), Syn mimari linterı (`RUST_ARCH001`), Axum/CQRS ve bellek adaptörleri |
| [`core/typescript/`](core/typescript/README.md) | Bun tabanlı TypeScript çekirdeği, TS sözleşmeleri (`contracts/`), AST linter (`ARCH_TS_001`) ve Event Inspector |
| [`core/elixir/`](core/elixir/README.md) | Elixir çekirdeği, Elixir sözleşmeleri (`contracts/`) ve canlı olay akış (streaming) köprü adaptörleri |
| [`templates/`](templates/) | Manifest tabanlı minimal başlangıç şablonları (`minimal-rust`, `minimal-typescript`, `minimal-csharp`) |
| [`tests/conformance/`](tests/conformance/README.md) | Tüm dillerin paylaştığı kanonik geçerli/geçersiz tel formatı (wire-format) test verileri |

Tüm dil çekirdekleri (`core/`) bu monorepo içinde yönetilir ve her biri kendi sözleşme ve şemalarını (`contracts/`) taşır. `tools/clrinf-codegen`, ekosistemin birleşik orkestrasyon ve kod üretim merkezidir.

---

## Kurulum

`clrinf` CLI ve Meta MCP sunucusu, açık kaynak dağıtım standartlarına uygun olarak GitHub Releases üzerinde önceden derlenmiş ikili (pre-built binary) paketler halinde yayınlanır. Sisteminizde **Rust veya Cargo kurulu olması gerekmez**.

### Hızlı Kurulum (Linux & macOS)

Tek satırlık kurulum betiği ile en güncel sürümü doğrudan kullanıcı dizininize (`~/.local/bin`) kurabilirsiniz:

```bash
curl -fsSL https://raw.githubusercontent.com/brkmustu/clrinf/master/install.sh | bash
```

*(Kaynak koddan derlemek isterseniz: depoyu klonlayıp `./install.sh --build` komutunu çalıştırabilirsiniz.)*

### Hızlı Kurulum (Windows - PowerShell)

PowerShell üzerinden en güncel sürümü doğrudan kullanıcı dizininize (`%LOCALAPPDATA%\clrinf\bin`) kurabilirsiniz:

```powershell
irm https://raw.githubusercontent.com/brkmustu/clrinf/master/install.ps1 | iex
```

*(Kaynak koddan derlemek isterseniz: depoyu klonlayıp `.\install.ps1 -Build` komutunu çalıştırabilirsiniz.)*

---

## Hızlı Başlangıç

`clrinf` kurulduktan sonra tüm mimari denetim, kod üretimi ve modül yönetim komutları doğrudan terminalden çalıştırılabilir:

### 1. Sözleşme ve Model Üretimi
Çalışma zamanı bağımlılığı olmaksızın modelleri üretin:

```bash
# Şemaları doğrula
clrinf check --schema-dir tools/clrinf-codegen/schemas

# Çok dilli (Rust, C#, TypeScript) sözleşme modellerini üret
clrinf generate \
  --schema-dir tools/clrinf-codegen/schemas \
  --templates-dir tools/clrinf-codegen/templates \
  --output ./generated
```

### 2. Çapraz Servis Olay Topolojisini Denetleme & Kod Üretimi
Dağıtık olay topolojisindeki ölü olayları, yetim tüketicileri ve versiyon geçişlerini doğrulayın:

```bash
# Topoloji grafiğini ve tanı raporunu çıkart
clrinf topology check \
  --schema-dir tools/clrinf-codegen/schemas \
  --upcasters-dir tools/clrinf-codegen/schemas

# Güvenli CloudEvent Publisher ve Idempotent Subscriber kabuklarını üret
clrinf generate-pubsub \
  --schema-dir tools/clrinf-codegen/schemas \
  --templates-dir tools/clrinf-codegen/templates \
  --lang all \
  --output ./generated-pubsub
```

**Kod ↔ sözleşme drift'i, etki analizi, PR raporu ve olay öncelikli iskelet:**

```bash
# Beyan edilen topoloji gerçek kodla uyuşuyor mu? (CI'da hata ile çıkar)
clrinf topology drift \
  --schema-dir tools/clrinf-codegen/schemas \
  --service billing=./services/billing --service orders=./services/orders

# Bir olayı değiştirmeden önce: kim etkilenir, hangi dosyalar?
clrinf topology impact orders.placed.v1 --src ./services

# PR için topoloji değişiklik özeti (taban: git ref veya dizin)
clrinf topology report --base-ref origin/main --fail-on-breaking

# Servisi aboneliğe kaydet + kabukları üret; geriye yalnızca handler gövdesi kalır
clrinf event subscribe orders.placed.v1 --service billing
```

### 3. Federated Language Workers & Mimari Denetim
Kurulu sistem araçlarını (`dotnet`, `cargo`, `bun`) denetleyin ve mimari kuralları çalıştırın:

```bash
# Dil işçilerinin (workers) durumunu görüntüle
clrinf worker check

# Tüm dillerde statik mimari linterları çalıştır (Roslyn, Syn, TS AST)
clrinf lint --lang all

# İzole ve test edilebilir yeni bir iş kuralı iskeleti oluştur
clrinf rule new CheckMaxDiscount --lang all --entity Order
```

### 4. Modüler Uygulama Yaşam Döngüsü & Çok Kiracılı Cedar Yetkilendirme
Mevcut bir projede veya yeşil alanda modül, entity ve Cedar kuralları yönetin:

```bash
# Projeyi profil ve dispatcher seçimiyle başlat
# Profiller: minimal (saf alan mantığı), standard (log + tx), full (tüm 7 modül)
# Dispatcher: native (FrozenDictionary), mediatr, mediatornet
clrinf init \
  --profile standard \
  --dispatcher native

# Proje ve aktif modül durumunu sorgula
clrinf status

# Caching/logging gibi cross-cutting modülleri ekle, kaldır, senkronize et
clrinf module add caching --provider memory
clrinf module remove caching
clrinf module sync

# Domain modülü (handlers, models, kurallar ve çok kiracılı Cedar policy) ekle
clrinf add module Orders

# İlgili modüle CRUD entity'si, repository portu ve tenant koruması ekle
clrinf add entity OrderItem -m Orders --prop name:string --prop price:f64
```

### 5. Modül Kataloğu ve Adaptasyon Motoru (`clrinf catalog` / `clrinf module adopt`)
`clrinf`, geliştiricilerin sıfırdan yazmak zorunda kalmaması için **yerleşik modüller** (CRM Deals, Contacts, Activities, birleşik CRM Suite) ve **altyapı modülleri** (Cedar Authz, Caching, Logging, Transaction, Idempotency, Outbox) sunar.

Geliştirici bu hazır modülleri dilediği projeye (.csproj, Cargo.toml, package.json) **iki farklı felsefede** aktarabilir:
- **Ham (raw / zero-dependency)**: Sıfır harici paket veya kütüphane bağımlılığıyla, tamamen saf ve izole modeller, komutlar, repository portları ve Cedar güvenlik kurallarıyla yerleştirilir.
- **Kablolanmış (wired / batteries-included)**: `clrinf` ekosisteminin tüm kabiliyetlerini barındıracak şekilde; hedef dilin doğal DI ve modül yapısına (C# `IServiceCollection`, Rust `pub mod`, TS barrel export) ve Cedar yetkilendirme hattına bağlanarak aktarılır.

```bash
# 1. Yerleşik modül kataloğunu listele
clrinf catalog

# 2. C# projesine yerleşik tam teşekküllü CRM Paketini (Deals+Contacts+Activities) ham (raw) olarak aktar
clrinf module adopt crm \
  --to-project ./apps/BillingService/BillingService.csproj \
  --mode raw

# 3. Rust crate'ine CRM Paketini modül ağacına bağlı (wired) ve Cedar politikasıyla aktar
clrinf module adopt crm \
  --to-project ./crates/crm-service/Cargo.toml \
  --mode wired

# 4. Tekil CRM Deals modülünü 'Sales' takma adıyla ham olarak aktar
clrinf module adopt deals \
  --to-project ./apps/BillingService/BillingService.csproj \
  --target-dir src/Features/Sales \
  --as Sales \
  --mode raw

# 5. Rust crate'ine Cedar yetkilendirme altyapısını ham haliyle aktar
clrinf module adopt authz \
  --to-project ./crates/identity-worker/Cargo.toml \
  --mode raw
```

> Ayrıntılı mimari tasarım, diller arası idiomatik bağlama kuralları ve Cedar çok kiracılı izolasyon detayları için: [Modüler Uygulama ve Cedar Kılavuzu](docs/architecture/modular-application-and-cedar-guide.md).

### 6. Otonom Yapay Zeka Geliştiricileri İçin Meta MCP Sunucusu
Model Context Protocol (JSON-RPC 2.0 stdio) sunucusunu başlatın:

```bash
clrinf mcp
```
*(MCP Araçları: `clrinf_list_catalog`, `clrinf_adopt_module`, `clrinf_module_manage`, `clrinf_add_domain_module`, `clrinf_add_entity`, `clrinf_lint_architecture`, `clrinf_scaffold_rule`, `clrinf_generate_pubsub`, `clrinf_validate_topology`, `clrinf_topology_drift`, `clrinf_event_impact`, `clrinf_topology_report`, `clrinf_event_register`, `clrinf_plan_change`, `clrinf_verify`. Profiller: `full`, `events`, `lean` — `clrinf mcp --profile lean`)*

> [!TIP]
> **Geliştiriciler İçin Kaynak Koddan Çalıştırma:** İkili dosyayı kurmadan doğrudan yerel kaynak kod üzerinden denemek isterseniz, komutları `cargo run --manifest-path tools/clrinf-codegen/Cargo.toml -- <komut>` şeklinde de yürütebilirsiniz.


---

## Lisans

Bu proje [BSL 1.1](LICENSE) lisansı altındadır. Lisans dönüşüm tarihi (Change Date) **2030-09-08** olarak belirlenmiş olup, bu tarihte otomatik olarak Apache-2.0 lisansına dönüşecektir.
