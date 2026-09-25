# Katalog (PIM) Alan Mimari Rehberi

> Bu belge, vitrin uygulamasının katalog ve PIM tasarım notlarını içerir.

Bu belge, **clrinf** ekosisteminde Ürün Bilgi Yönetimi (Product Information Management - PIM) ve Katalog etki alanının tasarım prensiplerini ve sözleşme standartlarını açıklar.

---

## 1. Servis ve Öbek Sınırları (Bounded Context)

Katalog etki alanı, ürünlerin temel meta verilerini, kategori ağaçlarını, varyantlarını ve dinamik fiyatlandırma kurallarını yönetir:

- **`UrunObegi` (Kök Öbek / Aggregate Root):** SKU bazında ürünün yaşam döngüsünü (Oluşturuldu, Fiyatı Güncellendi, Yayına Alındı) kontrol eder.
- **CQRS Okuma ve Yazma Ayrımı:**
  - **Komut Modeli:** `UrunOlusturKomut`, `UrunFiyatGuncelleKomut` (Yüksek tutarlılık, `DomainResult<T>`).
  - **Sorgu Modeli:** `KatalogListesiSorgula`, `UrunDetayiSorgula` (Sayfalama, arama ve Stok servisiyle anlık zenginleştirme).

---

## 2. Cedar ABAC ile Fiyat Güvenliği

Ürün fiyatlarının değiştirilmesi finansal ve operasyonel risk taşıdığı için Rust Cedar karar motoru ile korunur:

```cedar
permit (
    principal in Role::"PricingOfficer",
    action in [Action::"UpdatePrice", Action::"FiyatGuncelle"],
    resource
);
```

Yetkisiz roller (örn. Purchaser veya WarehouseManager) fiyat güncellemeye çalıştığında komut pipeline seviyesinde reddedilir (`DomainResult.Failure("CEDAR_RET", ...)`).
