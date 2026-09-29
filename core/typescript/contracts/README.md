# ClrinfJS Contracts — TypeScript Sözleşmeleri ve Şema Kataloğu

Bu dizin, `clrinfjs` kütüphanesinin kanonik tel formatı (wire-format) JSON şemalarını ve mimari kurallarını içerir.

Kod üretim şablonları (`.tera`) dil repolarında değil, merkezi kod üretim aracı olan [`tools/clrinf-codegen`](../../tools/clrinf-codegen) bünyesinde tutulmaktadır. TypeScript tarafında sözleşmeler doğrudan TypeScript tip güvenliğiyle yaşar.

---

## Dizin Yapısı

```text
contracts/
├── constitution.md              # Bağlayıcı mimari anayasa ve TypeScript ilkeleri
├── VERSIONING.md                # Şema evrimi, semantik sürümleme ve upcaster kuralları
├── slo.template.yaml            # Servis seviyesi hedefleri şablonu
├── agent-context.template.md    # Ajan bağlam şablonu
└── schemas/                     # Kanonik JSON Schemas (Draft 2020-12)
    ├── _context.schema.json     # tenant_id, correlation_id, causation_id
    ├── _envelope.schema.json    # CloudEvents 1.0 + clrinf ek alanları
    ├── _error.schema.json       # StandardErrorEnvelope (hata zarfı)
    └── module-manifest.schema.json # Modül manifest şeması
```

---

## TypeScript Tip Karşılıkları

TypeScript tarafındaki sözleşme yapıları doğrudan `src/core/contracts.ts` içinde yer alır:
- **`Context`**: `tenant_id`, `correlation_id`, `causation_id`.
- **`ErrorEnvelope`**: `error_code`, `message`, `tenant_id`, `correlation_id`, `retryable`, `details`.
- **`CloudEventEnvelope`**: CloudEvents 1.0 uyumlu olay interface'i.
- **`parseContext` / `parseEvent` / `parseError`**: Giriş doğrulama ve koruma fonksiyonları.

---

## Şema Doğrulama

JSON şemalarının standartlara uygunluğunu doğrulamak için:

```sh
clrinf-codegen check --schema-dir contracts/schemas
```
