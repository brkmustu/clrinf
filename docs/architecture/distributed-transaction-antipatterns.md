# Yerel işlemler, dağıtık tutarlılık ve Saga

clrinf'in [anayasası](../../clrinf-contracts/constitution.md) servisler arası
2PC/XA koordinasyonunu varsayılan çözüm olarak kullanmaz. Bu, monolith içinde
tek veritabanıyla yerel ACID işlemi yapmayı yasaklamaz.

## Önce doğru işlem sınırı

Tek uygulama/veritabanındaki belge onayı ile denetim kaydı aynı işlemde
yazılabilir. Bu işlem dışarıya bir olay çıkaracaksa outbox kaydı da aynı
işleme katılır. Ağ yayını başarılı olsa da yerel commit başarısız olabilir;
bu nedenle veritabanı yazısını ve ağ yayınını bağımsız iki çağrı olarak
yapmak güvenilir bir commit protokolü değildir.

Bağımsız servisler arasında tutarlılık ihtiyacında önce sınırların gerçekten
ayrı olması gerekip gerekmediğini değerlendirin. Her iş akışı Saga gerektirmez.

## Saga gerekiyorsa

Belge yayımlama sürecinde rezervasyon, dış servise bildirim ve teslim onayı
ayrı adımlarsa her adımın kalıcı durumu, zaman aşımı ve yeniden deneme politikası
olmalıdır. Geri alınabilir adımlar için telafi tanımlanır. Geri alınamaz dış
etkiler için aynı etkinin yinelenmesini önleyen anahtar ve gerektiğinde manuel
uzlaştırma gerekir; her adımın matematiksel bir tersi olduğu varsayılmaz.

Bir olayın yeniden teslimi, iş etkisinin tekrar uygulanmasına yol açmamalıdır.
Anahtar `tenant + consumer/operation + message_id` gibi açık bir kapsama sahiptir;
yalnızca `correlation_id` kullanmak sonraki geçerli adımları yanlışlıkla engeller.

NATS, HTTP veya süreç içi dispatch adaptör seçimidir. Mevcut canlı NATS
köprüsünden kalıcı Saga yürütücüsü, sıralama veya replay garantisi çıkarılamaz.
Uygulama bu davranışları ayrıca sağlamalıdır.

[Dayanıklılık rehberi](reliability.md) kalıcılık ve yeniden teslim sınırlarını açıklar.
