# EticaretApp

EticaretApp, .NET 10 (LTS) Clean Architecture, CQRS ve DDD prensiplerine uygun olarak `clrinf` ile üretilmiş projedir.

---

## 🚀 Hızlı Başlangıç

### 1. Yerel Veritabanını Çalıştırma
PostgreSQL veritabanını Docker ile başlatın:
```bash
docker-compose up -d
```

### 2. Uygulamayı Çalıştırma
WebAPI uygulamasını başlatın:
```bash
dotnet run --project src/WebAPI/WebAPI.csproj
```

OpenAPI arayüzüne erişim:
- **WebAPI OpenAPI UI:** `http://localhost:5055/openapi/v1.json` (veya `/swagger`)

---

## 🔐 Varsayılan Seed Admin Kullanıcı Bilgileri

Güvenlik (JWT Auth) modülü aktif edildiğinde veritabanına varsayılan olarak aşağıdaki seed admin kullanıcısı eklenir:

- **E-Posta:** `admin@admin.com`
- **Şifre:** `Admin123!`
- **Roller / Yetkiler:** `Admin`, `User`

> ⚠️ **Güvenlik Uyarısı:** Canlı ortama (Production) geçmeden önce varsayılan admin kullanıcısının şifresini ve appsettings.json içerisindeki JWT secret değerlerini güncelleyiniz.

---

## 📂 Katman Mimarisi

- **`src/Domain`**: Domain Varlıkları (Entities), Enuml'ar ve Temel Arabirimler.
- **`src/Application`**: CQRS Komut & Sorguları (Commands/Queries), DTO'lar, Pipeline Behavior'lar ve İş Kuralları.
- **`src/Persistence`**: EF Core DbContext, Entity Yapılandırmaları (Configurations) ve Repository Nesneleri.
- **`src/Infrastructure`**: Harici Servis Adaptörleri (Graylog Logging, JWT Token Helper, File Storage vb.).
- **`src/WebAPI`**: Controller Endpoint'leri, Middleware'ler ve OpenAPI Konfigürasyonu.

---

## 🛠️ CLI Komutları (`clrinf`)

Projeye yeni modül, varlık (entity) veya güvenlik katmanı eklemek için proje dizinindeyken:

```bash
# Güvenlik modülünü aktif etmek için (JWT Auth + Seed Admin):
clrinf add security           # (veya add basic-security / add advanced-security)

# Yeni bir ana modül eklemek için (Kodlama paradigması: -p oop|fp, Yetkilendirme: --secured):
clrinf add module <ModuleName> BaseDbContext [-p oop|fp] [--secured]

# Yeni bir entity ve CQRS katmanlarını eklemek için:
clrinf add entity <EntityName> BaseDbContext --module <ModuleName> [-p oop|fp] [--secured]

# Bağlı bir alt modül eklemek için:
clrinf add sub-module <SubModuleName> BaseDbContext --parent <ParentModuleName> [--paradigm oop|fp] [--secured]
```
