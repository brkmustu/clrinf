# Modüler monolith ile kullanım

Tek dilin çekirdeğini kullanarak başlayın. Context açıkça geçirilir; uygulama
komutları süreç içi işleyicilere yönlendirilir. İş verisi tek veritabanındaysa
yerel ACID işlem kullanılabilir. NATS veya ayrı auth sunucusu açmanız gerekmez.

## İzlenecek yol

1. Alan modelinizi ve tenant sınırınızı belirleyin; örneğin belge onaylama.
2. Girişte context'i doğrulayın ve mevcut kimlik sisteminizle eşleştirin.
3. Domain sonucunu dilin Result/tuple yapısıyla taşıyın; HTTP hata zarfını
   yalnızca HTTP adaptöründe oluşturun.
4. Süreç içi event/dispatcher ile modülleri ayırın. Bellek adaptörünün restart
   sonrası state'i korumadığını kabul edin.
5. Dışarıya olay çıkacaksa iş yazısı ile aynı transaction'da outbox kullanın;
   yalnızca bu sınırda bir mesajlaşma adaptörü ekleyin.

Çalışan dil örnekleri için [Rust](../../core/rust/README.md),
[C#](../../core/csharp/README.md), [TypeScript](../../core/typescript/README.md) ve
[Elixir](../../core/elixir/README.md) dizinlerinin örnek bölümlerine bakın.
Bu örnekler dağıtık servis zorunluluğu olmadan çekirdeğin kullanılmasını gösterir.

Servislere bölme sonraki bir karardır. Aynı sözleşmeleri korumak geçişi
kolaylaştırır ancak ağ hataları, sıralama ve kalıcılık problemlerini ortadan kaldırmaz.
