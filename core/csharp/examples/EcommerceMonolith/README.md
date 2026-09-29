# EcommerceMonolith — Direct Bounded Contexts & Flat Decomposition

`EcommerceMonolith` demonstrates the **Direct Bounded Contexts** (Root Slices) architectural pattern for high-throughput, low-cognitive-load .NET 10 systems, using Pure Functional Programming (`Result<T, DomainError>`), Native Dispatcher (`FrozenDictionary`), and Multi-Tenant Cedar Authorization.

---

## 🏛️ Mimari Yapı: Top-Level Bounded Contexts + Flat Decomposition

Bu projede her iş alanı (Bounded Context), ara bir `Modules/` klasörüne ihtiyaç duymadan **doğrudan projenin kök dizininde birinci sınıf vatandaş** (first-class citizen) olarak konumlanır:

```text
EcommerceMonolith/
├── Catalog/               # ─── Bounded Context: Katalog ───
│   ├── CatalogObjects.cs  # Entity, Command, Query, Repository arayüz ve InMemory implementasyonu
│   ├── CatalogRules.cs    # İş ve doğrulama kuralları (IBusinessRule<T>)
│   ├── CatalogHandlers.cs # İstek işleyicileri (IRequestHandler<TReq, TRes>)
│   └── CatalogModule.cs   # DI ServiceCollection uzantısı (AddCatalogModule)
├── Inventory/             # ─── Bounded Context: Stok / Envanter ───
│   ├── InventoryObjects.cs
│   ├── InventoryRules.cs
│   ├── InventoryHandlers.cs
│   └── InventoryModule.cs
├── Orders/                # ─── Bounded Context: Sipariş ───
│   ├── OrdersObjects.cs
│   ├── OrdersRules.cs
│   ├── OrdersHandlers.cs
│   └── OrdersModule.cs
├── Common/                # Result<T, DomainError> fonksiyonel monad altyapısı
├── Dispatching/           # Native Dispatcher (FrozenDictionary route table)
├── Domain/                # Temel domain hata tanımları (DomainError)
├── Policies/              # Çok kiracılı Cedar güvenlik ilkeleri (ecommerce.cedar)
├── Security/              # Cedar authorization pipeline behavior
├── Tests/                 # xUnit uçtan uca senaryo ve güvenlik testleri
└── clrinf.toml            # clrinf proje manifestosu
```

---

## 🎯 Neden Direkt Kök Slices ve Flat Ayrım Tercih Edildi?

### 1. Bounded Context Odaklılık (First-Class Citizens)
- İş alanları (`Catalog`, `Inventory`, `Orders`), projenin ana omurgasıdır. Projeyi açan bir geliştirici doğrudan domain modellerini görür.
- Ad alanı derinliği kısadır: `EcommerceMonolith.Catalog`, `EcommerceMonolith.Inventory` (araya gereksiz `.Modules.` girmez).

### 2. Düşük Bilişsel Yük (Low Cognitive Load via Flat Decomposition)
- Tek bir devasa `CatalogModule.cs` dosyası yerine, modül içi sorumluluklar düz (flat) dosyalara ayrılmıştır:
  - **`Objects`**: Veri yapıları, sözleşmeler ve depo arayüzü.
  - **`Rules`**: Saf iş kuralları ve domain invariantları.
  - **`Handlers`**: Yürütme ve orkestrasyon mantığı.
  - **`Module`**: Sadece DI ve servis kaydı.
- Derin alt klasör hiyerarşisi (`Features/Catalog/Commands/Create/Handlers/`) oluşturulmaz; tüm dosyalar aynı dizinde yan yana durur.

---

## 🔄 CrmMonolith ile Karşılaştırma: Hangi Yapı Ne Zaman Seçilmeli?

| Kriter | `EcommerceMonolith` (Direkt Bounded Contexts) | `CrmMonolith` (Containerized `Modules/`) |
| :--- | :--- | :--- |
| **Manifest Ayarı** | `folder_name` belirtilmez (doğrudan kök dizin) | `[backend.structure] folder_name = "Modules"` |
| **Ad Alanı (Namespace)** | `EcommerceMonolith.Catalog` | `CrmMonolith.Modules.Deal` |
| **Kök Dizin Görünümü** | Domain odaklı (domain klasörleri en tepede) | Altyapı odaklı (domain klasörleri `Modules/` içinde) |
| **Uygun Olduğu Senaryo** | Belirgin ve sınırlı sayıda (3-8 adet) Bounded Context içeren domain-driven sistemler | Onlarca küçük özellik/CRUD modülü barındıran geniş kurumsal platformlar |

---

## 🚀 Çalıştırma ve Test

```bash
# Projeyi derle
dotnet build examples/EcommerceMonolith/EcommerceMonolith.csproj

# Testleri çalıştır (13/13 test)
dotnet test examples/EcommerceMonolith/EcommerceMonolith.csproj
```
