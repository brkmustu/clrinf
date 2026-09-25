# Stok & Satınalma Domain — EARS Gereksinimleri

**Format Standardı:** Easy Approach to Requirements Syntax (EARS)
**Şablon:** `WHEN <tetikleyici> THE <servis> SHALL <davranış> [WHILE <koşul>]`

---

## 1. Stok Rezervasyonu (Happy Path)

- **REQ-STOK-001:**
  **WHEN** bir `com.clrinf.order.OrderCreated.v1` event'i alındığında
  **THE** Stok&Satınalma servisi **SHALL** talep edilen miktar kadar stok rezervasyonu oluşturmalı ve `com.clrinf.stok.StockReserved.v1` event'i yayınlamalı
  **WHILE** ilgili `tenant_id` için depoda yeterli stok mevcut.

---

## 2. Stok Yetersizliği (Compensating / Error Path)

- **REQ-STOK-002:**
  **WHEN** bir `com.clrinf.order.OrderCreated.v1` event'i alındığında
  **THE** Stok&Satınalma servisi **SHALL** rezervasyonu reddetmeli ve `StandardErrorEnvelope` ile `STOCK_INSUFFICIENT` hatası dönmeli/yayınlamalı
  **WHILE** talep edilen miktar mevcut stok bakiyesinden fazla.

---

## 3. Ödeme Başarısızlığı / Zaman Aşımı (Compensating Action)

- **REQ-STOK-003:**
  **WHEN** bir `com.clrinf.payment.PaymentFailed.v1` veya rezervasyon zaman aşımı sinyali alındığında
  **THE** Stok&Satınalma servisi **SHALL** rezerve edilen stoğu serbest bırakmalı ve `com.clrinf.stok.StockReleased.v1` event'i yayınlamalı.

---

## 4. Sipariş Onayı (Commit)

- **REQ-STOK-004:**
  **WHEN** bir `com.clrinf.payment.PaymentConfirmed.v1` event'i alındığında
  **THE** Stok&Satınalma servisi **SHALL** rezervasyonu kalıcı stok düşümüne çevirmeli ve `com.clrinf.stok.StockCommitted.v1` event'i yayınlamalı.
