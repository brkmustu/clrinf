# Güvenlik ve üretime hazırlık

Bu liste üretim hazırlık kapısıdır; bütün maddelerin mevcut örneklerde
uygulandığı iddiası değildir.

- [ ] Kimlik doğrulaması token imzasını, izin verilen algoritmayı, issuer,
  audience ve expiry'yi doğrular. Demo kullanıcıları açıkça opt-in'dir.
- [ ] Private signing key yalnızca secret mount/secret manager üzerinden gelir;
  JWKS yalnızca public parametreleri içerir. Rotasyon ve iptal planı vardır.
- [ ] Tenant, doğrulanmış principal ile eşleşir. Header'da tenant bulunması
  yetkilendirme kanıtı olarak kullanılmaz.
- [ ] Depolama sorguları ve benzersiz anahtarlar tenant kapsamlıdır.
  PostgreSQL RLS kullanılıyorsa uygulama rolü owner/BYPASSRLS değildir;
  `USING` ve `WITH CHECK`, bağlantı havuzu context temizliği de doğrulanır.
- [ ] Domain hataları standart zarfla döner; stack trace, key, token ve
  bağlantı URL'sindeki sırlar yanıta/loga sızmaz.
- [ ] Idempotency kapsamı tenant + operation + key; payload çakışması açıktır.
- [ ] Public HTTP, broker ve yönetim yüzeylerinde TLS, auth, rate limit,
  erişim kontrolü ve payload boyutu sınırı sağlanır.
- [ ] Inspector sadece güvenilen geliştirme ortamına açılır; olay yüklerindeki
  kişisel/hassas veriler maskelenir. Bellek sınırı erişim kontrolü değildir.
- [ ] İmajlar/bağımlılıklar sabittir; sabitleme güvenlik güncelliği anlamına gelmez.
  Bağımlılık denetimi ve güncelleme sahipliği tanımlıdır.
- [ ] Yedekleme/geri yükleme ve tenantlar arası veri sızıntısı senaryoları uygulanır.

## Bilinen bağımlılık riski

2026-09-09 incelemesinde Hex'in en güncel kararlı `cowlib` sürümü `2.20.0`
olmasına rağmen `EEF-CVE-2026-43971`, `EEF-CVE-2026-43966` ve
`EEF-CVE-2026-43969` bildirimleri vardı. `mix hex.audit` sonucu göz ardı edilmemeli.
Bu belge bir istismar doğrulaması değildir; upstream yama/uygulanabilirlik
değerlendirmesi tamamlanmadan “güvenlik sorunları kapandı” denemez.
