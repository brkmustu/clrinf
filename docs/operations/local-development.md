# Yerel geliştirme

Çekirdek kütüphaneleri ve contract-only üretim Docker gerektirmez.
Root Compose yalnızca isteğe bağlı geliştirme adaptörlerini açar.

| Komut | Açılan servisler |
|---|---|
| `docker compose up -d` | NATS (51700) |
| `docker compose --profile tools up -d --build` | NATS (51700), Inspector (51703), Elixir streaming (51704) |
| `docker compose --profile storage up -d` | NATS (51700), PostgreSQL (51780), Redis (51781) |
| `docker compose --profile observability up -d` | NATS (51700), OTel gRPC (51783), OTel HTTP (51784), Tempo, Loki, Prometheus, Grafana |

clrinf port şeması — ortak geliştirme portlarıyla çakışmaz:

| Port | Servis | Geçersiz kılma değişkeni |
|---|---|---|
| `51700` | NATS broker | `NATS_PORT` |
| `51701` | core/rust generic HTTP (Rust) | `PORT` |
| `51702` | core/csharp .NET HTTP service | `PORT` |
| `51703` | core/typescript Event Inspector | `INSPECTOR_PORT` |
| `51704` | core/elixir Streaming / Presence | `STREAMING_PORT` |
| `51705` | clrinf-auth IAM / JWKS | `PORT` |
| `51780` | PostgreSQL | `POSTGRES_PORT` |
| `51781` | Redis | `REDIS_PORT` |
| `51782` | NATS monitor | `NATS_MONITOR_PORT` |
| `51783` | OTel Collector gRPC | `OTEL_GRPC_PORT` |
| `51784` | OTel Collector HTTP | `OTEL_HTTP_PORT` |

Host portları `127.0.0.1` ile sınırlandırılır. Yerel DB/Grafana varsayılan
parolaları yalnızca geliştirme içindir; üretim sırrı olarak kullanmayın.
Tüm portlar ortam değişkeniyle geçersiz kılınabilir.

NATS JetStream depolaması `/data` volume'una yönlendirilir. Bu, mevcut
streaming adaptörünün JetStream consumer kullandığı anlamına gelmez.
Inspector ve presence bellek state'i kalıcı değildir.

## Generic uçtan uca akış

```bash
docker compose -p clrinf-e2e -f docker-compose.test.yml up \
  --build --abort-on-container-exit --exit-code-from test-runner
docker compose -p clrinf-e2e -f docker-compose.test.yml down
```

Bu senaryo geçerli belge olayını Elixir HTTP girişinden NATS üzerinden
TypeScript Inspector WebSocket'ine aktarır. Servis açılmazsa veya olay
görülmezse hata verir; yerel değişkenler üzerinde simülasyonu uçtan uca
Saga doğrulaması olarak adlandırmaz.

`CLRINF_E2E_STRICT=0 bun test tests/e2e/transport.test.ts` E2E'yi açıkça
**atlar**. CI bu modu kullanmaz.

## Kimlik ve örnek iş alanları

Rust auth anahtar/config kurulumunu [Rust README](../../core/rust/README.md) açıklar.
Örnek uygulamalar [kendi dizinlerinde](../../templates/eticaret-showcase/README.md)
yer alır; çalışan çekirdek akışın önkoşulu değildir.

OTel servislerini açmak uygulamalara otomatik instrumentation eklemez.
Her dilin export/log/trace entegrasyonunu ve [yetenek matrisini](../architecture/parity-matrix.md)
ayrıca okuyun.

