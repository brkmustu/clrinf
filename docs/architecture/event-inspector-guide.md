# Event Inspector

Inspector, TypeScript çekirdeğinden ayrı bir geliştirme aracıdır. Çekirdeği
import etmek HTTP sunucusu veya NATS bağlantısı başlatmaz.

```text
Üretici (herhangi bir dil)
  -> NATS Core veya HTTP ingest
  -> Inspector'ın sınırlı bellek kaydı
  -> WebSocket / geliştirme arayüzü
```

Olaylar ortak zarfla doğrulanır. Hata ve telafi durumu event adında `failed`
veya `released` sözcüğü arayarak değil, açık `is_error` / `is_compensation`
alanlarıyla taşınır. Correlation görünümü izlemeye yarar; mesaj sırası veya
işlem tamamlanması garantisi değildir.

HTTP `/api/events`, WebSocket `/ws`, replay ve waterfall yolları geliştirme
amaçlıdır. Replay'in iş komutu olarak tekrar broker'a gönderildiği varsayılmaz.
Bellek kapasitesi veya yavaş istemci sınırları nedeniyle olaylar atılabilir;
Inspector denetim kayıtlarının kalıcı kaynağı değildir.

```bash
docker compose --profile tools up -d --build
```

Compose host portlarını loopback'e bağlar; container içinde dış dinleme,
`INSPECTOR_ALLOW_UNAUTHENTICATED=true` ile açıkça geliştirme modundadır.
Üretime açmadan önce auth, tenant yetkilendirmesi, yük filtreleme ve retention
politikası ayrıca gerekir.

NATS yapılandırılmışken `/health` içindeki `nats` alanını kontrol edin:
`connected`, `disconnected` veya bağlantı kullanılmıyorsa `disabled`.
