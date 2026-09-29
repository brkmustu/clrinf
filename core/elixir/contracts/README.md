# ClrinfEx Contracts — Elixir Sözleşmeleri ve Şema Kataloğu

Bu dizin, `clrinfex` kütüphanesinin kanonik tel formatı (wire-format) JSON şemalarını ve mimari kurallarını içerir.

Kod üretim şablonları (`.tera`) dil repolarında değil, merkezi kod üretim aracı olan [`tools/clrinf-codegen`](../../tools/clrinf-codegen) bünyesinde tutulmaktadır. Elixir tarafında sözleşmeler doğrudan Elixir modülleriyle yaşar.

---

## Dizin Yapısı

```text
contracts/
├── constitution.md              # Bağlayıcı mimari anayasa ve Elixir/OTP ilkeleri
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

## Elixir Sözleşme Modülleri

Elixir sözleşme modülleri doğrudan `lib/clrinfex/core/` altında yer alır:
- **`Clrinfex.Core.Context`**: `tenant_id`, `correlation_id`, `causation_id` (`context.ex`).
- **`Clrinfex.Core.Error`**: `error_code`, `message`, `tenant_id`, `correlation_id`, `retryable`, `details` (`error.ex`).
- **`Clrinfex.Core.CloudEvent`**: CloudEvents 1.0 standardına uygun zarf (`cloud_event.ex`).

---

## Şema Doğrulama

JSON şemalarının standartlara uygunluğunu doğrulamak için:

```sh
clrinf-codegen check --schema-dir contracts/schemas
```
