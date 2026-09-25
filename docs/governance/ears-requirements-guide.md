# EARS ile gereksinim yazımı

Gereksinimler ölçülebilir olmalı, hedef ile mevcut uygulamayı ayırmalıdır.
EARS'ın beş temel biçimi şunlardır:

| Biçim | Örnek |
|---|---|
| Sürekli | THE adapter SHALL scope every record by tenant. |
| Olay odaklı | WHEN a document event is received, THE consumer SHALL validate its envelope. |
| Durum odaklı | WHILE the broker is disconnected, THE publish endpoint SHALL reject delivery with a retryable error. |
| İstenmeyen davranış | IF an idempotency key is reused with different content, THEN THE store SHALL report a conflict. |
| Opsiyonel özellik | WHERE durable delivery is enabled, THE application SHALL enqueue events in its local transaction. |

Bir kayıt gereksinim kimliği, kapsam, EARS cümlesi, sözleşme referansı,
başarı/hata örneği ve doğrulayan test konumunu içermelidir. Test yoksa
`hedef / henüz doğrulanmadı` olarak işaretleyin. Sabit bir listede `SAGA`
kelimesinin bulunması mimari uyumluluk testi değildir.
