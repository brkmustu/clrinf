# Ödeme & Tahsilat Domain — EARS Gereksinimleri

**Format Standardı:** Easy Approach to Requirements Syntax (EARS)  
**Şablon:** `WHEN <tetikleyici> THE <servis> SHALL <davranış> [WHILE <koşul>]`

---

## 1. Ödeme Başlatma ve Tahsilat (Happy Path)

- **REQ-ODEME-001:**
  **WHEN** geçerli bir sipariş için `com.clrinf.eticaret.Ode.v1` komutu alındığında
  **THE** Ödeme servisi **SHALL** tahsilat sürecini başlatmalı ve `com.clrinf.eticaret.OdemeBasladi.v1` event'i yayınlamalı.

- **REQ-ODEME-002:**
  **WHEN** sanal POS veya ödeme sağlayıcısından başarılı provizyon onayı alındığında
  **THE** Ödeme servisi **SHALL** `com.clrinf.payment.PaymentConfirmed.v1` event'i yayınlayarak sipariş kesinleştirme ve kalıcı stok düşümünü (StockCommitted) tetiklemeli.

---

## 2. Ödeme Başarısızlığı ve Telafi Tetikleme (Error Path)

- **REQ-ODEME-003:**
  **WHEN** kart bakiyesi yetersizliği, red veya zaman aşımı nedeniyle ödeme başarısız olduğunda
  **THE** Ödeme servisi **SHALL** `com.clrinf.eticaret.SiparisTelafiBekliyor.v1` event'i yayınlayarak rezerve edilmiş stoğun serbest bırakılmasını (StockReleased) sağlamalıdır.
