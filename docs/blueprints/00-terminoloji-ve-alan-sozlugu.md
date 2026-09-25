# Terminoloji ve sınırlar

Türkçe belgelerde aşağıdaki karşılıkları kullanırız. Genel API'ler dilin
alışılmış adlandırmasını izler; iş alanı örneklerinin Türkçe adları wire-format
uyumluluğu nedeniyle topluca yeniden adlandırılmaz.

| Kavram | Karşılık | Sınır |
|---|---|---|
| Aggregate / aggregate root | Öbek / kök öbek | Yerel tutarlılık sınırı; global statik sözlük demek değildir |
| Entity | Varlık | Kimliği ve yaşam döngüsü olan alan nesnesi |
| Value object | Değer nesnesi | Değere göre eşitlik |
| Command / query | Komut / sorgu | Yazma niyeti / okuma isteği |
| Dispatcher | Dağıtıcı | Süreç içi yönlendirme; mesaj aracısı olmak zorunda değil |
| Domain event | Alan olayı | İşlem sonrasında gerçekleşmiş olgu; taşıma adaptörü ayrı |
| State store | Durum haznesi | Bellekte veya kalıcı olabilir; özellik açıkça belirtilir |
| Adapter / port | Adaptör / arayüz | Teknolojiye özgü uygulama / çekirdek beklentisi |
| Outbox / inbox | Giden / işlenen mesaj kaydı | İşlem ve yeniden teslim protokolünün parçası |
| Saga | Telafili iş akışı | Dağıtık işlem değil, açıkça modellenmiş uzun süreli iş süreci |

`correlation_id` bir iş akışını ilişkilendirir; `causation_id` doğrudan tetikleyiciyi
belirtir; olay `id`'si tek olayın kimliğidir. Aynı correlation altında çok sayıda
geçerli işlem olabilir. İdempotency kapsamı en az tenant + operasyon + anahtar
olmalıdır.

Sözleşme alanlarında HTTP/context ve hata zarfları `snake_case`, CloudEvents
uzantıları `tenantid`, `correlationid`, `causationid` kullanır. Bu iki biçim
adaptör sınırında açıkça eşlenir.
