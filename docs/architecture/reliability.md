# Dayanıklılık ve teslim semantiği

## İdempotency bir sonuç protokolüdür

Kapsam `tenant + operation/consumer + key` olmalıdır. Aynı anahtarla farklı
istek yükü gelirse çakışma üretilmeli; önceki başarılı sonuç gibi dönülmemelidir.
Başarısız işlem, anahtarı kalıcı olarak tamamlanmış duruma getirmemelidir.
Bellekte tutulan bir anahtar kümesi yalnızca o süreç ömrü için koruma sağlar.

## Kalıcı outbox/inbox

İş verisi, gelen mesajın işlenmiş kaydı ve giden olay aynı yerel veritabanı
işleminde yazılır. Yayıncı yalnızca commit edilmiş outbox kayıtlarını alır.
Bir işçi öldüğünde lease süresi dolan kaydın başka bir işçi tarafından
alınabilmesi gerekir. Onay/yeniden deneme işlemleri lease sahibiyle eşleşmelidir.

Yayın ile outbox onayı arasında çökme olursa mesaj tekrar yayınlanabilir:
bu beklenen bir **at-least-once** durumudur. Alıcı kendi inbox/işlem sınırında
tekrarı önlemelidir. Dış ödeme/e-posta/HTTP etkisi yerel SQLite işleminin parçası
değildir; sağlayıcının idempotency desteği veya uzlaştırma gerekir.

`core/csharp` içindeki SQLite adaptörü kalıcı yerel işlem referansıdır. Diğer dillerin
bellek adaptörleri aynı kalıcılık iddiasını taşımaz. Kullanıcı uygulaması başka
veritabanı kullandığında yalnızca outbox'ı ayrı SQLite dosyasına yazmak atomiklik
sağlamaz; iş verisiyle aynı transaction'ı paylaşan adaptör gerekir.

## İzleme ve işlem kimlikleri

- `tenant_id`: izolasyon kapsamı; kimliği doğrulanmış principal ile eşleştirilir.
- `correlation_id`: tüm iş akışını ilişkilendirir, birden çok olayı kapsar.
- `causation_id`: doğrudan tetikleyici istek/olay.
- Olay `id`: yeniden teslimde değişmemesi gereken olay kimliği.

Traceparent/OTel entegrasyonu ayrıca kurulmalıdır. Bu kimliklerin log veya
zarf içinde bulunması kendi başına dağıtık tracing export edildiğini kanıtlamaz.

## Üretime geçiş kapıları

Yeniden başlatma, rollback, eşzamanlı işçiler, lease süresi dolması, tekrar teslim,
tenant sınırı ve çelişen idempotency payload davranışları ölçülmelidir. Retry
sayısı/süresi, hatalı mesaj karantinası, alarm ve manuel uzlaştırma uygulamaya
özgü operasyonel kararlardır. Core NATS canlı köprüsünde kaçan olayların replay'i yoktur.
