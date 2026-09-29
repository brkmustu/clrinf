// Cedar ABAC / RBAC E-Ticaret ve Katalog Politika Kuralları (clrinfrs Engine)

// 1. Kural: Sistem Yöneticisi (Admin) tüm eylemleri gerçekleştirebilir
permit (
    principal in Role::"Admin",
    action,
    resource
);

// 2. Kural: Fiyat Yöneticisi katalog fiyatlarını güncelleyebilir
permit (
    principal in Role::"PricingOfficer",
    action in [Action::"UpdatePrice", Action::"FiyatGuncelle"],
    resource
);

// 3. Kural: Satınalma Uzmanı yalnız $50.000 altındaki siparişleri verebilir
permit (
    principal in Role::"Purchaser",
    action in [Action::"Checkout", Action::"Ode", Action::"PlaceOrder"],
    resource
)
when {
    resource.amount <= 50000
};

// 4. Kural: Pazarlama Yetkilisi kupon oluşturabilir ve %20 üzeri indirim uygulayabilir
permit (
    principal in Role::"MarketingLead",
    action in [Action::"ApplyCoupon", Action::"KuponUygula"],
    resource
);

// 5. Kural: Depo Yöneticisi yalnız stok miktarını yönetebilir ve rezerve stoğu çözer
permit (
    principal in Role::"WarehouseManager",
    action in [Action::"UpdateInventory", Action::"ReleaseStock", Action::"StokSerbetBirak"],
    resource
);
