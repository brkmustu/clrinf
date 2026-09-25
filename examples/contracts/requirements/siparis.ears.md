# Sipariş & Sepet Domain — EARS Gereksinimleri

**Format Standardı:** Easy Approach to Requirements Syntax (EARS)  
**Şablon:** `WHEN <tetikleyici> THE <servis> SHALL <davranış> [WHILE <koşul>]`

---

## 1. Sepete Ürün Ekleme (Happy Path)

- **REQ-SIPARIS-001:**
  **WHEN** geçerli bir `SepeteEkle.v1` komutu alındığında
  **THE** Sipariş servisi **SHALL** müşterinin sepetine ilgili ürünü ve miktarı eklemeli ve `com.clrinf.eticaret.SepetGuncellendi.v1` event'i yayınlamalı
  **WHILE** ürün katalogda mevcut ve aktif olmalı.

---

## 2. Boş Kalem veya Minimum Tutar İhlali (Error Path)

- **REQ-SIPARIS-002:**
  **WHEN** sipariş oluşturma isteğinde ürün kalemi bulunmadığında (`items` boş)
  **THE** Sipariş servisi **SHALL** işlemi reddetmeli ve `EMPTY_ORDER_ITEMS` hata kodu dönmeli.

- **REQ-SIPARIS-003:**
  **WHEN** sipariş toplam tutarı minimum sipariş eşiğinden (örn. 25.0) düşük olduğunda
  **THE** Sipariş servisi **SHALL** işlemi reddetmeli ve `MINIMUM_ORDER_AMOUNT_NOT_MET` hata kodu dönmeli.

---

## 3. Sipariş Oluşturma ve Rezervasyon Başlatma (Happy Path)

- **REQ-SIPARIS-004:**
  **WHEN** geçerli kalemler ve tutarla bir sipariş oluşturulduğunda
  **THE** Sipariş servisi **SHALL** siparişi `Pending` statüsünde kaydetmeli, `com.clrinf.eticaret.SiparisOlusturuldu.v1` event'i yayınlamalı ve Stok servisine rezervasyon isteği göndermeli.

---

## 4. Sipariş İptali ve Telafi (Compensating Path)

- **REQ-SIPARIS-005:**
  **WHEN** müşteri veya sistem tarafından bekleyen bir sipariş iptal edildiğinde
  **THE** Sipariş servisi **SHALL** sipariş statüsünü `Cancelled` yapmalı, `com.clrinf.oms.SiparisDurumuDegisti.v1` yayınlamalı ve rezerve stoğun serbest bırakılması için `ReleaseStock` komutunu tetiklemeli
  **WHILE** sipariş henüz kargolanmamış veya tamamlanmamış olmalı (`status == Pending`).
