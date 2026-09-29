// ==============================================================================
// CLRINF CEDAR POLİTİKALARI — SİPARİŞ YÖNETİMİ & OMS İZİNLERİ
// ==============================================================================

// 1. Admin tüm OMS operasyonlarını yürütebilir
permit (
    principal in Role::"Admin",
    action in [
        Action::"SiparisHazirla",
        Action::"SiparisPaketle",
        Action::"SiparisKargola",
        Action::"SiparisTeslimEt",
        Action::"SiparisIptal",
        Action::"IadeOnayla"
    ],
    resource
);

// 2. Depo Yetkilisi (WarehouseManager) hazırlama, paketleme, kargolama ve iade kabul işlemlerini yapabilir
permit (
    principal in Role::"WarehouseManager",
    action in [
        Action::"SiparisHazirla",
        Action::"SiparisPaketle",
        Action::"SiparisKargola",
        Action::"IadeOnayla"
    ],
    resource
);

// 3. Satınalma / Müşteri (Purchaser) sipariş iptal ve iade talebi oluşturabilir
permit (
    principal in Role::"Purchaser",
    action in [
        Action::"SiparisIptal",
        Action::"IadeTalebi"
    ],
    resource
);
