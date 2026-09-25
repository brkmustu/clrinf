# Auth & Yetkilendirme Domain — EARS Gereksinimleri

**Format Standardı:** Easy Approach to Requirements Syntax (EARS)

---

## 1. Kullanıcı Girişi ve Token Üretimi

- **REQ-AUTH-001:**
  **WHEN** geçerli kimlik bilgileriyle bir `AuthenticateUserCommandV1` alındığında
  **THE** Auth servisi **SHALL** `tenant_id`, roller ve yetkileri içeren RS256 imzalı JWT access token ve refresh token üretmeli.

---

## 2. Kullanıcı Oluşturma

- **REQ-AUTH-002:**
  **WHEN** yeni bir kullanıcı hesabı başarıyla kaydedildiğinde
  **THE** Auth servisi **SHALL** veritabanına kullanıcıyı kaydetmeli ve `com.clrinf.auth.UserCreated.v1` event'i yayınlamalı.

---

## 3. Politika Güncelleme

- **REQ-AUTH-003:**
  **WHEN** bir Cedar/ABAC güvenlik politikası yönetici tarafından güncellendiğinde
  **THE** Auth servisi **SHALL** policy store'u güncellemeli ve `com.clrinf.auth.PolicyUpdated.v1` event'i ile tüm servislerin yetki önbelleğini invalide etmeli.

---

## 4. E-Ticaret Kiracı İzolasyonu ve Rol Yetkilendirmesi

- **REQ-AUTH-004:**
  **WHEN** herhangi bir katalog, stok veya sipariş işlemi gerçekleştirilmek istendiğinde
  **THE** Yetkilendirme servisi **SHALL** işlem sahibinin `tenant_id` bilgisinin hedef kaynakla eşleştiğini doğrulamalı ve kullanıcı rolünü (`Customer`, `CatalogManager`, `StoreAdmin`) kontrol etmelidir.

