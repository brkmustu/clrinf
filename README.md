# clrinf — Sözleşme Öncelikli, Çok Dilli Dağıtık Uygulama Altyapısı

[![CI](https://github.com/brkmustu/clrinf/actions/workflows/ci.yml/badge.svg)](https://github.com/brkmustu/clrinf/actions/workflows/ci.yml)
[![License: BSL 1.1](https://img.shields.io/badge/License-BSL%201.1-purple.svg)](LICENSE)

**clrinf**, Rust, C#, TypeScript ve Elixir ile **dağıtık sistemler ve modüler monolith mimariler** inşa etmek üzere geliştirilmiş kurumsal bir uygulama çatısıdır. Belirli bir veritabanını, mesaj kuyruğunu veya bulut sağlayıcısını dayatmadan; kanonik veri sözleşmeleri, dile özgü idiomatik iş kuralı motorları, statik mimari denetleyiciler ve otonom yapay zeka ajanları için merkezi bir Meta MCP orkestratörü sunar.

---

## Temel Mimari Prensipleri

1. **Sözleşme Öncelikli (Contract-First) Bütünlük**: Tüm servisler arası iletişim, JSON Schema (Draft 2020-12) ve CloudEvents 1.0 standartları ile güvence altına alınır.
2. **Dile Özgü İdiomatik İş Kuralları**:
   - **C#**: `IBusinessRule<T>` arayüzü, `RulePipeline` ve Roslyn tabanlı mimari denetleyiciler (`ARCH001` - `ARCH004`).
   - **Rust**: `BusinessRule<T>` trait'i, `RulePipeline` ve Syn tabanlı AST linter (`RUST_ARCH001`).
   - **TypeScript**: Saf fonksiyonel `Rule<TCtx>`, `pipeRules` zinciri, `Result<T, E>` monadı ve TS AST linter (`ARCH_TS_001`).
3. **Çapraz Servis Olay Topolojisi ve Pub/Sub Güvencesi**:
   - **Publisher**: %100 mekanik, kanonik CloudEvent zarflama ve atomik `OutboxStore` kuyruklama.
   - **Subscriber**: Otomatik deserialization ve `IdempotencyStore` (`claim`/`complete`/`release`) ile tam mükerrerlik koruması.
   - **Topoloji Doğrulama**: Dağıtık ortamda ölü olayları (`TOPOLOGY_DEAD_EVENT`), yetim aboneleri (`TOPOLOGY_ORPHAN_SUBSCRIBER`) ve şema evrim zincirlerindeki eksiklikleri (`TOPOLOGY_UPCASTER_MISSING`) derleme/CI öncesinde saptar.
4. **Merkezi Orkestratör & Meta MCP Gateway**: Otonom yapay zeka kodlama ajanları (Claude, Cursor, Antigravity) için JSON-RPC 2.0 stdio protokolü üzerinden tüm ekosistemi tek noktadan denetleme, kural üretme ve doğrulama yeteneği.

---

## Kullanım Biçimleri

| Senaryo | Kullanılan Bileşenler | Gereksinim Dışı Unsurlar |
|---|---|---|
| **Kanonik Veri & DTO Üretimi** | JSON Schema + `clrinf-codegen` | Çalışma zamanı servisleri, NATS, Docker |
| **Çapraz Servis Olay Doğrulama** | `clrinf-codegen topology check` + `generate-pubsub` | Taşıma katmanı bağımlılıkları (Kafka/NATS zorunluluğu yoktur) |
| **Modüler Monolith** | İlgili dilin çekirdeği, süreç-içi dispatch, bellek/SQLite adaptörü | Mikroservis bölünmesi, dağıtık kilit, mesaj brokerı |
| **Çok Dilli Dağıtık Sistem** | Kanonik sözleşmeler, Outbox/Inbox adaptörleri, HTTP/mesajlaşma | Tek dil zorunluluğu, tek veri tabanı mecburiyeti |
| **Otonom Yapay Zeka Geliştirme** | `clrinf-codegen mcp` (Meta MCP Sunucusu) | Diller arası parçalı ve uyumsuz AI araç entegrasyonları |

Monolith içerisinde yerel ACID veritabanı işlemleri esastır. Servis sınırları aşıldığında sistem; Outbox, Inbox, Idempotency ve telafi edici işlemleri (compensation) açık ve mekanik garantilerle yönetir. `correlation_id` izleme (telemetri) içindir; tek başına idempotency anahtarı olarak kullanılamaz.

---

## Depo Bileşenleri ve Sorumluluk Alanları

| Dizin | Sorumluluk ve Kapsam |
|---|---|
| [`tools/clrinf-codegen/`](tools/clrinf-codegen/README.md) | Çok dilli kod üretimi, şablonlar (`templates/`), Federated Language Worker orkestrasyonu, Topoloji Doğrulama ve Meta MCP stdio sunucusu |
| [`clrinfcs/`](clrinfcs/README.md) | .NET 10 / C# 13 çekirdeği, C# sözleşmeleri (`contracts/`), Roslyn linterları (`ARCH001-ARCH004`) ve SQLite referans adaptörü |
| [`clrinfrs/`](clrinfrs/README.md) | Rust çekirdeği, Rust sözleşmeleri (`contracts/`), Syn mimari linterı (`RUST_ARCH001`), Axum/CQRS ve bellek adaptörleri |
| [`clrinfjs/`](clrinfjs/README.md) | Bun tabanlı TypeScript çekirdeği, TS sözleşmeleri (`contracts/`), AST linter (`ARCH_TS_001`) ve Event Inspector |
| [`clrinfex/`](clrinfex/README.md) | Elixir çekirdeği, Elixir sözleşmeleri (`contracts/`) ve canlı olay akış (streaming) köprü adaptörleri |
| [`templates/`](templates/) | Manifest tabanlı minimal başlangıç şablonları (`minimal-rust`, `minimal-typescript`, `minimal-csharp`) |
| [`tests/conformance/`](tests/conformance/README.md) | Tüm dillerin paylaştığı kanonik geçerli/geçersiz tel formatı (wire-format) test verileri |

Dört dil çekirdeği ana depoda bağımsız alt modüller (submodule) olarak yönetilir ve her biri kendi sözleşme ve şemalarını (`contracts/`) taşır. `tools/clrinf-codegen`, ekosistemin birleşik orkestrasyon ve kod üretim merkezidir.

---

## Kurulum

`clrinf` CLI ve Meta MCP sunucusu, açık kaynak dağıtım standartlarına uygun olarak GitHub Releases üzerinde önceden derlenmiş ikili (pre-built binary) paketler halinde yayınlanır. Sisteminizde **Rust veya Cargo kurulu olması gerekmez**.

### Hızlı Kurulum (Linux & macOS)

Tek satırlık kurulum betiği ile en güncel sürümü doğrudan kullanıcı dizininize (`~/.local/bin`) kurabilirsiniz:

```bash
curl -fsSL https://raw.githubusercontent.com/brkmustu/clrinf/master/install.sh | bash
```

*(Kaynak koddan derlemek isterseniz: depoyu klonlayıp `./install.sh --build` komutunu çalıştırabilirsiniz.)*

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
Mevcut bir projede veya yeşil alanda sıfır sürtünmeyle modül, entity ve Cedar kuralları yönetin:

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

Geliştirici bu hazır modülleri dilediği projeye (.csproj, Cargo.toml, package.json) **iki farklı felsefede ve sıfır sürtünmeyle** aktarabilir:
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
*(MCP Araçları: `clrinf_list_catalog`, `clrinf_adopt_module`, `clrinf_module_manage`, `clrinf_add_domain_module`, `clrinf_add_entity`, `clrinf_lint_architecture`, `clrinf_scaffold_rule`, `clrinf_generate_pubsub`, `clrinf_validate_topology`)*

> [!TIP]
> **Geliştiriciler İçin Kaynak Koddan Çalıştırma:** İkili dosyayı kurmadan doğrudan yerel kaynak kod üzerinden denemek isterseniz, komutları `cargo run --manifest-path tools/clrinf-codegen/Cargo.toml -- <komut>` şeklinde de yürütebilirsiniz.

---

## Temel Amaç ve Mimari Odak

clrinf, geliştiricilerin yapay zeka desteğiyle çalışırken mimari çerçeveyi korumalarını kolaylaştırmayı hedefler:

- **Token Tasarrufu ve Bağlam Verimliliği:** Tüm kod tabanını bütünüyle modele yüklemek yerine; modüler manifestler (`clrinf.toml`), odaklı CLI araçları ve Graft bağlam grafiği sayesinde yapay zekaya yalnızca ihtiyaç duyduğu dar ve isabetli bağlamın verilmesini kolaylaştırarak token tüketimini düşürmeye yardımcı olur.
- **Yapay Zeka Destekli Geliştirmede Net Mimari Çerçeveler:** Kodlama ajanlarının mimari sapmalara yönelmesini sınırlandırmak adına deterministik linter'lar (Roslyn, Syn AST, TS Compiler API), tip-güvenli kural motorları ve açık sözleşmeler ile geliştirmelerin belirlenen mimari sınırlar dahilinde kalmasını destekleyen denetim mekanizmaları sunar.
- **Düşük Parametreli Modellerle Çalışırken Araç Desteği:** Yalnızca devasa frontier modellere bağımlı kalmadan; küçük ve yerel dil modelleriyle (SLM / Local LLM) çalışırken de mimari kaliteyi, tasarım disiplinini ve kod tutarlılığını yüksek tutabilmek için geliştiricinin elini güçlendiren araçlar ve kurallar sağlar.

---

## Lisans

Bu proje [BSL 1.1](LICENSE) lisansı altındadır. Lisans dönüşüm tarihi (Change Date) **2030-09-08** olarak belirlenmiş olup, bu tarihte otomatik olarak Apache-2.0 lisansına dönüşecektir.
