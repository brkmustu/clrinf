# ClrinfCS Contracts — C# Sözleşmeleri ve Şema Kataloğu

Bu dizin, `clrinfcs` kütüphanesinin kanonik tel formatı (wire-format) JSON şemalarını ve mimari kurallarını içerir.

Kod üretim şablonları (`.tera`) dil repolarında değil, merkezi kod üretim aracı olan [`tools/clrinf-codegen`](../../tools/clrinf-codegen) bünyesinde tutulmaktadır. C# tarafında sözleşmeler doğrudan C# tip güvenliğiyle yaşar.

---

## Dizin Yapısı

```text
contracts/
├── constitution.md              # Bağlayıcı mimari anayasa ve C# tasarım ilkeleri
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

## C# Tip Karşılıkları

C# tarafındaki tüm sözleşmeler doğrudan `ClrinfCS.Core` projesinde tip güvenli olarak tanımlıdır:
- **`RequestContext`**: `tenant_id`, `correlation_id`, `causation_id` alanlarını taşır.
- **`ErrorEnvelope`**: Kanonik hata modeli (`error_code`, `message`, `tenant_id`, `correlation_id`, `retryable`, `details`).
- **`CloudEvent`**: CloudEvents 1.0 standardına uygun olay modeli.
- **`ContractJson`**: Tip güvenli JSON serileştirme/deserileştirme motoru.

---

## Şema Doğrulama

JSON şemalarının standartlara uygunluğunu doğrulamak için `clrinf-codegen` aracını kullanabilirsiniz:

```sh
clrinf-codegen check --schema-dir contracts/schemas
```
