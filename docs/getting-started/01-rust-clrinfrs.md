# Rust

`core/rust` (`clrinfrs`) bir Cargo workspace'idir; tek bir `src/main.rs` servis scaffold'u
değildir. `clrinf-core` iş alanından bağımsız context, zarf ve çekirdek
arayüzlerini sağlar. HTTP/kimlik/depolama adaptörleri ayrı sorumluluklardır.
Kampanya uygulaması `examples/clrinf-kampanya` altında bir örnektir.

```bash
export CLRINF_CONFORMANCE_DIR="$PWD/tests/conformance/fixtures"
cargo test --manifest-path core/rust/Cargo.toml --workspace
```

Çalışan monolith/HTTP örnekleri, public API ve dependency ekleme adımları
[Rust çekirdeğinin README'sindedir](../../core/rust/README.md).

Hata zarfı `error_code`, `message`, `correlation_id`, `tenant_id`, `retryable`
alanlarını korur. Context ayrıca `causation_id` taşır. Core'u kullanmak Axum,
NATS veya PostgreSQL'i iş modelinin bağımlılığı yapmamalıdır.

Auth adaptörünün imzalama anahtarları runtime yapılandırmasıdır. JWKS public
anahtar sunar; örnek persona listesi çekirdek kimlik modeli değildir. Demo
politikaları üretim yetkilendirme politikası olarak kullanılmamalıdır.
