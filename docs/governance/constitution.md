# clrinf Constitution — Değişmez Mimari Kurallar

**Versiyon:** 1.1.0  
**Kapsam:** clrinf servisleri, kütüphaneleri, scaffold'ları ve AI ajanları için
tasarım kurallarıdır; tüm adapter/runtime özelliklerinin uygulanmış olduğunu
belgeleyen bir sertifika değildir.

## 1. RULE NO-DISTRIBUTED-TX

- Aynı servis veya yerel transaction sınırı içindeki ACID transaction desteklenen
  bir tasarım seçeneğidir; modular monolith kullanımını engellemez.
- Servis sınırları arasında distributed 2PC/XA kullanılmaz.
- Servisler arası süreçler için retry, timeout, idempotency ve gerekiyorsa
  Saga/compensating action politikaları açıkça belirlenir. Her işlem geri
  alınabilir değildir; geri alınamayan etkiler için recovery yolu tanımlanır.
- Dedupe anahtarı **tenant + operation + message/idempotency key** kapsamında
  tanımlanır. `correlation_id` workflow izleme içindir; tek başına evrensel
  dedupe anahtarı değildir. Saklama süresi ve atomiklik uygulama sorumluluğudur.

## 2. RULE TENANT-ISOLATION

- Her istek ve event tenant bağlamını taşır. Güvenilir sınırda tenant doğrulanır;
  depolama, cache, dedupe ve yetkilendirme tenant kapsamını korur.
- `_context.schema.json` içindeki `tenant_id`, `correlation_id`, `causation_id`
  string alanları kararlıdır; CloudEvents karşılıkları `tenantid`,
  `correlationid`, `causationid` olarak kesintisiz aktarılır.
- PostgreSQL RLS olası bir adapter stratejisidir, core gereksinimi değildir.
  NATS, PostgreSQL, Cedar ve Nomad opsiyonel capability adapter'larıdır;
  hiçbirinin kurulumu core kullanımı için zorunlu değildir.

## 3. RULE RESULT-PATTERN

- Beklenen domain/uygulama hataları idiomatik `Result<T, E>` / `Either` veya
  eşdeğer açık sonuç tipleriyle ifade edilir; exception normal kontrol akışı
  yerine geçmez.
- Servis sınırlarında `_error.schema.json` kararlı hata wire sözleşmesidir:
  `error_code`, `message`, `correlation_id`, `tenant_id`, `retryable` korunur.
- Adapter exception'ları uygun sınırda açık hatalara çevrilir; ağ hataları gibi
  geçici durumlar retry politikasına göre ele alınır, otomatik olarak fatal sayılmaz.

## 4. RULE CONTRACT-FIRST

- Implementasyondan önce wire sözleşmesi tanımlanır. Core sözleşmeleri
  `tools/clrinf-codegen/schemas`, uygulama örnekleri `examples/contracts/schemas`
  içindedir. Uygulamalar kendi sözleşmelerinin sahipliğini üstlenir.
- Kod tipleri canonical CLI ile sözleşmeden üretilir; core ve örnek giriş/çıkış
  dizinleri ayrı tutulur. DTO üretimi runtime şema doğrulamasının yerine geçmez.
- `x-domain` namespace metadata'sıdır, domain-specific framework bağımlılığı
  değildir. Eksik olduğunda codegen `common` kullanır.
- Tenant/context/error alanları ve envelope'un mevcut required listesi korunur;
  değişiklikler [versioning.md](versioning.md) politikasına tabidir.

## 5. RULE OBSERVABLE-BY-DEFAULT

- Tenant, correlation ve causation bağlamının korunması temel gereksinimdir.
  Yapılandırılmış loglar ve trace'ler bu bağlamla ilişkilendirilmelidir.
- OpenTelemetry önerilen instrumentation/export hedefidir; bütün dillerde veya
  adapter'larda hazır, eksiksiz uygulanmış olduğu garanti edilmez.
- Servis sahibi etkin instrumentation, exporter, sampling, veri gizliliği ve
  SLO kapsamını ayrıca doğrular. Telemetry backend seçimi core bağımlılığı değildir.
