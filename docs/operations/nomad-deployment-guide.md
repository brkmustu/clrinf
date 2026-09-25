# İsteğe bağlı Nomad dağıtımı

Nomad bir adaptör/dağıtım tercihi olarak kullanılabilir; clrinf'in zorunlu
çalışma zamanı değildir. Aynı çekirdek bir süreçte, systemd altında, Compose,
Kubernetes veya başka bir ortamda kullanılabilir.

Alt modüllerdeki `deploy/nomad.hcl` dosyaları başlangıç referansıdır.
İşletime almadan önce imaj sürümünü, endpoint'leri, key/secret enjeksiyonunu,
resource limitlerini ve ağ erişimini uygulamanıza göre düzenleyin.

İki replica tek başına yüksek erişilebilirlik sağlamaz: süreç belleğindeki
presence, inbox/outbox ve Inspector event listesi replica'lar arasında ortak
değildir. Kalıcı adaptör, servis keşfi, güvenilir mesajlaşma, backup/restore
ve uygun health/readiness politikası ayrıca gerekir.

Readiness bağımlı servisin çalışmadığını bildirirken liveness gereksiz restart
döngülerine neden olmamalıdır. Broker erişilemediğinde `/health` degraded
dönüyorsa bunu körlemesine tüm deployment'ı yeniden başlatma sinyali yapmayın.
