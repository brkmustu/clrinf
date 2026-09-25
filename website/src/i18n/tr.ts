export const tr = {
  nav: {
    home: "Ana Sayfa",
    philosophy: "Felsefe",
    problems: "Çözülen Problemler",
    modules: "Modül Kataloğu",
    simulator: "Topoloji Simülatörü",
    codeMatrix: "Çok Dilli Kod",
    docs: "Dokümantasyon",
    github: "GitHub",
    version: "v0.1.0",
  },
  hero: {
    badge: "YAPAY ZEKA ÇAĞINDA MİMARİ GÜVENCE",
    titleStart: "Yapay Zeka Ajanları İçin",
    titleHighlight: "Mimari Çerçeve",
    titleEnd: "ve Çok Dilli Yazılım Omurgası",
    subtitle: "AI kodlama ajanlarına (Claude, Cursor, Antigravity) kesin mimari tasarım sınırları çizerek; üretilen kodların ilk günden itibaren temiz, sürdürülebilir ve kurumsal kalitede kalmasını sağlayan sözleşme öncelikli çatı.",
    ctaQuickstart: "Hızlı Başlangıç",
    ctaSimulator: "Topoloji Simülatörü",
    ctaDocs: "Dokümantasyonu İncele",
  },
  terminal: {
    tabs: {
      lint: "Mimari Denetim (Linter)",
      topology: "Topoloji Doğrulama",
      catalog: "Yerleşik Katalog (clrinf catalog)",
      mcp: "Meta MCP Sunucusu",
      pubsub: "Mekanik Pub/Sub",
    },
    lintOutput: `$ clrinf-codegen lint --lang all
🔍 [csharp] Roslyn AST denetimi çalışıyor...
   ✓ ARCH001: Domain katmanı harici framework bağımlılığı içermiyor.
   ✓ ARCH002: Tüm kural sınıfları IBusinessRule<T> uyguluyor.
   ✓ ARCH003: API Controller'a doğrudan DbContext sızması engellendi.
   ✓ ARCH004: CQRS istekleri IRequest<T> ile uyumlu.
🔍 [rust] Syn AST statik analizi çalışıyor...
   ✓ RUST_ARCH001: Sıfır-panik güvencesi (unwrap/expect/panic saptanmadı).
🔍 [typescript] TS Compiler API analizi çalışıyor...
   ✓ ARCH_TS_001: İş kurallarında ham throw engellendi, monadic Result devrede.
   ✓ ARCH_TS_003: MediatR taklidi sınıflar engellendi, saf fonksiyonlar aktif.
✅ TÜM DİLLERDE MİMARİ KURALLAR BAŞARIYLA DOĞRULANDI.`,
    topologyOutput: `$ clrinf-codegen topology check --schema-dir clrinf-contracts/schemas
🌐 Cross-Service Pub/Sub Topology Map:
  📦 Service: order-service
     Publishes:
       ➔ ordering.order.placed.v1
  📦 Service: billing-service
     Subscribes:
       📥 ordering.order.placed.v1
──────────────────────────────────────────
Diagnostics: 0 fatal issue(s) detected.
✅ Topoloji doğrulandı: Ölü olay veya yetim abone saptanmadı.`,
    catalogOutput: `$ clrinf-codegen catalog
📦 clrinf Yerleşik Modül Kataloğu (Built-In Module Catalog)

🏢 Domain Modülleri & Paketler (Domain Features & Suites):
  • crm          — CRM Full Suite                   [raw ✅ | wired ✅]
    Tanım: Tam teşekküllü B2B CRM paketi (Deals + Contacts + Activities + birleşik Cedar koruması).
  • deals        — CRM Deals / Sales Pipeline       [raw ✅ | wired ✅]
  • contacts     — CRM Contacts / Customers         [raw ✅ | wired ✅]
  • activities   — CRM Activities / Interactions    [raw ✅ | wired ✅]

⚙️  Altyapı Modülleri (Cross-Cutting Concerns):
  • authz        — Multi-Tenant Cedar Authorization [raw ✅ | wired ✅]
  • caching      — Caching / In-Memory & Redis      [raw ✅ | wired ✅]
  • logging      — Structured Logging               [raw ✅ | wired ✅]
  • transaction  — Transaction & Unit of Work       [raw ✅ | wired ✅]
  • idempotency  — Idempotency Store                [raw ✅ | wired ✅]
  • outbox       — Transactional Outbox             [raw ✅ | wired ✅]
💡 İpucu: Herhangi bir projeye ham veya bağlı aktarmak için 'clrinf module adopt <MODÜL>' kullanın.`,
    mcpOutput: `$ clrinf-codegen mcp
[clrinf-meta-mcp] Model Context Protocol JSON-RPC 2.0 stdio server hazır...
Tools Kayıtlı:
  • clrinf_list_catalog (yerleşik modülleri ve domain paketlerini listele)
  • clrinf_adopt_module (raw veya wired modda projeye aktar)
  • clrinf_module_manage (caching, logging, authz, outbox)
  • clrinf_add_domain_module (Cedar kurallı domain scaffolding)
  • clrinf_add_entity (CRUD entity & repo portu)
  • clrinf_inspect_ecosystem & lint_architecture
🤖 Claude / Cursor / Antigravity ajanları ekosistemi otonom olarak yönetebilir.`,
    pubsubOutput: `$ clrinf-codegen generate-pubsub --lang all --output ./generated
📡 CloudEvent Publisher & Idempotent Subscriber kabukları üretiliyor...
  [Rust Publisher]  generated/rust/order_placed_publisher.rs (OutboxStore atomik)
  [Rust Subscriber] generated/rust/order_placed_subscriber.rs (Idempotency korumalı)
  [C# Publisher]    generated/csharp/OrderPlacedPublisher.cs (IOutboxStore atomik)
  [C# Subscriber]   generated/csharp/OrderPlacedSubscriber.cs (IIdempotencyStore)
  [TS Publisher]    generated/typescript/order-placed-publisher.ts
  [TS Subscriber]   generated/typescript/order-placed-subscriber.ts
🎉 %100 mekanik zarflama tamamlandı. İş mantığı gövdeleri incelenmek üzere hazır.`,
  },
  problems: {
    tag: "TEMEL PROBLEM & ÇÖZÜMLER",
    title: "Yapay Zeka Destekli Geliştirmede Mimari Bozulmayı Nasıl Engelliyoruz?",
    subtitle: "AI ajanları hızlı kod yazar; ancak mimari sınırlar ve tasarım anayasası olmadan kod tabanınız hızla teknik borç batağına sürüklenir. clrinf bu kaosu 8 temel mekanizmayla çözer:",
    items: [
      {
        icon: "🤖",
        title: "Kontrolsüz AI Kod Üretimi & Spagetti Tehlikesi",
        desc: "Yapay zeka modelleri bağlamsız kaldığında rastgele desenler uydurur, katmanları birbirine bağlar ve sürdürülemez kod yığınları üretir.",
        solution: "Mimari şablonlar üzerinden kontrollü kod üretimi, Meta MCP araçları ve linter denetimiyle ajanın uydurma desenler yerine doğrulanmış kalıplarla çalışması hedeflenir.",
      },
      {
        icon: "📦",
        title: "Monolitik Kilitlenme ve Dış Paket Dayatmaları",
        desc: "Var olan bir projeye sadece bir CRM modülü veya Cedar yetkilendirme eklemek istediğinizde, devasa kütüphane bağımlılıkları ve karmaşık kurulumlar projeyi kilitler.",
        solution: "clrinf module adopt motoru: İster sıfır harici bağımlılıkla saf kod (raw), ister dilin doğal DI ve modül ağacına kablolanmış (wired) olarak tek komutla aktarım.",
      },
      {
        icon: "🛡️",
        title: "Çok Kiracılı Veri Sızıntısı & İzolasyon İhlali",
        desc: "Tenant ID kontrolleri kod içine manuel serpiştirildiğinde geliştiriciler veya AI ajanları 'where TenantId == ...' filtresini bir sorguda unutur ve büyük güvenlik açığı doğar.",
        solution: "Amazon Cedar tabanlı varsayılan çok kiracılı güvenlik politikası (policies/*.cedar): context.tenant_id != resource.tenant_id durumunda otomatik mutlak red (forbid).",
      },
      {
        icon: "📐",
        title: "Çok Dilli Sistemlerde Sözleşme Kayması (Contract Drift)",
        desc: "C# geliştiricisi modeli günceller, Rust ve TS ekipleri manuel DTO yazarken bir alanı unutur; serileştirme hataları başlar.",
        solution: "CloudEvents 1.0 ve JSON Schema Draft 2020-12 kanonik sözleşmelerinden tüm diller için %100 hatasız tipler tek komutla üretilir.",
      },
      {
        icon: "📡",
        title: "Mikroservislerde Sessiz Veri & Olay Kaybı",
        desc: "Bir servis olay üretir ama kimse dinlemez (Dead Event); ya da bir servis dinler ama kimse üretmez. Hata ancak üretimde fatura kesilmeyince fark edilir.",
        solution: "clrinf-codegen topology check motoru, ölü olayları ve yetim tüketicileri daha CI aşamasında tespit edip süreci güvenle durdurur.",
      },
      {
        icon: "⚡",
        title: "Diller Arası Zorlama Framework Dayatmaları",
        desc: "C#'taki MediatR sınıf yapısını Rust veya TypeScript'e zorla kopyalamak o dillerin doğasını bozar ve yapay bir karmaşıklık yaratır.",
        solution: "Her dil kendi doğallığında yaşar: C#'ta native FrozenDictionary dispatcher veya MediatR, Rust'ta zero-panic BusinessRule, TS'te fonksiyonel pipeRules & Result monadı.",
      },
      {
        icon: "🔌",
        title: "Mesajlaşma & Bulut Sağlayıcı Bağımlılığı (Lock-in)",
        desc: "Çoğu altyapı belirli bir mesaj aracısını (Kafka, NATS, AWS SQS) zorunlu kılar; yerel test yazmak ve monolith başlatmak imkansızlaşır.",
        solution: "OutboxStore ve IdempotencyStore portları sayesinde transport-agnostic yapı: İster in-memory modüler monolith, ister dağıtık sistem.",
      },
      {
        icon: "🛡️",
        title: "Mimari Katman İhlalleri & Gizli Bağımlılıklar",
        desc: "Geliştiriciler veya AI; domain katmanına veritabanı bağlar, controller içine ORM sızdırır ya da servis içinde kontrolsüz panik fırlatır.",
        solution: "Roslyn (ARCH001-004), Syn (RUST_ARCH001) ve TS AST (ARCH_TS_001) denetleyicileri kuralları derleme ve analiz zamanında zorunlu kılar.",
      },
    ],
  },
  moduleStudio: {
    tag: "SIFIR SÜRTÜNMELİ MODÜL NAKLİ",
    title: "Yerleşik Modül Kataloğu ve Adaptasyon Stüdyosu",
    subtitle: "clrinf ekosisteminde hazır olarak sunduğumuz modülleri dilediğiniz projeye (.csproj, Cargo.toml, package.json) tek komutla aktarın. İster sıfır bağımlılıkla (raw), ister tüm yapımızla kablolanmış (wired) olarak!",
    catalogBadge: "RESMİ YERLEŞİK KATALOG",
    selectModule: "Nakledilecek Modülü Seçin:",
    selectMode: "Adaptasyon Modu:",
    modes: {
      raw: {
        title: "Ham (Raw / Zero-Dependency)",
        desc: "Sıfır harici paket/kütüphane bağımlılığı. Kendi içinde çalışan sözleşmeler (OperationClaim), izole domain mantığı ve Cedar politikası.",
      },
      wired: {
        title: "Kablolanmış (Wired / Full Entegrasyon)",
        desc: "clrinf'in tüm yeteneklerini barındırır. DI servis kaydı, modül ağacı, barrel export ve Cedar yetkilendirme muhafızları otomatik bağlanır.",
      },
    },
    selectTarget: "Hedef Proje Türü:",
    targets: {
      csharp: "C# (.csproj)",
      rust: "Rust (Cargo.toml)",
      ts: "TypeScript (package.json)",
    },
    cliCommandLabel: "Üretilen Adaptasyon Komutu",
    generatedFilesLabel: "Oluşturulan / Bağlanan Dosyalar",
    codePreviewTabs: {
      code: "Modül Kodu",
      cedar: "Cedar Güvenlik Politikası",
      wiring: "Kablolama / DI",
    },
    copyBtn: "Komutu Kopyala",
    copiedBtn: "Kopyalandı!",
  },
  simulator: {
    tag: "CANLI ETKİLEŞİMLİ DENEYİM",
    title: "Çapraz Servis Topoloji Simülatörü",
    subtitle: "clrinf'in dağıtık olay güvenliği motorunun ölü olayları, yetim aboneleri ve versiyon geçiş zincirlerini nasıl anında yakaladığını test edin.",
    scenarios: {
      valid: "Normal Akış (Doğrulanmış)",
      deadEvent: "Ölü Olay (Dead Event)",
      orphanSub: "Yetim Tüketici (Orphan Sub)",
      missingUpcaster: "Eksik Upcaster (v1 ➔ v2)",
    },
    statusTitle: "Topoloji Tanı Durumu",
    statusMessages: {
      valid: "✅ TOPOLOJİ DOĞRULANDI: 'order-service' tarafından yayınlanan 'ordering.order.placed.v1' olayı, 'billing-service' tarafından eksiksiz dinleniyor. Dağıtık veri kaybı riski sıfır.",
      deadEvent: "⚠️ [TOPOLOGY_DEAD_EVENT] UYARI: 'payment-service' tarafından yayınlanan 'payment.completed.v1' olayı ekosistemde hiçbir servis tarafından dinlenmiyor! Olay uzay boşluğunda kaybolacak.",
      orphanSub: "❌ [TOPOLOGY_ORPHAN_SUBSCRIBER] KRİTİK HATA: 'notification-service' tarafından dinlenen 'invoice.issued.v1' olayını hiçbir servis üretmiyor! Bu servis hiçbir zaman tetiklenmeyecek.",
      missingUpcaster: "❌ [TOPOLOGY_UPCASTER_MISSING] KRİTİK HATA: 'order-service' olayı 'v1' olarak yayınlarken 'shipping-service' 'v2' bekliyor. Arada 'v1 -> v2' upcaster dönüşüm tanımı bulunamadı! Veri kaybı riski.",
    },
  },
  codeMatrix: {
    tag: "DİLE ÖZGÜ İDİOMATİK TASARIM",
    title: "Tek İş Kuralı, Üç Dilde Kendi Doğallığında",
    subtitle: "Aynı iş kuralının (Minimum sipariş tutarı kontrolü) C#, Rust ve TypeScript'te o dilin en temiz idiomatik yapısıyla nasıl uygulandığını inceleyin.",
    tabs: {
      csharp: "C# (.NET 10)",
      rust: "Rust (Zero-Panic)",
      ts: "TypeScript (Functional)",
    },
  },
  docs: {
    searchPlaceholder: "Dokümantasyonda ara (örn: ARCH001, pipeRules, Outbox)...",
    chapters: {
      intro: "Giriş ve Felsefe",
      quickstart: "Kurulum ve Başlangıç",
      moduleAdopt: "Modül Kataloğu ve Adaptasyon",
      cedarAuthz: "Amazon Cedar Yetkilendirme",
      profilesDispatchers: "Profiller ve Dağıtıcılar",
      aiGovernance: "Yapay Zeka Yönetişimi (Meta MCP)",
      linters: "Mimari Linterlar",
      rulesEngine: "İş Kuralları Motoru",
      topology: "Topoloji ve Pub/Sub",
      monolithDist: "Monolith vs Dağıtık",
      cliReference: "CLI Komut Referansı",
    },
  },
  footer: {
    desc: "Yapay zeka ajanlarına kesin mimari sınırlar çizen; sözleşme öncelikli, çok dilli ve sürdürülebilir kurumsal uygulama omurgası.",
    sections: {
      ecosystem: "Ekosistem",
      tooling: "Araçlar & AI",
      governance: "Yönetişim",
    },
    links: {
      contracts: "clrinf-contracts",
      csharp: "clrinfcs (C#)",
      rust: "clrinfrs (Rust)",
      ts: "clrinfjs (TS)",
      cli: "clrinf-codegen CLI",
      mcp: "Meta MCP Sunucusu",
      scaffold: "Mimari Şablonlar",
      topology: "Topoloji Motoru",
      constitution: "Mimari Anayasa",
      versioning: "Sürümleme İlkeleri",
      license: "BSL 1.1 Lisansı",
    },
    rights: "Tüm hakları saklıdır. BSL 1.1 Lisansı altında sunulmaktadır. Change Date: 2030-09-08 (Apache-2.0).",
  },
};
