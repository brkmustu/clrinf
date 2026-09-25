# Diller arası yetenek matrisi

Bu matris wire-format ortaklığını runtime özellik eşitliğiyle karıştırmaz.
`Bellek` restart sonrası kalıcılık sağlamaz; `arayüz` hazır üretim adaptörü
değildir. Dört dil de destek kapsamındadır; tek bir dil diğerlerinin zorunlu
çalışma zamanı değildir.

| Yetenek | Rust | C# | TypeScript | Elixir |
|---|---|---|---|---|
| Context / standart hata / olay zarfı | Core | Core | Core | Core |
| Ortak geçerli/geçersiz JSON örnekleri | Native test | xUnit | Bun | ExUnit |
| Framework'süz / süreç içi kullanım | Core örneği | Dispatcher / örnek | Core örneği | `CLRINF_MODE=core` |
| HTTP adaptörü / örneği | Axum | ASP.NET Core örneği | Bun | Plug |
| İdempotency / outbox arayüzü | Var | Var | Var | Behaviour |
| Bellek referansı | Var, volatile | Kalıcı adaptör tercih edilir | Var, volatile | Var, volatile |
| İş verisi + inbox + outbox yerel atomiklik | Üretim adaptörü gerekir | SQLite referansı | Üretim adaptörü gerekir | Üretim adaptörü gerekir |
| Kalıcı leased outbox / yeniden başlatma | Referans yok | SQLite | Referans yok | Referans yok |
| Hazır canlı NATS gözlem/aktarım aracı | Yok | Yok | Inspector | Streaming köprüsü |
| Genel kalıcı Saga yürütücüsü | Yok | Yok | Yok | Yok |
| Her serviste otomatik OTel export | Garanti edilmez | Garanti edilmez | Garanti edilmez | Garanti edilmez |

## Destek sınırı

Bu dağılım bilinçlidir: dört dilde aynı veritabanı adapter'ını yüzeysel olarak
kopyalamak yerine ortak wire-format, açık portlar ve bir kalıcı işlem
referansı sunulur. Başka bir dil/veritabanı için üretim kalıcılığı gerektiğinde
o adaptör aynı hata/lease/transaction kabul ölçütleriyle geliştirilir.

Core fixture testleri yalnız örneklenen JSON biçimlerini kapsar. Aşağıdakiler
ayrı doğrulama konularıdır: JWT doğrulaması, broker yeniden teslimi, tenant
yetkilendirmesi, büyük yük/backpressure, disk kaybı, dağıtık tracing.

## Doğrulama girişleri

Root CI native dil testlerini ve `tests/conformance/fixtures` girdilerini
çalıştırır. Kod üretimi `tests/generated/regenerate.sh` ile core ve örnek
sözleşmeleri ayrı üretir, `tests/generated/check.sh` dört dil çıktısını denetler.
`docker-compose.test.yml` gerçek Elixir -> NATS -> Inspector yolunu kapsar;
bu senaryo dört dilde hazır Saga motoru bulunduğunun kanıtı değildir.
