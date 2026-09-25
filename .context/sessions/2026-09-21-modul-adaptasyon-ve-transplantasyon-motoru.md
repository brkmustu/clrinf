# clrinf — Oturum Raporu: Modül Adaptasyon ve Nakil Motoru (`clrinf module adopt`)

**Tarih:** 21 Eylül 2026  
**Oturum:** Antigravity AI Pair-Programming & Burak  
**Çalışma Alanı:** `/home/burak/Projeler/clrinf`  
**Durum:** Tamamlandı, 219/219 Test Başarılı, Git'e Taahhüt Edildi, Graft Grafiği Senkronize.

---

## 1. Oturumun Temel Amacı

Geliştiricilerin `clrinf` ekosisteminde var olan herhangi bir modülü (ister `caching`, `authz` gibi altyapı modülleri, ister CRM'deki `deals`/satış, `contacts`, `activities` gibi domain modülleri, ister harici bir projedeki özel modülü) **var olan başka herhangi bir hedef projeye** (.csproj, Cargo.toml, package.json veya proje dizini):
1. **En saf haliyle (`--mode raw`)**: Sıfır harici paket bağımlılığıyla, tamamen izole ve kendi içinde çalışan sözleşmelerle aktarabilmesi,
2. **Kablolanmış haliyle (`--mode wired`)**: Hedef dilin doğal modül ağacına (Rust `pub mod`, TS `export * as`, C# `IServiceCollection`) otomatik entegre edebilmesi.

---

## 2. Hayata Geçirilen Mimari & Komut Seti

### 2.1. Yeni Komut Arayüzü: `clrinf module adopt` (veya `clrinf adopt`)

```bash
# C# (.csproj) projesine CRM Deals modülünü 'Sales' adıyla, saf (raw) haliyle aktarma
clrinf module adopt deals \
  --to-project ./apps/BillingService/BillingService.csproj \
  --target-dir src/Features/Sales \
  --as Sales \
  --mode raw

# Rust crate'ine Cedar yetkilendirme altyapısını ham haliyle aktarma
clrinf module adopt authz \
  --to-project ./crates/identity-worker/Cargo.toml \
  --target-dir src/security/authz \
  --mode raw

# TypeScript projesine Contacts modülünü modül ağacına bağlı (wired) aktarma
clrinf module adopt contacts \
  --to-project ./packages/storefront/package.json \
  --target-dir src/modules/customers \
  --as Customers \
  --mode wired

# Harici bir projedeki özel bir modülü hedef projeye aktarma
clrinf module adopt Invoicing \
  --source-project ../FinanceApp \
  --to-project ./crates/order-service \
  --target-dir src/invoicing \
  --mode raw
```

### 2.2. Parametre Matrisi

| Parametre | Açıklama | Varsayılan |
|---|---|---|
| `<MODULE>` | Nakledilecek modül adı (`deals`, `contacts`, `activities`, `caching`, `authz`, `crm` vb.) | Zorunlu |
| `--to-project <PATH>` | Hedef proje dosya yolu (`.csproj`, `Cargo.toml`, `package.json` veya dizin) | Zorunlu |
| `--target-dir <PATH>` | Hedef projedeki yerleşim dizini | Dile özgü varsayılan dizin (`src/Features/<Name>`, `src/<name>`, `src/modules/<name>`) |
| `--mode <raw\|wired>` | `raw`: Sıfır harici bağımlılık, saf domain ve yerel sözleşmeler.<br>`wired`: Dile özgü modül ağacına otomatik entegrasyon. | `raw` |
| `--as <ALIAS>` | Modülü naklederken yeniden adlandırma (örn: `deals` -> `Sales`) | Kaynak modül adı |
| `--source-project <PATH>` | Dış projedeki modülü alıp aktarma yolu | Yerleşik katalog |

---

## 3. Eklenen ve Güncellenen Bileşenler

1. **[`tools/clrinf-codegen/src/module_adapter.rs`](file:///home/burak/Projeler/clrinf/tools/clrinf-codegen/src/module_adapter.rs)**:
   - `TargetProject::resolve(&path)`: Polyglot hedef proje tespiti.
   - `CatalogModule`, `get_built_in_catalog()`, `print_catalog()`: Yerleşik resmi domain (`crm`, `deals`, `contacts`, `activities`) ve altyapı modülleri kataloğu.
   - Birleşik CRM Suite (`crm` / `crm.all`): Tek komutla Deals, Contacts, Activities ve birleşik `policies/crm.cedar` üretim desteği.
   - `ModuleAdapter::adopt(&opts)`: Modül çıkarımı, alias dönüşümü, yerel sözleşme üretimi, Cedar politika emit'i ve dış proje kaynak aktarımı.
2. **[`tools/clrinf-codegen/src/module_wiring.rs`](file:///home/burak/Projeler/clrinf/tools/clrinf-codegen/src/module_wiring.rs)**:
   - C# `wire_csharp_service_registration`: Hem altyapı hem domain modülleri için genelleştirilmiş DI servis kaydı (`services.Add<Name>Module()`).
   - Rust `wire_rust_module_tree`: Modül ağacına derleme zamanı bildirim enjeksiyonu (`pub mod <name>;`).
   - TypeScript `wire_typescript_barrel_export`: `export * as <name>` barrel export enjeksiyonu.
3. **[`tools/clrinf-codegen/src/main.rs`](file:///home/burak/Projeler/clrinf/tools/clrinf-codegen/src/main.rs)**:
   - `ModuleCommands::Catalog`, `Commands::Catalog`, `ModuleCommands::Adopt` ve `Commands::Adopt` varyantları eklendi.
   - `execute_module_adopt(...)` ve `print_catalog()` yürütücüleri bağlandı.
4. **[`tools/clrinf-codegen/src/mcp.rs`](file:///home/burak/Projeler/clrinf/tools/clrinf-codegen/src/mcp.rs)**:
   - `clrinf_list_catalog` ve `clrinf_adopt_module` MCP JSON-RPC stdio araçları eklendi.
5. **[`tools/clrinf-codegen/tests/e2e_lifecycle_tests.rs`](file:///home/burak/Projeler/clrinf/tools/clrinf-codegen/tests/e2e_lifecycle_tests.rs)**:
   - `test_catalog_command`
   - `test_adopt_builtin_crm_suite_raw_into_csharp`
   - `test_adopt_builtin_crm_suite_wired_into_rust`
   - `test_adopt_module_raw_into_csharp_csproj`
   - `test_adopt_module_raw_into_rust_crate`
   - `test_adopt_module_wired_into_typescript_package`
   - `test_adopt_module_custom_target_dir_and_caching`

---

## 4. Test ve Doğrulama Özeti

Tüm ekosistem testleri %100 yeşil tamamlanmıştır:

- **Codegen & CLI (`tools/clrinf-codegen`)**: 51 test (Birim, CLI, MCP, Catalog, Adopt E2E, Topology).
- **C# (.NET 10)**: 71 test (`ClrinfCS.Tests` + `CrmMonolith`).
- **TypeScript**: 68 test (Core, Rules, Cedar Authz, Linter, CRM Monolith).
- **Rust**: 32 test (`clrinf_core`, `crm_monolith`, Cedar Authz, Dispatcher).
- **Toplam Test**: **222 test** (0 başarısız).
- **Graft İndeksi**: 5.952 düğüm senkronize (`graft check: OK`).

