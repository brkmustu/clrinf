# TypeScript / Bun

`core/typescript` (`clrinfjs`) çekirdeği context/hata/CloudEvent biçimlerini HTTP sunucusundan
bağımsız kullanır. Event Inspector ayrıca başlatılan geliştirme aracıdır;
core import'u sunucu/abonelik başlatmaz.

```bash
export CLRINF_CONFORMANCE_DIR="$PWD/tests/conformance/fixtures"
cd core/typescript
bun install --frozen-lockfile
bun test
```

Tip denetimi, minimal süreç içi örnek, HTTP adaptörü ve Inspector komutları
[TypeScript README'sindedir](../../core/typescript/README.md).

`JSON.parse(...) as CloudEvent` çalışma zamanı doğrulaması değildir. Dış girdiyi
core doğrulayıcılarıyla denetleyin. `is_error` / `is_compensation` bool alanlarını
açıkça taşıyın; olay adı üzerinden durum tahmin etmeyin.

Bellek outbox/idempotency adaptörleri geliştirme ve tek süreç davranışı içindir.
Üretimde aynı iş verisi transaction'ına katılan kalıcı adaptör gerekir.
