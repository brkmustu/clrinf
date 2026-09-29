# CrmMonolith — Containerized Modular Monolith Reference

`CrmMonolith` demonstrates the **Containerized Modules** architectural pattern for .NET 10 modular monoliths, using Functional Programming (`Result<T, DomainError>`), Entity Framework Core (SQLite & InMemory), Minimal APIs, and the Native Dispatcher.

---

## 🏛️ Mimari Yapı: Containerized Modules (`Modules/`)

Bu projede tüm iş alanı modülleri, konfigürasyonda açıkça belirtilen bir **kapsayıcı klasör** (`Modules/`) altında toplanmıştır:

```text
CrmMonolith/
├── Common/                # Fonksiyonel tipler (Result<T, E>), güvenlik yardımcıları
├── Data/                  # Altyapı ve veritabanı (CrmDbContext, EF Core mapping)
├── Domain/                # Çapraz domain varlıkları (User, OperationClaim vb.)
├── Modules/               # ─── KAPSAYICI MODÜL DİZİNİ ───
│   ├── Activity/          # Activity domain modülü (Activity, Rules, CRUD slices)
│   ├── Contact/           # Contact domain modülü (Contact, Rules, CRUD slices)
│   └── Deal/              # Deal domain modülü (Deal, Rules, CRUD slices)
├── Policies/              # Çok kiracılı Cedar yetkilendirme ilkeleri (crm.cedar)
├── Tests/                 # xUnit uçtan uca ve birim testleri
├── Program.cs             # ASP.NET Core minimal API giriş noktası
├── clrinf.toml            # clrinf proje manifestosu
└── codegen.toml           # Kod üretim konfigürasyonu
```

### Manifest Konfigürasyonu ([clrinf.toml](clrinf.toml))

```toml
[project]
name = "CrmMonolith"
lang = "csharp"
arch = "flat"
dispatcher = "native"
paradigm = "fp"
db_context = "CrmDbContext"
namespace = "CrmMonolith"

[backend.structure]
mode = "module"
folder_name = "Modules"
```

---

## 🎯 Neden `Modules/` Kapsayıcısı Tercih Edildi?

1. **Altyapı ile İş Alanının Fiziksel Olarak Ayrılması:**
   - Projenin kök dizininde `Data/`, `Domain/`, `Common/`, `Policies/` gibi altyapı klasörleri yer alırken, domain özellikleri `Modules/` altında izole edilir.
   - Kök dizinin onlarca özellikle kalabalıklaşması önlenir.

2. **Geniş ve Çok Modüllü Sistemler İçin Ölçeklenebilirlik:**
   - 20-30 farklı domain özelliğine sahip kurumsal projelerde, tüm modüllerin tek bir `Modules/` altında gruplanması navigasyonu kolaylaştırır.

3. **Hiyerarşik Ad Alanı (Namespace):**
   - Tipler `CrmMonolith.Modules.Deal`, `CrmMonolith.Modules.Contact` ad alanları altında mantıksal olarak sınıflandırılır.

---

## 🚀 Çalıştırma ve Test

```bash
# Projeyi derle
dotnet build examples/CrmMonolith/CrmMonolith.csproj

# Testleri çalıştır
dotnet test examples/CrmMonolith/CrmMonolith.csproj
```
