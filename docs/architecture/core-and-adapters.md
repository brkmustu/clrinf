# Çekirdek, adaptör ve örnek uygulama

## Bağımlılık yönü

```text
Uygulama / iş alanı örneği
        |
        v
Dilin alan-bağımsız çekirdeği <--- HTTP / mesajlaşma / depolama adaptörü
        |
        v
Ortak JSON Schema ve wire-format kuralları
```

Çekirdek belirli bir ürün, kullanıcı personası, kupon, stok veya eğitim modeli
içermez. Sözleşme üreticisi iş alanını şemadan öğrenir. Örnek alanlar
`examples/contracts/schemas` altında tutulur; dosya taşıma wire-format
kimliklerini değiştirmez.

Submodule'ler dağıtılabilir birimler olarak korunur. Bir submodule'ün .NET
üreticisi veya Inspector gibi ek araçlar sunması, onun her API'sinin dört
dilde birebir bulunduğu anlamına gelmez. [Yetenek matrisi](parity-matrix.md)
kapsamı görünür yapar.

## Kullanım seviyeleri

1. **Sözleşme:** Şema ve üretilmiş modeller mevcut uygulamaya eklenir.
2. **Çekirdek:** Context, hata zarfı, olay zarfı ve yerel işlem arayüzleri kullanılır.
3. **Adaptör:** Gereken HTTP, depolama veya mesajlaşma teknolojisi seçilir.
4. **Uygulama:** İş kuralları, yetkilendirme politikaları, retry/Saga kararları uygulanır.

Temel çekirdeği kullanmak için PostgreSQL, Cedar, NATS, Nomad veya Docker
gerekmemelidir. Çok kiracılı kullanımda tenant izolasyonu zorunlu sonuçtur;
RLS bunun PostgreSQL'e özgü uygulama seçeneklerinden biridir.

## CLI sorumlulukları

Root `clrinf-codegen` çok dilli sözleşme üretimi ve manifest tabanlı şablon
kataloğudur. `clrinfcs` üreticisi .NET proje/özellik iskeletlerine odaklanır.
`clrinfjs/cli` eski, uygulama dosyaları eksik bir snapshot'tır; çalışır CLI
olarak sunulmaz ve bozuk bin kayıtları kaldırılır. Komutlar aynı ürün adı altında
eşdeğer işlevler sunuyormuş gibi belgelenmez.
