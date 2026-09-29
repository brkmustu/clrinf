// Cedar ABAC / RBAC Kampanya ve Promosyon Politika Kuralları

// 1. Kural: Pazarlama Yetkilisi (MarketingLead) %30'a kadar kampanya tanımlayabilir
permit (
    principal in Role::"MarketingLead",
    action in [Action::"KampanyaOlustur", Action::"CreatePromotion", Action::"KampanyaDurumGuncelle"],
    resource
)
when {
    resource.discount_rate <= 0.30
};

// 2. Kural: Sistem Yöneticisi (Admin) sınırsız bütçeli ve oranda kampanya oluşturabilir ve durdurabilir
permit (
    principal in Role::"Admin",
    action in [Action::"KampanyaOlustur", Action::"CreatePromotion", Action::"KampanyaDurumGuncelle", Action::"KampanyaSil"],
    resource
);
