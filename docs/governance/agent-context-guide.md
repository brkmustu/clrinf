# Ajan bağlamı standardı

Her dil deposunun `agent-context.md` belgesi sürüm kontrolünde tutulur.
Belge gerçek klasörleri, API'leri ve komutları referanslamalıdır; uygulanmamış
özellikleri garantiler listesine eklememelidir.

## İçerik

1. Projenin alan-bağımsız amacı ve kapsam dışı iş alanları.
2. Çekirdek, adaptör, geliştirme aracı ve örnek sınırları.
3. Ortak sözleşme referansı ve wire-format alan eşlemeleri.
4. Gerçek derleme/test komutları ve dış servis ihtiyaçları.
5. Kalıcılık/teslim/izolasyon sınırları ve bilinen eksikler.
6. Değişiklik sahipliği ve bağımlı submodule yayın sırası.

`tenant_id` header'ı güvenilir kimlik kanıtı değildir. Bir correlation içindeki
tüm adımlar aynı idempotency işlemine indirgenmez. Monolith yerel ACID
kullanabilir. Adaptörler PostgreSQL/NATS/Cedar/Nomad'a kilitlenmez.

Çalışma dizinindeki değişikliklerle yayımlanmış gitlink sürümünü ayırın.
Alt modül değişiklikleri kendi depolarında yayımlanmadan umbrella commit'inin
temiz checkout'ta aynı davranışı sağladığını iddia etmeyin.
