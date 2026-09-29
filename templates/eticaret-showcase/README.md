# E-ticaret referans uygulaması

Bu dizin e-ticaret/PIM/checkout referans uygulamasıdır; clrinf mimarisinin
uçtan uca kullanımını örneklendirir.

## İçerik ve sınırlar

- `backend/`: .NET uygulama/domain örnekleri.
- `frontend/`: Svelte vitrin örneği.
- `docs/`: İş alanı mimari ve tasarım notları.
- İş alanı şemaları [examples/contracts](../../examples/contracts/README.md) altındadır.
- Kampanya motoru [Rust örneklerinde](../../core/rust/examples/clrinf-kampanya/) yer alır.

Root Compose, auth/campaign/checkout uygulamasını otomatik başlatmaz. Generic
altyapı akışı için root README ve minimal dil örnekleri kullanılmalıdır.

## Uyumluluk notu

Bu eski örnek kendi uygulama veritabanı, auth varsayımları ve domain politikaları
ile geliştirilmiştir. Genel RS256 auth adaptörüne geçiş, backend token
doğrulaması ve frontend login akışının birlikte uyarlanmasını gerektirir.
Eski HMAC token varsayımlarını yeni JWKS doğrulamasıyla uyumlu saymayın.

`docker-compose.yml` bu örneğin yerel başlangıç tarifidir; secret/config ve
uygulama migration'ları olmadan tek komutla üretime hazır kurulum değildir.
Yeni projeler için manifest kataloğundaki minimal şablonlar önerilir.
