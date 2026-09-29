# Kampanya, Promosyon ve Rust Kural Motoru Mimari Rehberi

> Bu belge, vitrin uygulamasının kampanya ve promosyon kural motoru mimarisini açıklar.

Bu belge, **clrinf** ekosisteminde çoklu promosyon birleştirme kuralları (Stackability / Exclusivity), etiket/kod bazlı kısıtlar ve **Rust** ile geliştirilmiş ultra yüksek performanslı kampanya çözümleme motorunun (Promotion Constraint Engine) mimari standartlarını açıklar.

---

## 1. Birleştirilebilirlik (Stackability) Prensipleri

1. **Münhasır / Birleştirilemez Kampanyalar (`birlestirilebilir: false` - Exclusive):**
   - Tek başına uygulanır. Sepette bu kampanya varken başka hiçbir kupon veya kampanya birleştirilemez.
   - Örnek: `EXCLUSIVE_FLAS50` (%50 Flaş İndirim).

2. **Birleştirilebilir Kampanyalar (`birlestirilebilir: true` - Stackable):**
   - **Kapsam (`birlestirmeTuru`):**
     - `SERBEST`: Tüm uyumlu kampanyalarla birleşebilir.
     - `TUR_KISITLI`: Yalnızca farklı türdeki (örn. Yüzdesel + Sabit Tutar) promosyonlarla birleşebilir.
     - `OZEL_LISTE`: Yalnızca `izinliKampanyaKodlari` veya `izinliEtiketler` listesindeki kampanyalarla birleşebilir.
   - **Yasaklı Etiketler (`yasakliEtiketler`):** Birlikte çalışması engellenen kampanya grupları (örn: `["FLAS_INDIRIM"]`).
   - **Uygulama Sırası (`uygulamaSirasi`):** İndirimlerin hangi sırayla düşüleceğini belirler (1: Ürün/kategori bazlı indirim, 2: Kalan bakiye üzerinden genel sepet kuponu).

---

## 2. Ultra Yüksek Performanslı Rust Kural Motoru (`core/rust`)

- **Çözümleme Algoritması:** Graf/Kısıt tabanlı (Constraint Solver) tek geçişli değerlendirme.
- **Performans:** Sub-mikrosaniye (< 5 µs) hesaplama süresi ve sıfır GC tahsisi.
- **Uç Nokta:** `POST http://localhost:8081/kampanya/hesapla`

```json
{
  "sepet_id": "cart-100",
  "orijinal_tutar": 3000.0,
  "toplam_indirim": 1500.0,
  "odenecek_tutar": 1500.0,
  "uygulanan_kampanyalar": [
    {
      "kod": "EXCLUSIVE_FLAS50",
      "baslik": "Tek Başına Geçerli %50 Flaş İndirim",
      "tur": "YUZDESEL",
      "indirim_tutari": 1500.0,
      "uygulama_sirasi": 1
    }
  ],
  "reddedilen_kampanyalar": [
    {
      "kod": "YAZ2026",
      "sebep": "Sepette halihazırda birleştirilemez münhasır bir kampanya bulunmaktadır."
    }
  ],
  "cozumleme_suresi_mikrosaniye": 3
}
```

---

## 3. Backend İzolasyonu (`templates/eticaret-showcase/backend/`)

- `core/csharp/examples/NativeApp`: Saf benchmark harness'ı olarak izole edilmiştir.
- `templates/eticaret-showcase/backend/EticaretApi.csproj`: Stok, Katalog, ETicaret ve Kampanya modüllerini barındıran bağımsız üretim referans API'sidir.
