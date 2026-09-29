# clrinf — Versiyonlama ve Evrim Politikası

**Politika modeli:** SemVer + genişletme öncelikli şema evrimi.

## 1. Geriye dönük uyumluluk

- Yeni opsiyonel alan: tüketici uyumluluğu doğrulanarak Patch / Minor.
- Yeni sözleşme: Minor.
- Açıklama/dokümantasyon değişikliği: Patch.
- Alan silme/yeniden adlandırma, yeni zorunlu alan veya kabul edilen değerleri
  daraltma: breaking change; ayrı sözleşme sürümü ve açık migrasyon gerektirir.

Tenant/context/error wire alanları kararlıdır. `_envelope.schema.json` mevcut
required listesi korunur; eklenen `is_error` ve `is_compensation` boolean
alanları opsiyoneldir. Önceden bu extension adlarıyla boolean dışı değer gönderen
üreticiler düzeltilmelidir; opsiyonel olmak tür kontrolünü kaldırmaz.
`_error.schema.json` byte-for-byte korunmuştur. Yeni `_context.schema.json`,
`tenant_id`, `correlation_id`, `causation_id` string alanlarını zorunlu kılar.
Core şemaları paylaşılan `tests/conformance/fixtures` örnekleriyle doğrulanır.

## 2. Breaking changes ve upcasting

1. Mevcut mesajın şeması ve kimliği korunur.
2. Yeni sürüm ayrı dosyada tanımlanır.
3. Yeni mesajın `$id` ve varsa event `type` kimliği açıkça sürümlenir.
4. Eski tüketiciler için geçiş/deprecation takvimi ve gerekiyorsa upcaster sağlanır.

Deprecated şemalar kendi core/uygulama sözleşme kökünde en az iki major release
korunur. Event sourcing kullanan uygulama, saklanan eski event'ler için uyumlu
okuma/upcasting stratejisini ayrıca sürdürür; dosya taşımak geçmişi yeniden yazmaz.
Somut örnek:
`examples/contracts/schemas/kampanya/upcasters/KampanyaOlustur.v1-to-v2.yaml`.

## 3. Core / örnek ayrımı migrasyonu

`auth`, `eticaret`, `kampanya`, `katalog`, `oms`, `stok`, `egitim` dizinleri
`examples/contracts/schemas/` altındadır. İki EARS dosyası ve stok cache kuralı da taşınmıştır; tam eşleştirme
[migration index](../../examples/contracts/README.md#migration-index) içindedir.
Örneklerin içeriği, `$id`, event kimlikleri ve göreli alt dizin yapıları değişmez.
Dosya yolu tüketen script/config bağlantıları güncellenmelidir; bu bir wire
sürüm değişikliği değildir. Auth örneğinin credential/JWT/Cedar tercihleri core
auth zorunluluğu oluşturmaz.

`x-domain` codegen namespace metadata'sıdır; domain-specific framework
bağımlılığı değildir. Core için değer `common`'dır; değişmeyen `_error` şemasında
metadata eksiktir ve generator aynı varsayılanı kullanır. Namespace metadata'sını
değiştirmek generated API'yi etkileyebilir; wire kimliğiyle karıştırılmamalıdır.

Umbrella kökünden ayrı üretim:

```sh
clrinf-codegen generate --schema-dir tools/clrinf-codegen/schemas --output tests/generated/core
clrinf-codegen generate --schema-dir examples/contracts/schemas --output tests/generated/all
```

İkinci çıktı yalnızca örneklerdir. Eski üretilmiş dosyalar otomatik silinmez;
taşınan şemaları içeren eski çıktıları uygulama sahibi temizlemelidir.

## 4. Davranış ve adapter uyumluluğu

Wire uyumluluğu runtime eşdeğerliği garantisi değildir. Yerel ACID serbesttir;
servis sınırları arasında distributed 2PC/XA kullanılmaz. Dedupe kapsamı
tenant + operation + message/idempotency key'dir, yalnız `correlation_id` değildir.
NATS/PostgreSQL/Cedar/Nomad opsiyonel capability adapter seçimleridir.
OpenTelemetry önerilen hedeftir, tüm adapter'larda uygulanmış olduğu garantisi
değildir. Bu yeteneklerin ve durability/izolasyon davranışlarının sürüm uyumluluğu
ilgili implementasyonun testleriyle ayrıca doğrulanmalıdır.
