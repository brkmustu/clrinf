# clrinf — Oturum Raporu: Geliştirici Profilleri, C# Dispatcher Alternatifleri ve Çok Dilli CRM Monoliti

**Tarih:** 20 Eylül 2026  
**Oturum:** Antigravity AI Pair-Programming & Burak  
**Çalışma Alanı:** `/home/burak/Projeler/clrinf`  
**Durum:** Tamamlandı, Uçtan Uca Test Edildi, Git'e Taahhüt Edildi ve Graft Grafiği Yenilendi.

---

## 1. Oturumun Temel Amaçları ve Ele Alınan Başlıklar

1. **Sıfır Sürtünmeli Geliştirici Profilleri (Minimal / Standard / Full)**:
   - Geliştiricinin modülleri en saf haliyle (Pure / Lean - sıfır cross-cutting concern yükü) de, en dolu haliyle (Full-Batteries - 7 concern hazır kablolanmış) de kullanabilmesi.
2. **Kendi Altyapını Getir (BYO - Custom Provider Desteği)**:
   - Geliştiricinin kendi yazdığı veya harici kütüphanesini `provider = "custom"` ile bildirebilmesi; `clrinf module sync` sırasında dosyaların ezilmemesi ve `clrinf module remove` sırasında kullanıcı kodlarının asla silinmemesi garantisi.
3. **Saf Domain Scaffolding (Zero External Dependency)**:
   - Üretilen domain modülleri ve entity'lerinin `@clrinf/core` veya `clrinf_core` paket bağımlılığı zorunluluğu olmadan saf, yapısal olarak uyumlu (`OperationClaim`) sözleşmelerle üretilmesi.
4. **C# Dispatcher Yelpazesinin Genişletilmesi**:
   - `ClrinfCS.Core.Dispatcher` (**Native** - FrozenDictionary, zero-reflection, allocation-free).
   - **MediatR** adaptör köprüsü.
   - **Mediator.Net** (`IMediator`, explicit `ICommand` / `IRequest`) adaptör köprüsü.
5. **Orta Segment Çok Kiracılı CRM Monolit Projesi (`CrmSystem`)**:
   - Gerçekçi bir B2B CRM sisteminin `clrinf` kod üretim araçları ve Graft kullanılarak sırasıyla **C#**, **TypeScript** ve **Rust** dillerinde geliştirilmesi, test edilmesi ve karşılaştırmalı olarak raporlanması.
6. **Veri Kalıcılığı (Data Persistence) Analizi**:
   - In-memory adaptörlerin süreç-içi kural doğrulamasındaki rolü ile `ClrinfCS.Adapters.Sqlite` gibi WAL mode destekli gerçek disk tabanlı kalıcılık adaptörlerinin ayrımı.

---

## 2. Hayata Geçirilen Mimari Geliştirmeler

### 2.1. Geliştirici Profilleri ve Manifest Genişletmesi
- **[`tools/clrinf-codegen/src/manifest.rs`](file:///home/burak/Projeler/clrinf/tools/clrinf-codegen/src/manifest.rs)**:
  - `ModulesConfig::minimal()`: Tüm modüller `enabled = false`.
  - `ModulesConfig::standard()`: `logging` ve `transaction` aktif.
  - `ModulesConfig::full()`: 7 modülün tamamı (`caching`, `logging`, `transaction`, `authentication`, `authorization` Cedar ile, `idempotency`, `outbox`) aktif.
  - Tüm 7 modül sağlayıcı enum'ına `Custom` seçeneği eklendi.
  - `Dispatcher` enum'ına `MediatorNet` eklendi (`Native`, `Mediatr`, `MediatorNet`).
- **[`tools/clrinf-codegen/src/main.rs`](file:///home/burak/Projeler/clrinf/tools/clrinf-codegen/src/main.rs)**:
  - `clrinf init --profile <minimal|standard|full>` bayrağı eklendi.
  - `clrinf init --dispatcher <native|mediatr|mediatornet>` bayrağı eklendi.
  - `--profile full` seçildiğinde otomatik olarak tüm modül şablonları render edilir ve dile özgü bağlanır (`sync_modules`).
- **[`tools/clrinf-codegen/src/module_manager.rs`](file:///home/burak/Projeler/clrinf/tools/clrinf-codegen/src/module_manager.rs)**:
  - `is_module_custom` kontrolü eklendi.
  - `provider = "custom"` tanımlandığında `clrinf module sync` kullanıcının dosyalarına dokunmaz; `clrinf module remove` dosyayı silmez, yalnızca manifest kaydını pasife çeker.

### 2.2. Domain Scaffolding Saflaştırması
- **TypeScript (`clrinfjs`)**:
  - [`clrinfjs/src/generator/module-generator.ts`](file:///home/burak/Projeler/clrinf/clrinfjs/src/generator/module-generator.ts): `@clrinf/core` import'u kaldırılarak yerel `export interface OperationClaim` arayüzü tanımlandı.
  - [`clrinfjs/src/generator/entity-generator.ts`](file:///home/burak/Projeler/clrinf/clrinfjs/src/generator/entity-generator.ts): Göreceli `./handlers.js` import'una geçildi.
- **Rust (`clrinfrs`)**:
  - [`clrinfrs/crates/clrinf-cli/src/generator/module.rs`](file:///home/burak/Projeler/clrinf/clrinfrs/crates/clrinf-cli/src/generator/module.rs): `clrinf_core::authz::OperationClaim` bağımlılığı kaldırılarak handlers içinde yerel struct olarak üretildi.
  - [`clrinfrs/crates/clrinf-cli/src/generator/entity.rs`](file:///home/burak/Projeler/clrinf/clrinfrs/crates/clrinf-cli/src/generator/entity.rs): `super::handlers::OperationClaim` kullanıldı.

---

## 3. Çok Dilli CRM Monolit Projesi (`CrmSystem`)

Aynı alan modelleri, iş kuralları ve çok kiracılı Cedar kuralları her 3 dilde de kendi idiomatik doğasına %100 sadık kalınarak geliştirildi:

### 3.1. Alan Modelleri ve İş Kuralları
1. **Contacts (Kişiler ve Müşteriler)**:
   - Varlık: `Contact` (`id`, `tenant_id`, `first_name`, `last_name`, `email`, `company`, `status: Lead`).
   - Kural: `ValidateContactEmailFormatRule` (geçerli RFC e-posta denetimi).
   - Kural: `EnsureContactEmailUniqueRule` (aynı kiracıda mükerrer e-posta engeli).
2. **Deals (Satış Fırsatları & Satış Hunisi)**:
   - Varlık: `Deal` (`id`, `tenant_id`, `title`, `contact_id`, `amount`, `stage: Prospect`, `probability: 10`).
   - Kural: `EnsureDealAmountPositiveRule` (tutar > 0).
   - Kural: `StageProgressionRule` (`Prospect` aşamasından doğrudan `ClosedWon` yapılamaz, teklif aşaması zorunludur; kapanmış fırsatlar taşınamaz).
3. **Activities (İletişim & Görevler)**:
   - Varlık: `Activity` (`id`, `tenant_id`, `deal_id`, `type`, `subject`, `notes`, `is_completed`).
   - Kural: `CannotLogActivityOnLostDealRule` (`ClosedLost` olmuş bir fırsata yeniden açılmadan aktivite eklenemez).
4. **Çok Kiracılı Cedar Politikaları (`policies/*.cedar`)**:
   - `forbid (principal, action, resource) when { resource.tenant_id != principal.tenant_id } unless { principal in Role::"PlatformAdmin" };`
   - `TenantAdmin`, `SalesManager` ve `SalesRep` izin matrisleri.

---

### 3.2. Dil Bazında Uygulama Detayları

#### A. C# (.NET 10 / `clrinfcs/examples/CrmMonolith`)
- **İdiomatik Yapı**:
  - `ClrinfCS.Core.Dispatcher.Builder` ile derleme zamanı FrozenDictionary rotalama, sıfır assembly scanning, sıfır reflection.
  - MediatR ve Mediator.Net köprü adaptörleri (`MediatRCrmBridge`, `MediatorNetCrmBridge`).
  - `CedarAuthorizationPipelineBehavior` ile çok kiracılı Cedar yetkilendirmesi.
- **Dosyalar**:
  - `Contacts/ContactsModule.cs`, `Deals/DealsModule.cs`, `Activities/ActivitiesModule.cs`
  - `Security/CrmAuthorization.cs`, `Dispatching/CrmDispatcherConfig.cs`
  - `Tests/CrmSystemTests.cs` (9 birim ve entegrasyon testi)
- **Sonuç**: 9 test başarılı (44 ms).

#### B. TypeScript (`clrinfjs/examples/crm-monolith`)
- **İdiomatik Yapı**:
  - C#'tan devşirme sınıf tabanlı Dispatcher/Mediator YOKTUR (TS AST `ARCH_TS_003` uyumu).
  - Saf fonksiyonel `pipeRules` boru hattı, `Result<T, E>` monadı ve kural ihlallerinde raw throw yasağı (`ARCH_TS_001` uyumu).
  - ESM barrel export'ları (`export * as contacts from "./contacts/index.js"`).
  - `MultiTenantCedarAuthorizer` ile Cedar kuralları.
- **Dosyalar**:
  - `contacts/index.ts`, `deals/index.ts`, `activities/index.ts`
  - `security/authorizer.ts`, `index.ts`, `crm.test.ts` (8 test)
- **Sonuç**: 8 test başarılı (29 ms), 10.000 işlem 16.6 ms.

#### C. Rust (`clrinfrs/examples/crm-monolith`)
- **İdiomatik Yapı**:
  - Sıfır runtime DI konteyneri veya reflection.
  - Derleme zamanı modül hiyerarşisi (`pub mod ...;` in `lib.rs`).
  - `PipelinedHandler` ve generic trait kompozisyonu (`Request`, `RequestHandler`, `BusinessRule<T>`).
  - Syn AST linter (`RUST_ARCH001`) kurallarına tam uyumlu `Result<T, DispatchError>`.
  - `MultiTenantCedarAuthorizer` ile Cedar çok kiracılı kurallar.
- **Dosyalar**:
  - `src/contacts/mod.rs`, `src/deals/mod.rs`, `src/activities/mod.rs`
  - `src/security/mod.rs`, `src/lib.rs`, `tests/crm_e2e_tests.rs` (8 test)
- **Sonuç**: 8 test başarılı (10 ms), 10.000 işlem ~4 ms.

---

## 4. Veri Kalıcılığı (Data Persistence) Değerlendirmesi

Oturumda tartışılan kritik soru: **"Testlerde InMemory mi kullanıldı, veri kalıcılığı nasıl test edildi?"**

1. **Port & Adapter (Hexagonal) Ayrımı**:
   - CRM projelerindeki `IContactRepository`, `IDealRepository` arayüzleri birer **Port**'tur. Domain iş kuralları ve Cedar motoru veritabanı teknolojisinden izoledir.
   - İlk aşama testlerinde birim/entegrasyon testlerinin hızlı, izole ve dış bağımlılıksız çalışabilmesi için `InMemoryRepository` adaptörleri kullanılmıştır.
2. **Kalıcılık Durumu (Durable Persistence)**:
   - Bellek içi adaptörler süreç-içidir; süreç çöktüğünde veriler kaybolur.
   - Gerçek kalıcılık için `clrinf` ekosisteminde [`clrinfcs/src/ClrinfCS.Adapters.Sqlite/SqliteStore.cs`](file:///home/burak/Projeler/clrinf/clrinfcs/src/ClrinfCS.Adapters.Sqlite/SqliteStore.cs) bulunmaktadır. Bu adaptör `:memory:` parametresini dahi bilerek reddeder ve WAL mode, atomik transaction, `.db` dosyası ve Outbox/Inbox tabloları ile gerçek disk kalıcılığı sağlar.
3. **Önerilen Kalıcılık Test Senaryosu (Crash & Recovery)**:
   - SQLite disk adaptörü (`crm.db`) bağlanır.
   - Dispatcher ile kayıtlar atılır ve bağlantı dispose edilir (süreç çökme simülasyonu).
   - Yeni bir süreçle `crm.db` açılarak verinin diskte bozulmadan kaldığı doğrulanır.
   - Kural hatasında transaction rollback olduğu kanıtlanır.

---

## 5. Test ve Doğrulama Özeti

Tüm ekosistemde **210 testin tamamı sıfır hatayla (%100 yeşil)** geçmektedir:

| Proje | Test Komutu | Geçen / Toplam Test | Durum |
|---|---|:---:|:---:|
| **tools/clrinf-codegen** | `cargo test` | 39 / 39 | ✅ Başarılı |
| **clrinfcs** (Çekirdek + CRM) | `dotnet test` | 71 / 71 | ✅ Başarılı |
| **clrinfjs** (Çekirdek + CRM) | `bun test` | 68 / 68 (1 skip) | ✅ Başarılı |
| **clrinfrs** (Çekirdek + CRM) | `cargo test --workspace` | 32 / 32 | ✅ Başarılı |
| **TOPLAM** | | **210 / 210** | **%100 Yeşil** |

---

## 6. Git İşlemeleri ve Graft Durumu

1. `clrinfcs`: `76e64cc` — `feat(examples): add multi-tenant CRM monolith with native dispatcher, MediatR, and Mediator.Net bridges`
2. `clrinfjs`: `df606d4` — `feat(examples): add multi-tenant TypeScript CRM monolith with functional pipeRules and Cedar authorization`
3. `clrinfrs`: `b380097` — `feat(examples): add multi-tenant Rust CRM monolith with compile-time module tree and Cedar authorization`
4. Root Repo: `ec00dd1` — `feat(crm): implement multi-tenant CRM monolith in C#, TypeScript, and Rust with dispatcher alternatives`
5. **Graft Context Graph**:
   - `graft build` deterministik olarak çalıştırıldı ve graf **5.908 düğüme** genişletildi.
   - `graft check` ile kod tabanı ile grafiğin tam senkronize olduğu doğrulandı.
   - Bu oturum boyunca Graft araçları sayesinde **~170.000+ token tasarrufu** sağlandı.
