# Sipariş Yönetimi (OMS) ve Yaşam Döngüsü Mimari Rehberi

> Bu yaşam döngüsü yalnızca örnek uygulamaya aittir. clrinf bir OMS ürünü
> sunmaz; aşağıdaki akışlar çekirdek yetenek veya hazır üretim entegrasyonu değildir.

Bu belge, **clrinf** ekosisteminde sipariş yaşam döngüsü (Order Lifecycle), deterministik durum makinesi (State Machine), iptal, iade / RMA (Return Merchandise Authorization) ve kargo takip süreçlerinin mimarisini açıklar.

---

## 1. Sipariş Durum Makinesi (State Machine)

Siparişler `OdemeSaga` ile kesinleştikten sonra aşağıdaki deterministik akıştan geçer:

```
[Onaylandı] ───────────────▶ [İptal Edildi] (Stok otomatik envantere iade edilir)
     │
     ▼
[Hazırlanıyor] (Depo toplama listesi / Pick-list)
     │
     ▼
[Paketlendi] (Desi ve koli barkodu)
     │
     ▼
 [Kargoda] (Kargo takip no: TR-KRG-XXXXXX)
     │
     ▼
[Teslim Edildi] ───────────▶ [İade Talebi / RMA] ──▶ [İade Tamamlandı] (Stok iadesi)
     │
     ▼
[Tamamlandı]
```

### 1.1. Durum Tablosu

| Durum | Değer | Açıklama |
|---|---|---|
| `Onaylandi` | `0` | OdemeSaga tamamlandı, sipariş kaydı açıldı |
| `Hazirlaniyor` | `1` | Depo personeli ürünleri raftan topluyor |
| `Paketlendi` | `2` | Ürünler paketlendi, koli barkodu basıldı |
| `Kargoda` | `3` | Kuryeye teslim edildi, takip no üretildi |
| `TeslimEdildi` | `4` | Müşteriye teslim edildi |
| `Tamamlandi` | `5` | Yasal iade süresi doldu, sipariş kapandı |
| `IptalEdildi` | `6` | Sipariş iptal edildi, tahsisli stok stoğa döndü |
| `IadeTalebi` | `7` | Müşteri RMA iade talebi oluşturdu |
| `IadeTamamlandi` | `8` | İade depoya girdi, bedel iade edildi |

---

## 2. Cedar ABAC Yetkilendirme Matrisi

- **`WarehouseManager` (Depo Yöneticisi):** `SiparisHazirla`, `SiparisPaketle`, `SiparisKargola`, `IadeOnayla`.
- **`Purchaser` (Satınalma / Müşteri):** `Checkout`, `SiparisIptal`, `IadeTalebi`.
- **`Admin`:** Tüm durumlara ve operasyonlara tam yetkili.

---

## 3. Uç Noktalar (REST API)

| HTTP Metot | Yol | Açıklama |
|---|---|---|
| `GET` | `/api/siparis` | Kullanıcıya/duruma göre sipariş listesi |
| `GET` | `/api/siparis/{id}` | Sipariş detayları, kargo ve durum tarihçesi |
| `PUT` | `/api/siparis/{id}/durum` | Durum makinesini ilerletme |
| `POST` | `/api/siparis/{id}/iptal` | Siparişi iptal etme ve stok telafisi |
| `POST` | `/api/siparis/{id}/iade` | RMA iade talebi oluşturma ve tamamlama |
