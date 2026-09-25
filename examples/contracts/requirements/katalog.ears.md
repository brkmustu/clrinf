# Katalog & Ürün Yönetimi Domain — EARS Gereksinimleri

**Format Standardı:** Easy Approach to Requirements Syntax (EARS)  
**Şablon:** `WHEN <tetikleyici> THE <servis> SHALL <davranış> [WHILE <koşul>]`

---

## 1. Yeni Ürün Tanımlama (Happy Path)

- **REQ-KATALOG-001:**
  **WHEN** geçerli SKU, isim ve sıfırdan büyük fiyat bilgisiyle bir `com.clrinf.katalog.UrunOlustur.v1` komutu alındığında
  **THE** Katalog servisi **SHALL** ürünü katalog deposuna kaydetmeli ve `com.clrinf.katalog.UrunYayinlandi.v1` event'i yayınlamalı
  **WHILE** `sku` ilgili `tenant_id` içerisinde tekil (unique) olmalı.

---

## 2. Negatif veya Sıfır Fiyat Girişi (Error Path)

- **REQ-KATALOG-002:**
  **WHEN** bir `com.clrinf.katalog.UrunOlustur.v1` veya `com.clrinf.katalog.UrunFiyatGuncelle.v1` komutunda fiyat <= 0 gönderildiğinde
  **THE** Katalog servisi **SHALL** işlemi reddetmeli ve `INVALID_PRODUCT_PRICE` hata koduyla Result hatası dönmeli.

---

## 3. Ürün Fiyat Güncelleme (Happy Path)

- **REQ-KATALOG-003:**
  **WHEN** mevcut bir ürün için geçerli pozitif fiyat içeren `com.clrinf.katalog.UrunFiyatGuncelle.v1` komutu alındığında
  **THE** Katalog servisi **SHALL** ürün fiyatını güncellemeli ve `com.clrinf.katalog.FiyatGuncellendi.v1` event'i yayınlamalı
  **WHILE** ürün ilgili `tenant_id` altında mevcut ve aktif olmalı.

---

## 4. Katalog Listeleme ve Detay Sorgulama

- **REQ-KATALOG-004:**
  **WHEN** bir `KatalogListesi` veya `UrunDetayi` sorgusu alındığında
  **THE** Katalog servisi **SHALL** yalnızca talepte bulunan kullanıcının `tenant_id` değerine ait aktif ürünleri dönmeli.
