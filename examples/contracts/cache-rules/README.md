# Event-Driven Cache Invalidation Rules

Bu dizin, `clrinf` ekosistemindeki servis ve modüllerin yayımladığı domain olaylarına (CloudEvents) karşılık gelen bildirimsel (declarative) önbellek geçersiz kılma (cache invalidation) kurallarını içerir.

---

## Mimari Yaklaşım

- **Olay Odaklı Temizlik (Event-Driven Invalidation):** Doğrudan veritabanı veya iş katmanında ad-hoc cache temizleme kodları yazmak yerine, her domain kendi yayımladığı kanonik olaylara göre hangi önbellek kalıplarının (`pattern`) temizleneceğini beyan eder.
- **Dinamik Örüntü Çözümleme (`{data.property}`):** Kurallar içindeki `{data.xyz}` kalıpları, olayın CloudEvent `data` yükündeki ilgili alan ile çalışma zamanında dinamik olarak eşleştirilir.
- **Tenant İzolasyonu:** Önbellek anahtarları çalışma zamanı adaptöründe `tenant_id` önekiyle izole edilir.

---

## Tanımlı Kural Setleri

| Alan (Domain) | Dosya | Tetikleyen Kanonik Olaylar |
|---|---|---|
| **Stok & Envanter** | [`stok.yaml`](stok.yaml) | `StockReserved`, `StockReleased`, `StockCommitted` |
| **Ürün Kataloğu** | [`katalog.yaml`](katalog.yaml) | `FiyatGuncellendi`, `UrunYayinlandi` |
| **Sipariş Yönetimi** | [`oms.yaml`](oms.yaml) | `SiparisDurumuDegisti`, `SiparisKargoyaVerildi`, `SiparisIadeEdildi` |
| **E-Ticaret & Sepet** | [`eticaret.yaml`](eticaret.yaml) | `SepetGuncellendi`, `SiparisOlusturuldu`, `OdemeBasladi` |
| **Kampanya & İndirim** | [`kampanya.yaml`](kampanya.yaml) | `KampanyaOlusturuldu`, `KampanyaTukendi`, `KampanyaUygulandi` |
| **Kimlik & Yetki (Auth)** | [`auth.yaml`](auth.yaml) | `UserCreated`, `PolicyUpdated` |
| **Müşteri İlişkileri (CRM)** | [`crm.yaml`](crm.yaml) | `ContactCreated`, `ContactUpdated`, `DealStageChanged`, `ActivityLogged` |
