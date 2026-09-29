# Mevcut projeye aşamalı entegrasyon

Tüm uygulamayı yeniden üretmek veya clrinf showcase'ine taşımak gerekmez.

## Önce wire-format

Mevcut domain modelleriniz yerinde kalır. Dış sınırda `_context`, `_error` ve
`_envelope` şemalarıyla eşleyen bir adaptör kurun. Eski event isimlerini sırf
genel görünmesi için değiştirmeyin; şema major değişimlerini ayrı yönetin.

```bash
# Mevcut domain şemalarını kendi dizininizden üretin (veya cargo run --manifest-path ... ile)
clrinf generate \
  --schema-dir ./my-contracts \
  --templates-dir ./tools/clrinf-codegen/templates \
  --output ./generated-contracts
```

`my-contracts` sizin şema dizininizdir; örnek şemaları kopyalamadan doğrudan
kendi modellerinizi tanımlayabilirsiniz. Üreticinin desteklediği JSON Schema alt kümesini okuyun;
desteklenmeyen model kısıtlarını sessizce tip garantisi saymayın.

## Sonra bir dilin çekirdeği

Rust'ta path crate, C#'ta project reference, TypeScript'te paket/dosya export'u,
Elixir'de path dependency ile yerelde başlayabilirsiniz. Henüz yayımlanmamış
bir paket sürümünü registry'de varmış gibi kullanmayın.

Kimlik doğrulaması mevcut uygulamada kalabilir. Tenant header'ını doğrulanmış
principal ile eşleyin; rastgele istemci girdisini güvenilir tenant context'e
dönüştürmeyin.

## En son kalıcılık ve dağıtım

Mevcut transaction sınırını koruyun. Outbox iş verisiyle aynı transaction'a
katılamıyorsa çift yazının atomik olduğunu iddia etmeyin. Bir modülü dış
servise taşıdığınızda retry, timeout, deduplication ve uzlaştırma tasarımını
ayrı kabul ölçütleriyle ekleyin.

Geçişte eski ve yeni error/event biçimlerini köprülemek için sınır adaptörleri
kullanın; bütün çağrıları tek seferde değiştirmek yerine bir dikey akışla başlayın.

---

## Modül Nakli ve Ham Entegrasyon (`clrinf module adopt`)

Var olan projenize `clrinf`'te bulunan hazır bir modülü (örneğin CRM Satış/Deals modülünü, Müşteriler/Contacts modülünü veya Cedar yetkilendirmesini) **sıfır harici paket bağımlılığıyla** doğrudan ham kod olarak aktarabilirsiniz.

Hedef proje olarak bir `.csproj`, `Cargo.toml`, `package.json` ya da proje dizini göstermeniz yeterlidir:

```bash
# C# (.csproj) projesine Deals modülünü 'Sales' adıyla, saf domain haliyle aktar
clrinf module adopt deals \
  --to-project ./apps/BillingService/BillingService.csproj \
  --target-dir src/Features/Sales \
  --as Sales \
  --mode raw

# Rust crate'ine Cedar yetkilendirme altyapısını ham haliyle aktar
clrinf module adopt authz \
  --to-project ./crates/identity-worker/Cargo.toml \
  --target-dir src/security/authz \
  --mode raw

# TypeScript projesine Contacts modülünü barrel export ile bağlayarak aktar
clrinf module adopt contacts \
  --to-project ./packages/storefront/package.json \
  --target-dir src/modules/customers \
  --as Customers \
  --mode wired
```

### Mod Garantileri:
- **`--mode raw`**: Hiçbir `@clrinf/core` veya `clrinf_core` paket zorunluluğu dayatılmaz. C#'ta `OperationClaim` sözleşmesi modül içine gömülü üretilir. Projenizin kendi kurallarına göre özgürce adapte edebilirsiniz.
- **`--mode wired`**: Hedef dilin standart modül yapısına (Rust `pub mod`, TS `export * as`, C# `IServiceCollection`) otomatik kaydolur.
- **Cedar Politikası**: Nakledilen her modülle birlikte çok kiracılı güvenlik muhafızı (`policies/<modul>.cedar`) dil-bağımsız olarak üretilir.

