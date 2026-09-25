# clrinf Modüler Uygulama ve Çok Kiracılı Cedar Yetkilendirme Kılavuzu

Bu doküman, `clrinf`'in **tam modüler uygulama üretim motorunu**, **dile özgü idiomatik yaşam döngüsü yönetimini** ve **tüm dillerde varsayılan çok kiracılı (multi-tenant by default) Cedar policy tabanlı yetkilendirme mimarisini** açıklar.

---

## 1. Temel Mimari Felsefe ve Dil Doğallığı (Language Idiom Fidelity)

`clrinf`, diller arası ortak sözleşmeleri ve iş kurallarını birleştirirken, **her dilin kendi doğal tasarım kalıplarına ve ekosistem standartlarına %100 sadık kalır**. Diller arasında yapay soyutlamalar veya yabancı kalıplar (anti-pattern) dayatılmaz:

| Dil | Modül Bağlama & Yaşam Döngüsü | Yetkilendirme & Middleware | Tip ve Hata Güvencesi |
|---|---|---|---|
| **C# (.NET 10)** | `IServiceCollection` IoC / DI konteyneri, MediatR Pipeline Behaviors | `[CedarAuthorize]` attribute'ları ve `AuthorizationBehavior` | Güçlü tipler, Records, Nullable Reference Types |
| **Rust** | **Derleme Zamanı Modül Ağacı** (`pub mod ...;` in `lib.rs`/`main.rs`), Açık Struct ve Trait Kompozisyonu (Runtime DI konteyneri YOKTUR) | `MultiTenantCedarAuthorizer` ve Axum / Handler katmanları | `Result<T, ErrorEnvelope>`, Derleme zamanı sahiplik (ownership) |
| **TypeScript (Bun / Node)** | **ESM Barrel Export'ları** (`export * as ... from ...`), Fonksiyonel middleware hatları | `MultiTenantCedarAuthorizer` ve saf fonksiyonel `pipeRules` | Zod / Tip sözleşmeleri, `Result<T, E>` monadı |

> [!IMPORTANT]
> **Rust Dilinde DI Konteyneri Anti-Pattern'i**: Rust'ta nesne yönelimli dillerdeki çalışma zamanı DI konteynerları kullanılmaz. Rust'ın doğası, bağımlılıkların derleme zamanında modül hiyerarşisi (`pub mod ...;`) ve açık struct başlatıcıları (`new(...)`) ile bağlanmasıdır. `clrinf`, Rust projelerinde kod üretirken runtime DI eklemez; modül ağacını yönetir.

---

## 2. Tekil Proje Manifesti (`clrinf.toml`)

Her proje, kök dizinindeki `clrinf.toml` dosyası ile konfigüre edilir. `clrinf init` komutu bu dosyayı brownfield olarak otomatik oluşturabilir:

```toml
# clrinf Project Manifest
[project]
name = "StoreApp"
lang = "rust"             # "rust" | "typescript" | "csharp"
arch = "clean-cqrs"       # "clean-cqrs" | "layered" | "flat"
deployment = "monolith"   # "monolith" | "distributed"
host_type = "api"         # "api" | "worker" | "cli"
dispatcher = "native"     # "native" (FrozenDictionary) | "mediatr" | "mediatornet"

[modules.caching]
enabled = true
provider = "memory"       # "memory" | "redis" | "custom"
default_ttl_seconds = 300

[modules.logging]
enabled = true
provider = "structured"   # "structured" | "console" | "custom"

[modules.transaction]
enabled = true
provider = "native"       # "native" | "custom"

[modules.authentication]
enabled = true
provider = "jwt"          # "jwt" | "custom"

[modules.authorization]
enabled = true
provider = "cedar"        # "cedar" | "custom"

[modules.error_handling]
enabled = true             # Kanonik hata zarfı

[modules.validation]
enabled = true             # Girdi doğrulama

[modules.idempotency]
enabled = true
provider = "memory"       # "memory" | "redis" | "custom"

[modules.outbox]
enabled = true
provider = "database"     # "database" | "outbox_table" | "custom"
```

---

## 2.1. Geliştirici Profilleri: "En Saf Halden" "En Dolu Hale" (Zero-Friction Modülerlik)

Geliştiricilerin mimari tercihlerine tam özgürlük sağlamak amacıyla `clrinf init` komutu 3 farklı profil sunar:

| Profil | Açıklama | Aktif Modüller | Kullanım Amacı |
|---|---|---|---|
| `--profile minimal` | **En Saf Hali (Lean / Pure)** | Sıfır cross-cutting concern (`enabled = false`) | Sıfır yük, saf iş mantığı, mikro mimariler veya kendi altyapısını tamamen kendisi kurmak isteyen geliştiriciler. Üretilen domain kodları sıfır harici paket bağımlılığıyla çalışır. |
| `--profile standard` | **Dengeli Standart** | `logging`, `transaction`, `error_handling`, `validation` | Standart iş uygulamaları için dengeli başlangıç. |
| `--profile full` | **En Dolu Hali (Batteries-Included)** | 7 modülün tamamı (`caching`, `logging`, `transaction`, `authentication`, `authorization` Cedar ile, `idempotency`, `outbox`) | Kurumsal (Enterprise) seviyede tüm cross-cutting concern'lerin ve Cedar çok kiracılı yetkilendirmenin hazır kablolanmış olduğu tam donanımlı modüler monolitler. |

### 2.2. Kendi Altyapını Getir (BYO - Custom Provider Desteği)

Geliştirici standart hazır şablonlar yerine kendi caching, logging veya yetkilendirme mekanizmasını kullanmak istediğinde:
- İlgili modül `provider = "custom"` olarak tanımlanır (örn: `clrinf module add caching --provider custom`).
- **Dokunulmazlık Garantisi**: `clrinf`, `custom` sağlayıcılı modüllere hazır şablon enjekte etmez, `clrinf module sync` çalıştırıldığında kullanıcının yazdığı dosyaları ezmez ve `clrinf module remove` yapıldığında kullanıcının kodunu **asla silmez** (yalnızca manifest durumunu günceller).
- Üretilen Domain modülleri ve Entity'leri (`clrinf add module`, `clrinf add entity`), `@clrinf/core` veya `clrinf_core` gibi harici paket bağımlılığı zorunluluğu olmadan saf, yapısal olarak uyumlu (`OperationClaim`) sözleşmelerle üretilir.

---

## 3. Çok Kiracılı Cedar Yetkilendirme Mimarisi (Universal Multi-Tenant Cedar)

Yetkilendirme sistemi **tüm dillerde (C#, Rust, TypeScript)** Amazon Cedar policy dili temelinde ve **varsayılan olarak çok kiracılı (multi-tenant by default)** çalışır.

### 3.1. Temel İlkeler

1. **Varsayılan Kiracı İzolasyon Muhafızı (Tenant Isolation Guard)**:
   - İstekte bulunan kullanıcının kiracısı (`principal.tenant_id`) ile üzerinde işlem yapılan kaynağın kiracısı (`resource.tenant_id`) birebir eşleşmek zorundadır.
   - Bu eşleşme sağlanmadığında erişim doğrudan **TenantViolation** hatasıyla engellenir.
2. **Cedar Forbid Muhafazası**:
   - Cedar'da `forbid` kuralı tüm `permit` kurallarını ezer (kesin öncelik).
   - Kiracı uyuşmazlığı durumunda `forbid` kuralı devreye girer ve kullanıcının başka bir `permit` rolü olsa dahi işlemi engeller.
3. **PlatformAdmin İstisnası**:
   - Sadece global sistem yöneticisi olan `PlatformAdmin` rolü kiracılar arası (cross-tenant) operasyon yapabilir.
4. **TenantAdmin Kapsamı**:
   - `TenantAdmin`, yalnızca kendi kiracısına ait kaynaklarda tam yetkilidir.

### 3.2. Kanonik Cedar Kural Şablonu (`policies/<module>.cedar`)

Her modül oluşturulduğunda (`clrinf add module <Name>`), o modüle ait çok kiracılı Cedar kural kümesi otomatik üretilir:

```cedar
// @clrinf:generated — Multi-Tenant Cedar Policy: Orders

// 1. PlatformAdmin: Kiracılar arası tam yetki
permit (
    principal in Role::"PlatformAdmin",
    action,
    resource
);

// 2. TenantAdmin: Kendi kiracısındaki Orders kaynaklarında tam yetki
permit (
    principal in Role::"TenantAdmin",
    action,
    resource in ResourceType::"Orders"
) when {
    resource.tenant_id == principal.tenant_id
};

// 3. Modül Yöneticisi: Kendi kiracısında tam CRUD yetkisi
permit (
    principal in Role::"OrdersManager",
    action in [
        Action::"orders.create",
        Action::"orders.read",
        Action::"orders.update",
        Action::"orders.delete"
    ],
    resource in ResourceType::"Orders"
) when {
    context.tenant_id == principal.tenant_id &&
    resource.tenant_id == principal.tenant_id
};

// 4. Okuma Rolü: Kendi kiracısında okuma yetkisi
permit (
    principal in Role::"OrdersReader",
    action == Action::"orders.read",
    resource in ResourceType::"Orders"
) when {
    context.tenant_id == principal.tenant_id &&
    resource.tenant_id == principal.tenant_id
};

// 5. KESİN ÇOK KİRACILI İZOLASYON MUHAFIZI (Forbid Guard):
// Kiracı uyuşmazlığı durumunda tüm izin kurallarını ezer!
forbid (
    principal,
    action,
    resource in ResourceType::"Orders"
) when {
    resource.tenant_id != principal.tenant_id
} unless {
    principal in Role::"PlatformAdmin"
};
```

---

## 4. Dillerdeki Çalışma Zamanı Uygulaması

### 4.1. Rust (`clrinfrs`)

- **Çekirdek**: `clrinfrs_core::authz::MultiTenantCedarAuthorizer`
- **Operasyon İddiası**:
  ```rust
  let claim = OperationClaim::with_tenant(
      "user_123",
      vec!["OrdersManager".into()],
      "tenant_corp_a",
      "orders.create",
      "orders",
      "tenant_corp_a",
  );
  let result = authorizer.authorize(&claim);
  assert!(result.is_ok());
  ```
- **Modül Bağlama**: `src/lib.rs` veya `src/main.rs` içine `pub mod <module>;` satırı eklenir.

### 4.2. TypeScript (`clrinfjs`)

- **Çekirdek**: `MultiTenantCedarAuthorizer` (`src/core/authz.ts`)
- **Operasyon İddiası**:
  ```typescript
  const claim: OperationClaim = {
    principalId: "user_456",
    roles: ["OrdersManager"],
    principalTenantId: "tenant_corp_a",
    action: "orders.create",
    resourceType: "orders",
    resourceTenantId: "tenant_corp_a",
    contextTenantId: "tenant_corp_a",
  };
  const result = authorizer.authorize(claim);
  if (!result.ok) {
    // 403 Forbidden - Tenant veya Yetki ihlali
  }
  ```
- **Modül Bağlama**: `src/index.ts` içine `export * as <module> from './<module>/index.js';` eklenir.

### 4.3. C# (`clrinfcs`)

- **Çekirdek**: `CedarPolicyService`, `AuthorizationBehavior<TRequest, TResponse>`
- **Modül Bağlama**: `ServiceRegistration.cs` içerisine `.AddSingleton<...>()` servis kayıtları enjekte edilir.
- **Dekorasyon**:
  ```csharp
  [CedarAuthorize("orders.create", "orders")]
  public record CreateOrderCommand(string CustomerId) : IRequest<OrderResult>;
  ```

---

## 5. CLI Komut Seti ve Geliştirici Deneyimi

### 5.1. Proje Başlatma (Brownfield Auto-Detection)

Mevcut veya yeni bir dizinde projenin dilini otomatik algılayarak manifest oluşturur:

```bash
# Otomatik tespit: Cargo.toml -> rust, package.json -> typescript, *.csproj/*.sln -> csharp
clrinf init

# Açık parametrelerle:
clrinf init --name MyService --lang rust --arch clean-cqrs
```

### 5.2. Proje Durumu Sorgulama

```bash
clrinf status
```
Çıktı örneği:
```text
📋 clrinf Proje Durumu

   Manifest: /home/burak/Projeler/my-app/clrinf.toml
   Proje: MyService
   Dil: rust
   Mimari: CleanCqrs
   Dağıtım: Monolith

   Etkin Modüller:
   ✅ caching
   ✅ logging
   ✅ transaction
   ✅ error_handling
   ✅ validation
```

### 5.3. Sıfır Sürtünmeli Cross-Cutting Concern Yönetimi

```bash
# Caching modülünü ekle (şablon üretilir, dile özgü bağlanır, manifest güncellenir)
clrinf module add caching --provider memory

# Modülü projeden temizle (korunan kullanıcı koduna dokunmadan kaldırır)
clrinf module remove caching

# Manifest ile proje dosyalarını senkronize et
clrinf module sync

# Modül listesini gör
clrinf module list
```

### 5.4. Domain Modül ve CRUD Entity Scaffolding

```bash
# Çok kiracılı Cedar kuralı ve handler'ları ile yeni modül ekle
clrinf add module Orders

# İlgili modüle CRUD entity'si ekle
clrinf add entity OrderItem -m Orders --prop name:string --prop price:f64 --prop quantity:i32
```

### 5.5. Yerleşik Modül Kataloğu ve Adaptasyon Motoru (`clrinf catalog` / `clrinf module adopt`)

`clrinf`, geliştiricilerin sıfırdan geliştirmekle vakit kaybetmemesi için hem yerleşik domain modülleri ve paketleri (`crm`, `deals`, `contacts`, `activities`) hem de altyapı modülleri (`authz`, `caching`, `logging`, `transaction`, `idempotency`, `outbox`) sunar:

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

# 4. C# projesine tekil Deals modülünü 'Sales' adıyla, ham (sıfır bağımlılık) aktar
clrinf module adopt deals \
  --to-project ./apps/BillingService/BillingService.csproj \
  --target-dir src/Features/Sales \
  --as Sales \
  --mode raw

# 5. Rust crate'ine Cedar yetkilendirme modülünü ham haliyle aktar
clrinf module adopt authz \
  --to-project ./crates/identity-worker/Cargo.toml \
  --target-dir src/security/authz \
  --mode raw

# 6. TypeScript projesine Contacts modülünü barrel export ile kablolanmış (wired) aktar
clrinf module adopt contacts \
  --to-project ./packages/storefront/package.json \
  --as Customers \
  --mode wired
```

- **`--mode raw`**: Sıfır harici paket bağımlılığı (`OperationClaim` yerel üretilir), izole domain mantığı, repository portu ve `policies/<module>.cedar` güvenlik politikası.
- **`--mode wired`**: Hedef dilin doğal modül ağacına (Rust'ta `pub mod`, TS'te `export * as`, C#'ta `IServiceCollection`) otomatik bağlantı ve Cedar güvenlik muhafızları.

---

## 6. Yapay Zeka Ajanları İçin Meta MCP Sunucusu Araçları

`clrinf-codegen mcp` komutu JSON-RPC 2.0 stdio üzerinden çalışarak Claude, Cursor, Antigravity ve IDE eklentilerine aşağıdaki araçları sunar:

| Araç Adı | Açıklama |
|---|---|
| `clrinf_list_catalog` | Yerleşik domain modülleri (CRM suite, deals, contacts, activities) ve altyapı modüllerini listeler. |
| `clrinf_adopt_module` | Var olan herhangi bir modülü hedef projeye raw (saf) veya wired modda nakleder. |
| `clrinf_project_status` | Proje dilini, mimarisini ve aktif modüllerini okur. |
| `clrinf_module_manage` | Cross-cutting modülleri (`caching`, `logging`, `outbox`, vb.) ekler, siler veya senkronize eder. |
| `clrinf_add_domain_module` | Dile özgü domain modülü ve Cedar policy'sini üretir. |
| `clrinf_add_entity` | Domain modülüne CRUD entity'si, repository portu ve tenant korumasını ekler. |
| `clrinf_lint_rules` | Roslyn, Syn ve TS AST mimari denetleyicilerini çalıştırır. |
| `clrinf_scaffold_rule` | Test edilebilir, saf iş kuralı üretir. |
| `clrinf_pubsub_topology` | Dağıtık olay topolojisini, ölü olayları ve yetim tüketicileri denetler. |
| `clrinf_generate_pubsub` | Kanonik CloudEvent Publisher ve Idempotent Subscriber üretir. |
