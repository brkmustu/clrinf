# Yeniden organizasyon kapsamı ve durum

## Kabul edilen sınırlar

- Rust/C#/TypeScript/Elixir depoları submodule olarak korunur.
- İş alanları çekirdekten ayrılır; dört dil de kapsamda kalır.
- Çekirdek, teknoloji adaptörü, geliştirme aracı ve örnek uygulama ayrı belgelenir.
- Yerel monolith kullanımı ve mevcut projeye aşamalı ekleme birinci sınıf kullanım yoludur.
- Çalışma dizini değişiklikleri yayımlanmış sürüm değildir; commit/push kullanıcı kararıdır.

## Fazlar

| Faz | Çıktı | Durum |
|---|---|---|
| 0 | Derleme kaynakları, Elixir bağımlılık kilidi, CI, hata gizlemeyen E2E, lisans/link düzeltmeleri | Yerel değişiklikler uygulandı |
| 1 | Core/example sözleşme ayrımı, submodule çekirdek/adaptör ayrımı, manifest kataloğu, generic başlangıçlar | Entegrasyon sürüyor |
| 2 | Ortak conformance, context/hata standardı, SQLite outbox/inbox referansı, açık adaptör sınırları | Entegrasyon sürüyor |
| 3 | README, monolith/brownfield rehberleri, capability matrisi, doküman kontrolleri | Entegrasyon sürüyor |

Güncel sonuçlar tamamlanınca bu tablo güncellenir. Eski oturum raporundaki
öneri süreleri iş tahminiydi, tamamlanma kanıtı değildir.

## İlk rapordaki önerilerden ayrılan kararlar

- `sdk/{rs,cs,ts,ex}` tek depoya taşıma yerine mevcut submodule'ler korunur.
- .NET 10 preview değildir; ihtiyaç olmadan .NET 8 multi-target eklenmez.
- Her dilde aynı persistence motorunu taklit etmek yerine ortak portlar ve
  C# SQLite kalıcı referansı kullanılır.
- Generic Saga yürütücüsü ve otomatik OTel export yapılmış gibi gösterilmez.
  Saga tasarımı ve instrumentation uygulamanın açık entegrasyon sorumluluğudur.
- Örnek iş alanları çekirdeğin varsayılan yolu olmaktan çıkarılır.
- Örnek/kısmi CLI snapshot'ı çalışır paket gibi sunulmaz; root sözleşme CLI'ı
  ortak giriş, C# üreticisi uzmanlaşmış araç olarak korunur.

## Açık yayın/üretim kapıları

Alt modüller önce kendi depolarında yayımlanmalı; ardından umbrella gitlink'leri
güncellenmelidir. Temiz checkout CI ancak bu sürümlerle aynı dosyaları görebilir.
Tüm değişiklikler yereldeyken remote CI başarısı veya release hazır olduğu iddia edilmez.

Cowlib upstream bildirimleri [güvenlik listesinde](../operations/security-checklist.md)
açıktır. Showcase uygulamasının kendi auth/migration/entegrasyonları ayrı uygulama
sorumluluğudur.
