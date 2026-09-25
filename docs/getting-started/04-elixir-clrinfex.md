# Elixir

`clrinfex` çekirdeği tagged tuple sonuçları, context ve ortak wire-format
zarflarını süreç içinde kullanabilir. Streaming/presence/NATS sunucusu ayrı
çalışma modudur.

```bash
export CLRINF_CONFORMANCE_DIR="$PWD/tests/conformance/fixtures"
cd clrinfex
mix deps.get
mix compile --warnings-as-errors
mix test --no-start --warnings-as-errors

# NATS veya HTTP sunucusu olmadan monolith örneği
CLRINF_MODE=core mix run examples/monolith.exs
```

Tam Erlang/OTP kurulumu gerekir; yalnız `erlang-core` paketi Hex için gereken
`public_key`, `ssl` ve `inets` uygulamalarını içermeyebilir.

| Streaming durumu | `/health` | Yayınlama |
|---|---|---|
| `NATS_URL=""` | `200`, `standalone` | Yalnızca yerel aboneler |
| NATS bağlı | `200`, `connected` | NATS ve yerel aboneler |
| Yapılandırılmış NATS erişilemiyor | `503`, `disconnected` | `503`, retryable standart hata |

Köprü bağlantıyı yeniden dener ve yerel aboneleri korur. Core NATS kullanır:
durable consumer, disk saklama ve kaçan mesaj replay'i sunmaz.
Domain olayları canonical envelope olarak doğrulanır; presence/signal gibi
yerel mesajlar açık yerel API üzerinden geçer.

Gerçek NATS testleri için `NATS_TEST_URL` ayarlayın. Bu adres verilmezse ilgili
entegrasyon testleri açıkça dışlanır; geçiyormuş gibi raporlanmaz.
API ve HTTP belge örneği [Elixir deposunda](../../clrinfex/README.md).
