import { Language } from "../i18n";

export interface DocChapter {
  id: string;
  title: string;
  category: string;
  content: string;
}

export const docsContent: Record<Language, DocChapter[]> = {
  tr: [
    {
      id: "intro",
      title: "Giriş ve Mimari Felsefe",
      category: "Temeller",
      content: `
# Giriş ve Mimari Felsefe

## Neden clrinf?

Yapay zeka çağında yazılım geliştirme hızı katlanarak arttı. AI kodlama ajanları (Claude Code, Cursor, Antigravity, GitHub Copilot) birkaç saniye içinde yüzlerce satır kod üretebilmektedir. Ancak bu hız, beraberinde **büyük bir mimari bozulma ve teknik borç riski** getirmektedir:

- **Bağlamsız Kod Üretimi:** Ajanlar yerel projenin mimari sınırlarını bilmediğinde rastgele desenler uydurur.
- **Katman İhlalleri:** Domain modellerinin içine ORM bağımlılıkları veya controller içine veritabanı referansları sızar.
- **Diller Arası Uyumsuzluk:** Çok dilli bir ekosistemde aynı iş kuralı veya veri şeması diller arasında kaymaya (drift) uğrar.
- **Sessiz Dağıtık Hatalar:** Bir mikroservis olay üretirken kimsenin o olayı dinlememesi veya şema versiyon uyumsuzluğu nedeniyle mesajların sessizce kaybolması.

**clrinf (Common Language Runtime Infrastructure)**, bu problemlere karşı geliştirilmiş **Sözleşme Öncelikli (Contract-First)** ve **Yapay Zeka Destekli Mimari Çerçeve**'dir.

---

## 5 Temel İlke

1. **Yapay Zeka İçin Mimari Yönetişim ve Şablonlar (AI Governance):**
   AI ajanlarına sadece prompt vermek mimari kaliteyi garanti etmez. Doğrulanmış mimari şablonlar, Meta MCP araçları ve AST linterlar ile ajanın rastgele desenler uydurması yerine sürdürülebilir kurumsal kalıplar içinde kod üretmesi hedeflenir.

2. **Derleme Zamanı Güvencesi (AST-Based Linters):**
   Mimari kurallar geliştiricinin veya AI'ın inisiyatifine bırakılmaz. Roslyn (C#), Syn (Rust) ve TS Compiler API (TypeScript) ile kod derlenirken veya lint edilirken mimari ihlaller anında hata verir.

3. **Kanonik Sözleşme Önceliği (Contract-First Parity):**
   Sistemdeki tüm DTO'lar, olaylar ve şemalar CloudEvents 1.0 ve JSON Schema standartlarında tek bir kaynakta tanımlanır. C#, Rust ve TypeScript modelleri bu şemalardan otomatik üretilir.

4. **Dile Özgü Doğallık (Idiomatic Freedom):**
   C# dünyasındaki bir kütüphane (örneğin MediatR) diğer dillere zorla kopyalanmaz. Her dil kendi doğasına uygun en temiz kalıpları kullanır: C#'ta OOP & Interfaces, Rust'ta zero-panic Trait & Enums, TypeScript'te pure functions & monadic Results.

5. **Mekanik Dağıtık Güvenlik (Cross-Service Topology):**
   Ölü olaylar (Dead Events), yetim aboneler (Orphan Subscribers) ve eksik versiyon dönüştürücüler (Upcasters) statik analizle derleme öncesinde tespit edilir.
      `,
    },
    {
      id: "quickstart",
      title: "Kurulum ve Hızlı Başlangıç",
      category: "Başlangıç",
      content: `
# Kurulum ve Hızlı Başlangıç

## Gereksinimler

- **Rust:** \`cargo\` 1.80+ (CLI derleyicisi için)
- **.NET SDK:** \`dotnet\` 9.0 veya 10.0+ (C# kütüphaneleri için)
- **Node.js:** \`node\` 20+ veya 22+ (TypeScript kütüphaneleri için)

---

## CLI Kurulumu

\`clrinf-codegen\` tek bir komut satırı aracı olarak çalışır ve tüm dilleri orkestre eder:

\`\`\`bash
# Kaynak koddan derleme
git clone https://github.com/brkmustu/clrinf.git
cd clrinf/clrinf-codegen
cargo build --release

# Binary'yi PATH'e ekleme
cp target/release/clrinf-codegen /usr/local/bin/
\`\`\`

Doğrulama:
\`\`\`bash
clrinf-codegen --version
\`\`\`

---

## İlk Projenizi Başlatın

Yeni bir çok dilli servis iskeleti oluşturmak için:

\`\`\`bash
# Sözleşmeleri derleyin ve kodları üretin
clrinf-codegen generate --schema-dir clrinf-contracts/schemas --lang all --output ./src

# Mimari denetimleri çalıştırın
clrinf-codegen lint --lang all

# Dağıtık olay topolojisini doğrulayın
clrinf-codegen topology check --schema-dir clrinf-contracts/schemas
\`\`\`
      `,
    },
    {
      id: "aiGovernance",
      title: "Yapay Zeka Yönetişimi (Meta MCP ve Şablonlar)",
      category: "Yapay Zeka & Ajanlar",
      content: `
# Yapay Zeka Yönetişimi (Meta MCP ve Şablonlar)

## AI Ajanlarına Neden Mimari Sınırlar Gereklidir?

Yapay zeka ajanları (Claude Code, Cursor Composer, Antigravity) kod üretirken projenin genel mimarisini bilmediklerinde şu hataları yaparlar:
1. Rastgele NuGet veya NPM paketleri eklerler.
2. Domain katmanına EntityFramework veya SQL istemcileri enjekte ederler.
3. Hata yönetimini try/catch çorbasına çevirirler.

clrinf bu sorunu **şablon odaklı kod üretimi** ve **anlık mimari denetim** mekanizmalarıyla çözer. Ajanın sıfırdan mimari uydurması yerine, sözleşmelerden türetilen doğrulanmış kalıplarla çalışması sağlanır.

---

## clrinf Meta MCP Sunucusu

\`clrinf-codegen mcp\` komutu, AI kodlama asistanlarına standart bir **Model Context Protocol (MCP)** JSON-RPC 2.0 sunucusu sağlar. Bu sayede AI ajanı projeyi doğrudan kontrol edebilir:

### Sunulan MCP Araçları (Tools):
- \`clrinf_inspect_ecosystem\`: Projedeki dilleri, sözleşmeleri ve aktif kuralları ajana raporlar.
- \`clrinf_lint_architecture\`: Ajanın yazdığı kodun mimari kuralları ihlal edip etmediğini Roslyn/Syn/TS AST ile denetler.
- \`clrinf_scaffold_rule\`: Standartlara uygun, tip-güvenli yeni bir iş kuralı iskeleti üretir.
- \`clrinf_validate_topology\`: Eklenen yeni bir olayın topolojide kırılmaya yol açıp açmadığını test eder.
- \`clrinf_generate_pubsub\`: %100 mekanik Outbox/Idempotency kabuklarını üretir.
- \`clrinf_new_project\`: Çok dilli yeni bir servis iskeletini mimari standartlara uygun olarak ayağa kaldırır.
      `,
    },
    {
      id: "linters",
      title: "Mimari AST Linterlar",
      category: "Kalite Güvencesi",
      content: `
# Mimari AST Linterlar

Mimari kuralların ihlal edilmesini önlemenin tek kesin yolu **derleme zamanı denetimidir**. clrinf, 3 dil için de özel AST (Abstract Syntax Tree) denetleyicilerine sahiptir:

---

## 1. C# Roslyn Analizörleri (clrinfcs.Analyzers)

| Kod | Kural | Açıklama |
| :--- | :--- | :--- |
| **ARCH001** | Clean Architecture İzolasyonu | Domain katmanında EF Core, ASP.NET Core veya Newtonsoft.Json gibi dış bağımlılıklar kesinlikle yasaktır. |
| **ARCH002** | İş Kuralı Standardı | Tüm kural sınıfları \`IBusinessRule<T>\` arayüzünü uygulamalı ve \`Check()\` metodu sunmalıdır. |
| **ARCH003** | Controller İzolasyonu | Web API controller sınıflarına doğrudan \`DbContext\` sızması engellenir; CQRS Handler zorunludur. |
| **ARCH004** | CQRS Sözleşme Uyumu | Command ve Query sınıfları standart \`IRequest<T>\` modeline uymalıdır. |

---

## 2. Rust Syn AST Analizörü (clrinfrs-linter)

- **RUST_ARCH001 (Zero-Panic Policy):** Domain ve business rule modüllerinde \`.unwrap()\`, \`.expect()\` veya \`panic!()\` makrolarının kullanımı derleme hatası verir. Hatalar mutlaka \`Result<T, E>\` tipiyle dönmelidir.

---

## 3. TypeScript Compiler API Analizörü (clrinfjs-linter)

- **ARCH_TS_001:** İş kuralı fonksiyonlarında çıplak \`throw new Error()\` fırlatılması engellenir; fonksiyonel \`Result.err()\` zorunludur.
- **ARCH_TS_003:** C# MediatR taklidi hantal sınıf hiyerarşileri yasaktır; saf TypeScript fonksiyonları ve kompozisyon boru hatları (\`pipeRules\`) kullanılmalıdır.
      `,
    },
    {
      id: "rulesEngine",
      title: "Çok Dilli İş Kuralları Motoru",
      category: "Tasarım Kalıpları",
      content: `
# Çok Dilli İş Kuralları Motoru

Geleneksel mimarilerde iş kuralları (validation & business invariants) ya controller içine dağılır ya da servis sınıflarında binlerce satırlık if/else bloklarına dönüşür.

clrinf, iş kurallarını **atomik, test edilebilir ve bağımsız** birimler olarak modeller.

---

## Temel Yaklaşım

1. **Kural Atomiktir:** Her kural yalnızca tek bir iş gereksinimini doğrular.
2. **Yan Etkisizdir:** Kural veri tabanına yazmaz, dış dünya ile iletişim kurmaz; saf bir doğrulamadır.
3. **Monadiktir:** İstisna fırlatmak yerine başarı/başarısızlık durumunu tip güvenli bir sonuç nesnesi olarak döner.

---

## Kod Örnekleri

### C# (.NET)
\`\`\`csharp
public sealed class OrderMinimumAmountRule : IBusinessRule<Order>
{
    private readonly decimal _minAmount;
    public OrderMinimumAmountRule(decimal minAmount) => _minAmount = minAmount;

    public Result Check(Order entity)
    {
        if (entity.TotalAmount < _minAmount)
            return Result.Failure(OrderErrors.BelowMinimumAmount(_minAmount));

        return Result.Success();
    }
}
\`\`\`

### Rust
\`\`\`rust
pub struct OrderMinimumAmountRule {
    pub min_amount: f64,
}

impl BusinessRule<Order> for OrderMinimumAmountRule {
    fn check(&self, entity: &Order) -> Result<(), DomainError> {
        if entity.total_amount < self.min_amount {
            return Err(DomainError::RuleViolation("Below minimum order amount".into()));
        }
        Ok(())
    }
}
\`\`\`

### TypeScript
\`\`\`typescript
export const validateMinimumAmount = (minAmount: number): RuleFn<Order> => 
  (order: Order): Result<Order, DomainError> => {
    if (order.totalAmount < minAmount) {
      return Result.err(new DomainError(\`Amount must be at least \${minAmount}\`));
    }
    return Result.ok(order);
  };

// Kuralların zincirlenmesi:
const validateOrder = pipeRules(
  validateMinimumAmount(100),
  validateCustomerStatus
);
\`\`\`
      `,
    },
    {
      id: "topology",
      title: "Topoloji ve Dağıtık Pub/Sub",
      category: "Dağıtık Sistemler",
      content: `
# Topoloji ve Dağıtık Pub/Sub

## Olay Odaklı (Event-Driven) Mimarilerin Gizli Tehlikesi

Büyüyen mikroservis sistemlerinde zamanla şu problemler ortaya çıkar:
- **Dead Events (Ölü Olaylar):** Bir servis olay yayınlar; ancak refactor sonrasında artık hiçbir servis bu olayı dinlemez. Kaynak ve ağ israfı oluşur.
- **Orphan Subscribers (Yetim Tüketiciler):** Bir servis belirli bir olayı bekler; ancak sistemde bu olayı üreten hiçbir servis yoktur. Servis sessizce bekler.
- **Version Skew (Sürüm Sapması):** Servis A \`v1\` olay üretirken, Servis B \`v2\` beklemektedir. Arada bir Upcaster (şema dönüştürücü) yoksa serileştirme hatası yaşanır.

---

## clrinf Topoloji Motoru

\`clrinf-codegen topology check\` komutu, servis tanımlarını ve şemalarını tarayarak bir küresel topoloji grafiği oluşturur:

\`\`\`bash
$ clrinf-codegen topology check --schema-dir clrinf-contracts/schemas
\`\`\`

Bu komut:
1. Tüm yayınlanan olayları ve dinlenen abonelikleri eşleştirir.
2. Eşleşmeyen ölü olayları uyarı olarak bildirir.
3. Yetim aboneleri kritik hata olarak işaretler.
4. Sürüm geçişlerinde upcaster eksikliklerini raporlar.

CI/CD pipeline'ında bu komut çalıştırılarak bozuk dağıtımların prod ortamına çıkması engellenir.
      `,
    },
    {
      id: "monolithDist",
      title: "Modüler Monolith'ten Dağıtık Mimariye",
      category: "Mimari Evrim",
      content: `
# Modüler Monolith'ten Dağıtık Mimariye

## Erken Dağıtık Sistem Tuzağı

Pek çok proje daha başlangıç aşamasında Kafka, RabbitMQ ve onlarca mikroservis kurarak aşırı mühendislik tuzağına düşer. Ağ gecikmeleri, dağıtık transaction karmaşası ve yüksek altyapı maliyetleri projeyi yavaşlatır.

**clrinf, 'Modular Monolith First' ilkesini destekler.**

---

## Transport-Agnostic Portlar

clrinf'in mimari yapısında mesajlaşma aracı koda doğrudan sızmaz:
- **\`IOutboxStore\`**: Olayların güvenle depolanmasını sağlar (PostgreSQL, SQLite, In-Memory).
- **\`IIdempotencyStore\`**: Aynı mesajın mükerrer işlenmesini önler.
- **\`IEventBus\`**: Taşıyıcı katmanıdır.

### Geliştirme Evreleri:
1. **Aşama 1: In-Memory Modüler Monolith**
   Tüm servisler tek bir process içinde çalışır. Olaylar bellek içi kuyrukla aktarılır. Sıfır ağ gecikmesi, anında yerel hata ayıklama.
2. **Aşama 2: Outbox Pattern ile Güvenli Geçiş**
   Aynı veritabanı içinde atomik Outbox tablosu kullanılarak veri tutarlılığı garanti altına alınır.
3. **Aşama 3: Dağıtık Mikroservisler**
   Kodda tek bir satır iş mantığı değiştirmeden, \`IEventBus\` implementasyonu NATS, Kafka veya AWS SQS ile değiştirilerek servisler ayrıştırılır.
      `,
    },
    {
      id: "moduleAdopt",
      title: "Yerleşik Modül Kataloğu ve Adaptasyon",
      category: "Modüler Mimari",
      content: `
# Yerleşik Modül Kataloğu ve Adaptasyon Motoru

## Genel Bakış

clrinf, geliştiricilerin sıfırdan domain modülü veya altyapı katmanı yazmak zorunda kalmaması için **resmi yerleşik modüller (built-in modules)** sunar.

Geliştirici veya yapay zeka ajanı, bu modülleri var olan herhangi bir hedef projeye (.csproj, Cargo.toml, package.json) **iki farklı felsefede ve sıfır sürtünmeyle** aktarabilir:

1. **Ham Mod (\`--mode raw\` / Zero-Dependency):**
   - Hedef projeye hiçbir harici paket (nuget/cargo/npm) veya framework bağımlılığı eklemez.
   - Kendi kendine yeten sözleşmeler (\`OperationClaim\`, \`IRequest<T>\`), saf domain modelleri, repository portları ve bellek-içi test adaptörleriyle üretilir.
   - Her modülün yanında dilden bağımsız Amazon Cedar yetkilendirme politikası (\`policies/<module>.cedar\`) oluşturulur.

2. **Kablolanmış Mod (\`--mode wired\` / Full Entegrasyon):**
   - clrinf ekosisteminin tüm kabiliyetlerini barındırır.
   - C# için \`IServiceCollection.Add<Name>Module()\` DI kaydını,
   - Rust için \`pub mod <name>;\` derleme zamanı modül ağacı bildirimini,
   - TypeScript için \`export * as <name>\` barrel export'unu otomatik olarak bağlar.

---

## Yerleşik Modül Kataloğu

Katalogdaki tüm modülleri terminalden veya MCP aracıyla keşfedebilirsiniz:

\`\`\`bash
clrinf-codegen catalog
\`\`\`

| Modül Kimliği | Kategori | Tanım | Üretilen Varlıklar |
|---|---|---|---|
| \`crm\` | Domain Suite | Tam teşekküllü B2B CRM Paketi (Deals + Contacts + Activities) | Deal, Contact, Activity |
| \`deals\` | Domain Feature | Satış fırsatları hunisi (Pipeline), aşama kuralları | Deal |
| \`contacts\` | Domain Feature | Müşteri & aday yönetimi, RFC e-posta doğrulaması | Contact |
| \`activities\` | Domain Feature | İletişim, görüşme ve görev takibi, koruma kuralları | Activity |
| \`authz\` | Cross-Cutting | Amazon Cedar tabanlı, varsayılan çok kiracılı yetkilendirme | ICedarAuthorizationService |
| \`caching\` | Cross-Cutting | Bellek-içi ve Redis önbellekleme soyutlaması | ICacheService, MemoryCache |
| \`logging\` | Cross-Cutting | Yapılandırılmış loglama ve işlem yürütme süresi ölçümü | LoggingBehavior / Middleware |
| \`transaction\` | Cross-Cutting | Atomik işlem sınırları ve Unit of Work yönetimi | TransactionBehavior / UoW |
| \`idempotency\` | Cross-Cutting | Kiracı-yalıtımlı mükerrer işlem engelleme portu | IdempotencyStore (claim/release) |
| \`outbox\` | Cross-Cutting | Güvenilir olay dağıtımı için transactional outbox | OutboxStore |

---

## Örnek Adaptasyon Komutları

\`\`\`bash
# 1. C# projesine tüm CRM paketini ham (raw) modda aktar
clrinf module adopt crm \\
  --to-project ./apps/BillingService/BillingService.csproj \\
  --mode raw

# 2. Rust crate'ine CRM paketini kablolanmış (wired) ve Cedar politikasıyla aktar
clrinf module adopt crm \\
  --to-project ./crates/crm-service/Cargo.toml \\
  --mode wired

# 3. Deals modülünü 'Sales' takma adıyla C# projesine aktar
clrinf module adopt deals \\
  --to-project ./apps/BillingService/BillingService.csproj \\
  --target-dir src/Features/Sales \\
  --as Sales \\
  --mode raw

# 4. Cedar yetkilendirme altyapısını ham haliyle Rust crate'ine aktar
clrinf module adopt authz \\
  --to-project ./crates/identity-worker/Cargo.toml \\
  --mode raw
\`\`\`
      `,
    },
    {
      id: "cedarAuthz",
      title: "Amazon Cedar Çok Kiracılı Yetkilendirme",
      category: "Güvenlik",
      content: `
# Amazon Cedar Çok Kiracılı Yetkilendirme Motoru

## Neden Amazon Cedar?

Geleneksel rol tabanlı (RBAC) yetkilendirme modelleri, B2B SaaS ve çok kiracılı (multi-tenant) sistemlerde yetersiz kalır. Kod içine serpiştirilen \`if (user.TenantId == resource.TenantId)\` kontrolleri, geliştiricilerin veya yapay zeka ajanlarının bir sorguda veya endpoint'te tenant filtresini unutması durumunda **büyük veri sızıntılarına (data breach)** yol açar.

**Amazon Cedar**, Amazon AWS tarafından geliştirilen açık kaynaklı, biçimsel olarak doğrulanabilir (formally verified) bir erişim kontrol politikası dilidir. clrinf ekosisteminde yetkilendirme mantığı **koddan tamamen soyutlanarak** Cedar politikalarına (\`policies/*.cedar\`) emanet edilir.

---

## 3 Altın Kural

1. **Varsayılan Olarak Çok Kiracılı (Multi-Tenant by Default):**
   Bir kullanıcının bir kaynağa erişebilmesi için kiracı kimliğinin kaynak kiracı kimliğiyle eşleşmesi zorunludur:
   \`\`\`cedar
   permit (
       principal in Role::"TenantAdmin",
       action,
       resource in ResourceType::"Deals"
   ) when {
       resource.tenant_id == principal.tenant_id
   };
   \`\`\`

2. **Forbid Kuralının Mutlak Önceliği (Overriding Forbid Guard):**
   Cedar motorunda \`forbid\` kuralı tüm \`permit\` kurallarını ezer. Kullanıcının hangi rollere sahip olduğuna bakılmaksızın, kiracılar arası sızıntı denemesi anında mutlak olarak reddedilir:
   \`\`\`cedar
   forbid (
       principal,
       action,
       resource in ResourceType::"Deals"
   ) when {
       resource.tenant_id != principal.tenant_id
   };
   \`\`\`

3. **Süper Kullanıcı (PlatformAdmin) Geçişi:**
   Sistem bakım ve platform operasyonları için \`PlatformAdmin\` rolü tüm kiracı sınırlarını güvenle aşabilir:
   \`\`\`cedar
   permit (
       principal in Role::"PlatformAdmin",
       action,
       resource
   );
   \`\`\`

---

## Diller Arası Kullanım

- **C#:** \`[CedarAuthorize("deals.create", "Deals")]\` attribute'u veya \`AuthorizationBehavior<TRequest, TResponse>\` pipeline muhafızı.
- **Rust:** \`clrinfrs_auth::CedarAuthorizer::authorize(&claim)\` ile sıfır-panik denetimi.
- **TypeScript:** \`MultiTenantCedarAuthorizer.authorize(claim)\` saf fonksiyonel doğrulama.
      `,
    },
    {
      id: "profilesDispatchers",
      title: "Proje Profilleri ve Dağıtıcılar",
      category: "Performans",
      content: `
# Proje Profilleri ve Dağıtıcı Çerçeveleri

## 1. clrinf Başlatma Profilleri (\`--profile\`)

Yeni bir proje başlatırken veya clrinf'i entegre ederken mimari ihtiyaçlarınıza göre üç profilden birini seçebilirsiniz:

\`\`\`bash
clrinf init --profile <minimal | standard | full>
\`\`\`

- **\`minimal\` (Lean / Pure):**
  - Sıfır dayatılmış cross-cutting concern.
  - Sadece saf domain sözleşmeleri ve iş mantığı yer alır.
  - Aşırı hafif mikroservisler ve gömülü kütüphaneler için idealdir.
- **\`standard\` (Dengeli Varsayılan):**
  - \`logging\` ve \`transaction\` aktif olarak yapılandırılır.
  - Kurumsal uygulamaların %80'i için önerilen başlangıç noktasıdır.
- **\`full\` (Pilleri Dahil / Batteries-Included):**
  - 7 altyapı modülünün tamamı devrededir: \`caching\`, \`logging\`, \`transaction\`, \`authentication\`, \`authorization\` (Cedar), \`idempotency\`, \`outbox\`.
  - Kurumsal seviyede tam donanımlı bir altyapı sunar.

---

## 2. Yüksek Başarımlı Dağıtıcılar (\`--dispatcher\`)

C# ekosisteminde komut ve sorguları (CQRS) yönlendirmek için üç alternatif sunulur:

\`\`\`bash
clrinf init --dispatcher <native | mediatr | mediatornet>
\`\`\`

1. **\`native\` (ClrinfCS.Core.Dispatcher):**
   - Derleme zamanında \`FrozenDictionary\` ile üretilen statik yönlendirme haritası kullanır.
   - **Sıfır Reflection:** Çalışma zamanında assembly taraması yapmaz.
   - **Sıfır Heap Allocation:** İstek başına nesne oluşturmaz.
   - Standart MediatR'a göre **%40 daha hızlı** ve deterministiktir.

2. **\`mediatr\`:**
   - Popüler MediatR kütüphanesiyle geriye dönük tam uyumluluk köprüsü.
   - Var olan MediatR projelerini clrinf ekosistemine taşımak için idealdir.

3. **\`mediatornet\`:**
   - Mediator.Net hafif kütüphane alternatifi entegrasyonu.
      `,
    },
    {
      id: "cliReference",
      title: "CLI Komut Referansı",
      category: "Referans",
      content: `
# CLI Komut Referansı

\`clrinf-codegen\` aracı ekosistemin merkezi CLI motorudur.

---

## Temel Komutlar

### 1. \`catalog\`
Resmi yerleşik modülleri ve domain paketlerini listeler.
\`\`\`bash
clrinf-codegen catalog
\`\`\`

### 2. \`module adopt\`
Var olan herhangi bir modülü (.csproj, Cargo.toml, package.json) hedef projeye ham (raw) veya kablolanmış (wired) olarak aktarır.
\`\`\`bash
clrinf-codegen module adopt <MODÜL> --to-project <PROJE> [--mode raw|wired] [--as <TAKMA_AD>]
\`\`\`

### 3. \`init\`
Hedef projede \`clrinf.toml\` manifest dosyasını oluşturur; proje dilini otomatik algılar.
\`\`\`bash
clrinf-codegen init --profile standard --dispatcher native
\`\`\`

### 4. \`add module\` & \`add entity\`
İlgili dile özgü language worker üzerinden Cedar korumalı domain modülü ve CRUD entity'si üretir.
\`\`\`bash
clrinf-codegen add module Orders
clrinf-codegen add entity OrderItem -m Orders --prop name:string --prop price:f64
\`\`\`

### 5. \`generate\` & \`generate-pubsub\`
JSON Schema sözleşmelerinden modeller ve CloudEvents 1.0 Publisher/Subscriber kabukları üretir.
\`\`\`bash
clrinf-codegen generate --schema-dir ./schemas --lang all --output ./src
clrinf-codegen generate-pubsub --lang all --output ./src/pubsub
\`\`\`

### 6. \`lint\` & \`topology check\`
Tüm dillerde mimari kuralları denetler ve olay topolojisini doğrular.
\`\`\`bash
clrinf-codegen lint --lang all
clrinf-codegen topology check --schema-dir ./schemas
\`\`\`

### 7. \`mcp\`
Yapay zeka ajanları için Model Context Protocol (MCP) JSON-RPC 2.0 sunucusunu başlatır.
\`\`\`bash
clrinf-codegen mcp
\`\`\`
*(Araçlar: \`clrinf_list_catalog\`, \`clrinf_adopt_module\`, \`clrinf_module_manage\`, \`clrinf_add_domain_module\`, \`clrinf_add_entity\`, \`clrinf_lint_architecture\`, \`clrinf_scaffold_rule\`)*
      `,
    },
  ],
  en: [
    {
      id: "intro",
      title: "Introduction & Architectural Philosophy",
      category: "Fundamentals",
      content: `
# Introduction & Architectural Philosophy

## Why clrinf?

In the age of AI-assisted engineering, software development velocity has increased exponentially. AI coding agents (Claude Code, Cursor, Antigravity, GitHub Copilot) can produce hundreds of lines of code in seconds. However, this velocity introduces **severe architectural drift and long-term technical debt**:

- **Context-Free Code Generation:** Lacking rigid architectural boundaries, AI models invent arbitrary patterns.
- **Layer Violations:** ORM references leak into domain models, and direct database contexts leak into API controllers.
- **Cross-Language Drift:** In polyglot stacks, business rules and data models diverge silently between languages.
- **Silent Distributed Failures:** Microservices publish events that nobody consumes (Dead Events), or fail to parse versions without upcasters.

**clrinf (Common Language Runtime Infrastructure)** is a **Contract-First Architectural Framework** engineered to enforce constitutional boundaries on AI agents and preserve enterprise quality from day zero.

---

## 5 Core Pillars

1. **AI Agent Governance & Scaffolding:**
   Prompting alone cannot guarantee software quality. Verified architectural scaffolding, Meta MCP tools, and compile-time AST linters keep AI coding agents within sustainable enterprise boundaries rather than inventing arbitrary abstractions.

2. **Compile-Time Architectural Enforcement (AST Linters):**
   Architectural rules are never left to human or AI goodwill. Custom AST analyzers—Roslyn (C#), Syn (Rust), and TS Compiler API (TypeScript)—fail the build on architectural violations.

3. **Canonical Contract-First Parity:**
   All data transfer objects and events are defined in canonical CloudEvents 1.0 and JSON Schema specifications. C#, Rust, and TypeScript code is generated with 100% mechanical parity.

4. **Idiomatic Language Freedom:**
   Never force-feed one ecosystem's paradigms into another. No MediatR-style class bloat in Rust or TypeScript. C# enjoys interfaces and records; Rust leverages traits and zero-panic patterns; TypeScript embraces pure functions and monadic Results.

5. **Mechanical Distributed Safety:**
   Cross-service topology validation statically detects Dead Events, Orphan Subscribers, and Missing Upcasters before deployment.
      `,
    },
    {
      id: "quickstart",
      title: "Installation & Quickstart",
      category: "Getting Started",
      content: `
# Installation & Quickstart

## Prerequisites

- **Rust:** \`cargo\` 1.80+ (for the CLI compiler)
- **.NET SDK:** \`dotnet\` 9.0 or 10.0+ (for C# libraries and Roslyn analyzers)
- **Node.js:** \`node\` 20+ or 22+ (for TypeScript toolchain)

---

## CLI Installation

Compile and install \`clrinf-codegen\` from source:

\`\`\`bash
git clone https://github.com/brkmustu/clrinf.git
cd clrinf/clrinf-codegen
cargo build --release

# Install binary to PATH
cp target/release/clrinf-codegen /usr/local/bin/
\`\`\`

Verify installation:
\`\`\`bash
clrinf-codegen --version
\`\`\`

---

## Quickstart Workflow

Run the full verification and generation pipeline:

\`\`\`bash
# Generate polyglot types from contracts
clrinf-codegen generate --schema-dir clrinf-contracts/schemas --lang all --output ./src

# Run AST architectural linters across all languages
clrinf-codegen lint --lang all

# Verify distributed event topology
clrinf-codegen topology check --schema-dir clrinf-contracts/schemas
\`\`\`
      `,
    },
    {
      id: "aiGovernance",
      title: "AI Agent Governance (Meta MCP & Scaffolding)",
      category: "AI & Agents",
      content: `
# AI Agent Governance (Meta MCP & Scaffolding)

## Why AI Agents Require Architectural Guardrails

Autonomous coding agents write code quickly, but without strict architectural guidance they inevitably:
1. Introduce rogue dependencies.
2. Leak persistence and framework logic into core domain entities.
3. Degrade error handling into unstructured try/catch blocks.

clrinf resolves this challenge through **scaffold-driven code generation** and **real-time static analysis**. Agents are steered toward verified domain patterns rather than inventing ad-hoc structures.

---

## The clrinf Meta MCP Server

Running \`clrinf-codegen mcp\` starts a Model Context Protocol (MCP) JSON-RPC 2.0 stdio server, exposing a controlled execution interface directly to AI assistants (Claude Code, Cursor, Antigravity):

### Exposed MCP Tools:
- \`clrinf_inspect_ecosystem\`: Informs the agent about active languages, contracts, and linter rules.
- \`clrinf_lint_architecture\`: Evaluates newly authored agent code against Roslyn, Syn, and TS AST rules.
- \`clrinf_scaffold_rule\`: Generates idiomatic, type-safe rule skeletons following clean architecture.
- \`clrinf_validate_topology\`: Verifies that agent-added events do not break the distributed topology.
- \`clrinf_generate_pubsub\`: Produces 100% mechanical Outbox and Idempotency wrappers.
- \`clrinf_new_project\`: Bootstraps new multi-lingual services according to architectural invariants.
      `,
    },
    {
      id: "linters",
      title: "Architectural AST Linters",
      category: "Quality Assurance",
      content: `
# Architectural AST Linters

Static architectural analysis is the only definitive way to prevent technical decay. clrinf provides dedicated AST analyzers across all three ecosystems:

---

## 1. C# Roslyn Analyzers (clrinfcs.Analyzers)

| Rule ID | Rule Name | Description |
| :--- | :--- | :--- |
| **ARCH001** | Clean Architecture Isolation | The domain layer must have zero external dependencies on EF Core, ASP.NET Core, or serialization libraries. |
| **ARCH002** | Business Rule Standardization | All rule classes must implement \`IBusinessRule<T>\` with an explicit \`Check()\` method. |
| **ARCH003** | Controller Isolation | Web API controllers must not inject \`DbContext\` directly; CQRS handlers are enforced. |
| **ARCH004** | CQRS Contract Conformance | Commands and queries must implement \`IRequest<T>\`. |

---

## 2. Rust Syn AST Analyzer (clrinfrs-linter)

- **RUST_ARCH001 (Zero-Panic Policy):** Forbids \`.unwrap()\`, \`.expect()\`, or \`panic!()\` within domain and rule crates. All operations must return \`Result<T, E>\`.

---

## 3. TypeScript Compiler API Analyzer (clrinfjs-linter)

- **ARCH_TS_001:** Forbids raw \`throw new Error()\` inside rule evaluations; requires monadic \`Result.err()\`.
- **ARCH_TS_003:** Forbids artificial MediatR-style class hierarchies; requires functional composition via \`pipeRules\`.
      `,
    },
    {
      id: "rulesEngine",
      title: "Idiomatic Business Rules Engine",
      category: "Design Patterns",
      content: `
# Idiomatic Business Rules Engine

In traditional codebases, business validation rules often become tangled inside controllers or bloated into massive imperative methods.

clrinf isolates business invariants into **atomic, side-effect-free, composable** units.

---

## Core Characteristics

1. **Atomic:** Each rule validates exactly one invariant.
2. **Pure & Isolated:** Zero database queries or network I/O; deterministic validation.
3. **Monadic Results:** Returns explicit success or failure objects instead of throwing exceptions.

---

## Multi-Language Implementations

### C# (.NET)
\`\`\`csharp
public sealed class OrderMinimumAmountRule : IBusinessRule<Order>
{
    private readonly decimal _minAmount;
    public OrderMinimumAmountRule(decimal minAmount) => _minAmount = minAmount;

    public Result Check(Order entity)
    {
        if (entity.TotalAmount < _minAmount)
            return Result.Failure(OrderErrors.BelowMinimumAmount(_minAmount));

        return Result.Success();
    }
}
\`\`\`

### Rust
\`\`\`rust
pub struct OrderMinimumAmountRule {
    pub min_amount: f64,
}

impl BusinessRule<Order> for OrderMinimumAmountRule {
    fn check(&self, entity: &Order) -> Result<(), DomainError> {
        if entity.total_amount < self.min_amount {
            return Err(DomainError::RuleViolation("Below minimum order amount".into()));
        }
        Ok(())
    }
}
\`\`\`

### TypeScript
\`\`\`typescript
export const validateMinimumAmount = (minAmount: number): RuleFn<Order> => 
  (order: Order): Result<Order, DomainError> => {
    if (order.totalAmount < minAmount) {
      return Result.err(new DomainError(\`Amount must be at least \${minAmount}\`));
    }
    return Result.ok(order);
  };

// Composing rules functionally:
const validateOrder = pipeRules(
  validateMinimumAmount(100),
  validateCustomerStatus
);
\`\`\`
      `,
    },
    {
      id: "topology",
      title: "Topology & Distributed Pub/Sub",
      category: "Distributed Systems",
      content: `
# Topology & Distributed Pub/Sub

## The Silent Traps of Event-Driven Systems

As microservices expand, distributed asynchronous bugs become harder to detect:
- **Dead Events:** A service publishes an event that has no active subscribers.
- **Orphan Subscribers:** A service subscribes to an event that is never produced by any publisher.
- **Version Skew:** Service A publishes \`v1\` while Service B expects \`v2\`, resulting in runtime deserialization failure unless an Upcaster is defined.

---

## clrinf Topology Verification

The \`clrinf-codegen topology check\` command scans service manifests and contracts to build a holistic event graph:

\`\`\`bash
$ clrinf-codegen topology check --schema-dir clrinf-contracts/schemas
\`\`\`

Diagnostics include:
- Warning for unconsumed Dead Events.
- Fatal errors for Orphan Subscribers.
- Upcaster evolution gap detection for multi-version migrations.
      `,
    },
    {
      id: "monolithDist",
      title: "Modular Monolith to Distributed Evolution",
      category: "Architectural Evolution",
      content: `
# Modular Monolith to Distributed Evolution

## Avoiding the Premature Microservices Trap

Teams often introduce Kafka or Kubernetes too early, burdening development with distributed transaction complexity and network debugging.

**clrinf champions a 'Modular Monolith First' philosophy.**

---

## Transport-Agnostic Ports

By abstracting persistence and communication behind clean ports:
- **\`IOutboxStore\`**: Transactionally records published events before dispatch.
- **\`IIdempotencyStore\`**: Guarantees exactly-once processing semantics.
- **\`IEventBus\`**: Pluggable transport provider.

### Evolution Lifecycle:
1. **Stage 1: In-Memory Modular Monolith**
   Zero network overhead; in-memory message bus; instant debugging in a single process.
2. **Stage 2: Transactional Outbox**
   Persist outbox records in PostgreSQL/SQLite alongside business state.
3. **Stage 3: Distributed Microservices**
   Switch the \`IEventBus\` adapter to NATS, RabbitMQ, or Kafka without changing a single line of domain code.
      `,
    },
    {
      id: "moduleAdopt",
      title: "Built-in Module Catalog & Adoption Engine",
      category: "Architecture",
      content: `
# Built-in Module Catalog & Adoption Engine

## Overview

To eliminate the friction of building domain modules or infrastructure cross-cutting concerns from scratch, clrinf provides an official catalog of **built-in modules**.

Developers and AI coding agents can adopt any of these modules into existing target projects (.csproj, Cargo.toml, package.json) with **zero friction across two operational modes**:

1. **Raw Mode (\`--mode raw\` / Zero-Dependency):**
   - Injects zero external packages (nuget/cargo/npm) and imposes zero framework lock-in.
   - Generates self-contained contracts (\`OperationClaim\`, \`IRequest<T>\`), pure domain models, repository ports, and in-memory test doubles.
   - Emits language-agnostic Amazon Cedar multi-tenant policies (\`policies/<module>.cedar\`) alongside the code.

2. **Wired Mode (\`--mode wired\` / Batteries-Included):**
   - Injects full clrinf infrastructure wiring.
   - Automatically binds \`IServiceCollection.Add<Name>Module()\` for C#,
   - Binds compile-time module tree declarations \`pub mod <name>;\` for Rust,
   - Binds barrel exports \`export * as <name>\` for TypeScript.

---

## Built-in Module Catalog

Discover all available modules via the terminal or MCP tool:

\`\`\`bash
clrinf-codegen catalog
\`\`\`

| Module ID | Category | Description | Generated Entities |
|---|---|---|---|
| \`crm\` | Domain Suite | Full-scale B2B CRM Suite (Deals + Contacts + Activities) | Deal, Contact, Activity |
| \`deals\` | Domain Feature | Sales opportunity pipeline, stage transition rules | Deal |
| \`contacts\` | Domain Feature | Lead & customer management, RFC-compliant email checks | Contact |
| \`activities\` | Domain Feature | Meeting, call & task tracking, completion guards | Activity |
| \`authz\` | Cross-Cutting | Amazon Cedar RBAC & multi-tenant authorization engine | ICedarAuthorizationService |
| \`caching\` | Cross-Cutting | In-memory & distributed cache abstraction | ICacheService, MemoryCache |
| \`logging\` | Cross-Cutting | Structured execution logging & duration tracking | LoggingBehavior / Middleware |
| \`transaction\` | Cross-Cutting | Atomic transaction boundaries & Unit of Work | TransactionBehavior / UoW |
| \`idempotency\` | Cross-Cutting | Tenant-isolated duplicate execution guard | IdempotencyStore (claim/release) |
| \`outbox\` | Cross-Cutting | Reliable event publishing transactional outbox | OutboxStore |

---

## Example Adoption Commands

\`\`\`bash
# 1. Adopt full CRM suite into a C# project in raw (zero-dependency) mode
clrinf module adopt crm \\
  --to-project ./apps/BillingService/BillingService.csproj \\
  --mode raw

# 2. Adopt CRM suite into a Rust crate in wired mode with Cedar security policy
clrinf module adopt crm \\
  --to-project ./crates/crm-service/Cargo.toml \\
  --mode wired

# 3. Adopt Deals feature into C# under custom alias 'Sales'
clrinf module adopt deals \\
  --to-project ./apps/BillingService/BillingService.csproj \\
  --target-dir src/Features/Sales \\
  --as Sales \\
  --mode raw

# 4. Adopt Cedar authorization engine into a Rust crate
clrinf module adopt authz \\
  --to-project ./crates/identity-worker/Cargo.toml \\
  --mode raw
\`\`\`
      `,
    },
    {
      id: "cedarAuthz",
      title: "Amazon Cedar Multi-Tenant Authorization",
      category: "Security",
      content: `
# Amazon Cedar Multi-Tenant Authorization Engine

## Why Amazon Cedar?

Traditional Role-Based Access Control (RBAC) breaks down in multi-tenant B2B architectures. Scattering \`if (user.TenantId == resource.TenantId)\` checks across controllers and queries is fragile; a single missed check by an engineer or AI agent causes **catastrophic cross-tenant data leaks**.

**Amazon Cedar** is an open-source, formally verified policy language developed by AWS. In clrinf, authorization logic is **completely decoupled from business code** and offloaded to Cedar policies (\`policies/*.cedar\`).

---

## The 3 Golden Rules

1. **Multi-Tenant by Default:**
   Principal and resource tenant IDs must match before any action is permitted:
   \`\`\`cedar
   permit (
       principal in Role::"TenantAdmin",
       action,
       resource in ResourceType::"Deals"
   ) when {
       resource.tenant_id == principal.tenant_id
   };
   \`\`\`

2. **Absolute Forbid Precedence (Overriding Guard):**
   Cedar's \`forbid\` rule overrides all \`permit\` rules. Any cross-tenant data access attempt is rejected unconditionally, regardless of user roles:
   \`\`\`cedar
   forbid (
       principal,
       action,
       resource in ResourceType::"Deals"
   ) when {
       resource.tenant_id != principal.tenant_id
   };
   \`\`\`

3. **Super-Admin (PlatformAdmin) Override:**
   Platform operations and system audits can cross tenant boundaries safely via dedicated rules:
   \`\`\`cedar
   permit (
       principal in Role::"PlatformAdmin",
       action,
       resource
   );
   \`\`\`

---

## Cross-Language Implementation

- **C#:** \`[CedarAuthorize("deals.create", "Deals")]\` attribute or pipeline guard \`AuthorizationBehavior<TRequest, TResponse>\`.
- **Rust:** Zero-panic verification via \`clrinfrs_auth::CedarAuthorizer::authorize(&claim)\`.
- **TypeScript:** Pure functional verification via \`MultiTenantCedarAuthorizer.authorize(claim)\`.
      `,
    },
    {
      id: "profilesDispatchers",
      title: "Project Profiles & Dispatchers",
      category: "Performance",
      content: `
# Project Profiles & Dispatcher Frameworks

## 1. clrinf Initialization Profiles (\`--profile\`)

Tailor new projects or integrations to your exact architectural needs:

\`\`\`bash
clrinf init --profile <minimal | standard | full>
\`\`\`

- **\`minimal\` (Lean / Pure):**
  - Zero forced cross-cutting concerns.
  - Pure domain contracts and business logic only.
  - Perfect for ultra-light microservices, CLI tools, and embedded libraries.
- **\`standard\` (Balanced Default):**
  - Configures active \`logging\` and \`transaction\` behaviors.
  - Recommended starting point for 80% of enterprise backends.
- **\`full\` (Batteries-Included):**
  - All 7 infrastructure concerns active: \`caching\`, \`logging\`, \`transaction\`, \`authentication\`, \`authorization\` (Cedar), \`idempotency\`, \`outbox\`.
  - Comprehensive foundation for mission-critical enterprise platforms.

---

## 2. High-Performance CQRS Dispatchers (\`--dispatcher\`)

For the C# ecosystem, clrinf provides three dispatcher choices:

\`\`\`bash
clrinf init --dispatcher <native | mediatr | mediatornet>
\`\`\`

1. **\`native\` (ClrinfCS.Core.Dispatcher):**
   - Built on compile-time \`FrozenDictionary\` routing maps.
   - **Zero Reflection:** No runtime assembly scanning.
   - **Zero Heap Allocations:** Zero per-dispatch object allocations.
   - Up to **40% faster** than standard MediatR with deterministic latency.

2. **\`mediatr\`:**
   - Backward compatibility bridge with MediatR.
   - Seamlessly adopts existing MediatR pipelines into the clrinf contract system.

3. **\`mediatornet\`:**
   - Lightweight alternative mediator integration.
      `,
    },
    {
      id: "cliReference",
      title: "CLI Command Reference",
      category: "Reference",
      content: `
# CLI Command Reference

The \`clrinf-codegen\` binary is the operational hub of the ecosystem.

---

## Core Commands

### 1. \`catalog\`
Lists all official built-in modules and domain suites.
\`\`\`bash
clrinf-codegen catalog
\`\`\`

### 2. \`module adopt\`
Adopts any catalog module into a target project (.csproj, Cargo.toml, package.json) in raw or wired mode.
\`\`\`bash
clrinf-codegen module adopt <MODULE> --to-project <PROJECT> [--mode raw|wired] [--as <ALIAS>]
\`\`\`

### 3. \`init\`
Initializes \`clrinf.toml\` manifest in a project with auto-detected language.
\`\`\`bash
clrinf-codegen init --profile standard --dispatcher native
\`\`\`

### 4. \`add module\` & \`add entity\`
Generates Cedar-protected domain modules and CRUD entities via language workers.
\`\`\`bash
clrinf-codegen add module Orders
clrinf-codegen add entity OrderItem -m Orders --prop name:string --prop price:f64
\`\`\`

### 5. \`generate\` & \`generate-pubsub\`
Generates typed models and CloudEvents 1.0 Publisher/Subscriber skeletons from JSON Schema contracts.
\`\`\`bash
clrinf-codegen generate --schema-dir ./schemas --lang all --output ./src
clrinf-codegen generate-pubsub --lang all --output ./src/pubsub
\`\`\`

### 6. \`lint\` & \`topology check\`
Enforces architectural rules via AST analyzers and verifies pub/sub topology.
\`\`\`bash
clrinf-codegen lint --lang all
clrinf-codegen topology check --schema-dir ./schemas
\`\`\`

### 7. \`mcp\`
Launches the Model Context Protocol (MCP) JSON-RPC 2.0 server for AI coding assistants.
\`\`\`bash
clrinf-codegen mcp
\`\`\`
*(Tools: \`clrinf_list_catalog\`, \`clrinf_adopt_module\`, \`clrinf_module_manage\`, \`clrinf_add_domain_module\`, \`clrinf_add_entity\`, \`clrinf_lint_architecture\`, \`clrinf_scaffold_rule\`)*
      `,
    },
  ],
  es: [
    {
      id: "intro",
      title: "Introducción y Filosofía Arquitectónica",
      category: "Fundamentos",
      content: `
# Introducción y Filosofía Arquitectónica

## ¿Por qué clrinf?

En la era del desarrollo asistido por Inteligencia Artificial, la velocidad de desarrollo de software se ha multiplicado. Los agentes de código (Claude Code, Cursor, Antigravity, GitHub Copilot) pueden generar cientos de líneas de código en segundos. Sin embargo, esta velocidad conlleva un **grave riesgo de degradación arquitectónica y deuda técnica**:

- **Generación de código sin contexto:** Al carecer de límites arquitectónicos rígidos, los modelos de IA inventan patrones arbitrarios.
- **Violación de capas:** Los modelos de dominio incorporan dependencias de ORM y los controladores de API acceden directamente a la base de datos.
- **Divergencia políglota:** En pilas tecnológicas mixtas, las reglas de negocio y los contratos de datos se desalinean entre lenguajes.
- **Fallos silenciosos distribuidos:** Los microservicios publican eventos que nadie consume (Dead Events) o pierden mensajes por incompatibilidad de versiones.

**clrinf (Common Language Runtime Infrastructure)** es un marco de trabajo **basado en contratos** diseñado para establecer límites constitucionales a los agentes de IA y preservar la calidad empresarial desde el primer día.

---

## 5 Pilares Fundamentales

1. **Gobernanza de Agentes IA y Plantillas Arquitectónicas:**
   Las instrucciones en lenguaje natural no garantizan la calidad del software. Mediante plantillas arquitectónicas verificadas, herramientas Meta MCP y linters AST en tiempo de compilación, los agentes de IA se mantienen dentro de límites empresariales sostenibles.

2. **Garantía en Tiempo de Compilación (AST Linters):**
   Las reglas arquitectónicas no dependen de la buena voluntad humana o de la IA. Analizadores sintácticos (Roslyn en C#, Syn en Rust y TS Compiler API en TypeScript) bloquean la compilación ante cualquier infracción arquitectónica.

3. **Paridad Canónica Basada en Contratos:**
   Todos los eventos y DTOs se definen canónicamente mediante CloudEvents 1.0 y JSON Schema. El código para C#, Rust y TypeScript se genera con paridad mecánica del 100%.

4. **Libertad Idiomática por Lenguaje:**
   No se imponen paradigmas ajenos a cada lenguaje. C# aprovecha interfaces e inyección de dependencias; Rust utiliza traits y un enfoque sin pánico (zero-panic); TypeScript implementa funciones puras y mónadas Result.

5. **Seguridad Mecánica Distribuida:**
   La validación de topología entre servicios detecta eventos muertos, consumidores huérfanos y falta de transformadores de versión (Upcasters) antes del despliegue.
      `,
    },
    {
      id: "quickstart",
      title: "Instalación y Guía Rápida",
      category: "Inicio",
      content: `
# Instalación y Guía Rápida

## Requisitos Previos

- **Rust:** \`cargo\` 1.80+ (para compilar la herramienta CLI)
- **.NET SDK:** \`dotnet\` 9.0 o 10.0+ (para librerías C# y analizadores Roslyn)
- **Node.js:** \`node\` 20+ o 22+ (para el ecosistema TypeScript)

---

## Instalación de la CLI

\`\`\`bash
git clone https://github.com/brkmustu/clrinf.git
cd clrinf/clrinf-codegen
cargo build --release

# Copiar binario a PATH
cp target/release/clrinf-codegen /usr/local/bin/
\`\`\`

Verificación:
\`\`\`bash
clrinf-codegen --version
\`\`\`

---

## Flujo de Trabajo Rápido

\`\`\`bash
# Generar código políglota desde esquemas
clrinf-codegen generate --schema-dir clrinf-contracts/schemas --lang all --output ./src

# Ejecutar linters arquitectónicos AST
clrinf-codegen lint --lang all

# Validar topología distribuida
clrinf-codegen topology check --schema-dir clrinf-contracts/schemas
\`\`\`
      `,
    },
    {
      id: "aiGovernance",
      title: "Gobernanza de Agentes IA (Meta MCP y Plantillas)",
      category: "IA y Agentes",
      content: `
# Gobernanza de Agentes IA (Meta MCP y Plantillas)

## ¿Por qué los agentes de IA necesitan límites arquitectónicos?

Los agentes autónomos generan código a gran velocidad, pero sin restricciones arquitectónicas tienden a:
1. Añadir dependencias no autorizadas.
2. Mezclar lógica de persistencia en entidades de dominio.
3. Degradar el manejo de errores a bloques try/catch caóticos.

clrinf resuelve este desafío mediante **generación de código guiada por plantillas** y **análisis estático en tiempo real**. Los agentes se orientan hacia patrones de dominio verificados en lugar de inventar estructuras arbitrarias.

---

## Servidor Meta MCP de clrinf

El comando \`clrinf-codegen mcp\` inicia un servidor Model Context Protocol (MCP) JSON-RPC 2.0 por stdio, proporcionando una interfaz estandarizada para asistentes de IA (Claude Code, Cursor, Antigravity):

### Herramientas MCP Disponibles:
- \`clrinf_inspect_ecosystem\`: Reporta lenguajes activos, contratos y reglas arquitectónicas.
- \`clrinf_lint_architecture\`: Evalúa el código del agente con analizadores Roslyn, Syn y TS AST.
- \`clrinf_scaffold_rule\`: Genera plantillas seguras para nuevas reglas de negocio.
- \`clrinf_validate_topology\`: Verifica que los nuevos eventos no rompan la topología distribuida.
- \`clrinf_generate_pubsub\`: Produce esqueletos Outbox e Idempotencia garantizados al 100%.
- \`clrinf_new_project\`: Inicializa nuevos servicios políglotas siguiendo los invariantes arquitectónicos.
      `,
    },
    {
      id: "linters",
      title: "Linters Arquitectónicos AST",
      category: "Control de Calidad",
      content: `
# Linters Arquitectónicos AST

El análisis estático es el único medio seguro para evitar la degradación del diseño de software. clrinf incluye analizadores sintácticos dedicados para cada lenguaje:

---

## 1. Analizadores Roslyn para C# (clrinfcs.Analyzers)

| Regla | Nombre | Descripción |
| :--- | :--- | :--- |
| **ARCH001** | Aislamiento de Clean Architecture | La capa de dominio no puede depender de EF Core, ASP.NET Core ni librerías externas. |
| **ARCH002** | Estandarización de Reglas de Negocio | Todas las reglas deben implementar \`IBusinessRule<T>\` y exponer \`Check()\`. |
| **ARCH003** | Aislamiento de Controladores | Los controladores de API no deben inyectar directamente \`DbContext\`; se requiere CQRS. |
| **ARCH004** | Conformidad CQRS | Los comandos y consultas deben implementar \`IRequest<T>\`. |

---

## 2. Analizador Syn AST para Rust (clrinfrs-linter)

- **RUST_ARCH001 (Política Zero-Panic):** Prohíbe el uso de \`.unwrap()\`, \`.expect()\` o \`panic!()\` en módulos de dominio. Es obligatorio retornar \`Result<T, E>\`.

---

## 3. Analizador TS Compiler API para TypeScript (clrinfjs-linter)

- **ARCH_TS_001:** Prohíbe \`throw new Error()\` en la evaluación de reglas; requiere \`Result.err()\`.
- **ARCH_TS_003:** Prohíbe emulaciones artificiales de MediatR; fomenta la composición funcional pura con \`pipeRules\`.
      `,
    },
    {
      id: "rulesEngine",
      title: "Motor Idiomático de Reglas de Negocio",
      category: "Patrones de Diseño",
      content: `
# Motor Idiomático de Reglas de Negocio

En arquitecturas tradicionales, las validaciones de negocio suelen quedar dispersas en controladores o sobrecargar los servicios con bloques if/else interminables.

clrinf modela las reglas como unidades **atómicas, puras y componibles**.

---

## Características Principales

1. **Atómica:** Cada regla verifica una única condición de negocio.
2. **Pura y Aislada:** Sin operaciones de E/S ni acceso a bases de datos; validación determinista.
3. **Mónada Result:** Devuelve objetos explícitos de éxito o fallo en lugar de lanzar excepciones.

---

## Implementación Multilingüe

### C# (.NET)
\`\`\`csharp
public sealed class OrderMinimumAmountRule : IBusinessRule<Order>
{
    private readonly decimal _minAmount;
    public OrderMinimumAmountRule(decimal minAmount) => _minAmount = minAmount;

    public Result Check(Order entity)
    {
        if (entity.TotalAmount < _minAmount)
            return Result.Failure(OrderErrors.BelowMinimumAmount(_minAmount));

        return Result.Success();
    }
}
\`\`\`

### Rust
\`\`\`rust
pub struct OrderMinimumAmountRule {
    pub min_amount: f64,
}

impl BusinessRule<Order> for OrderMinimumAmountRule {
    fn check(&self, entity: &Order) -> Result<(), DomainError> {
        if entity.total_amount < self.min_amount {
            return Err(DomainError::RuleViolation("Below minimum order amount".into()));
        }
        Ok(())
    }
}
\`\`\`

### TypeScript
\`\`\`typescript
export const validateMinimumAmount = (minAmount: number): RuleFn<Order> => 
  (order: Order): Result<Order, DomainError> => {
    if (order.totalAmount < minAmount) {
      return Result.err(new DomainError(\`Amount must be at least \${minAmount}\`));
    }
    return Result.ok(order);
  };

// Composición funcional de reglas:
const validateOrder = pipeRules(
  validateMinimumAmount(100),
  validateCustomerStatus
);
\`\`\`
      `,
    },
    {
      id: "topology",
      title: "Topología y Pub/Sub Distribuido",
      category: "Sistemas Distribuidos",
      content: `
# Topología y Pub/Sub Distribuido

## Riesgos Ocultos en Sistemas Dirigidos por Eventos

A medida que una arquitectura distribuida crece, aparecen fallos difíciles de rastrear:
- **Eventos Muertos (Dead Events):** Un servicio emite un evento que ningún suscriptor consume.
- **Consumidores Huérfanos (Orphan Subscribers):** Un servicio escucha un evento que jamás es producido en el ecosistema.
- **Divergencia de Versiones:** El Servicio A emite la versión \`v1\` mientras el Servicio B espera la \`v2\`, causando fallos de deserialización si no hay un Upcaster registrado.

---

## Validación de Topología en clrinf

El comando \`clrinf-codegen topology check\` analiza los contratos y los manifiestos de los servicios para construir un grafo global de eventos:

\`\`\`bash
$ clrinf-codegen topology check --schema-dir clrinf-contracts/schemas
\`\`\`

Diagnósticos generados:
- Advertencias de eventos muertos no consumidos.
- Errores fatales ante consumidores huérfanos.
- Detección de saltos de versión sin Upcasters de migración.
      `,
    },
    {
      id: "monolithDist",
      title: "De Monolito Modular a Sistema Distribuido",
      category: "Evolución Arquitectónica",
      content: `
# De Monolito Modular a Sistema Distribuido

## Evitando la Trampa de los Microservicios Prematuros

Muchas organizaciones adoptan Kafka o clústeres de Kubernetes de manera prematura, sufriendo la complejidad de transacciones distribuidas y problemas de red innecesarios.

**clrinf respalda la filosofía 'Monolito Modular Primero'.**

---

## Puertos Independientes del Transporte

Al desacoplar el transporte del núcleo de la aplicación:
- **\`IOutboxStore\`**: Registra de forma transaccional los eventos antes de enviarlos.
- **\`IIdempotencyStore\`**: Garantiza el procesamiento una sola vez (exactly-once semantics).
- **\`IEventBus\`**: Adaptador conectable de transporte de mensajes.

### Ciclo de Evolución:
1. **Fase 1: Monolito Modular en Memoria**
   Cero latencia de red; bus de eventos en memoria; depuración rápida en un único proceso.
2. **Fase 2: Patrón Outbox Transaccional**
   Persistencia en SQLite o PostgreSQL garantizando consistencia atómica de datos.
3. **Fase 3: Microservicios Distribuidos**
   Cambio del adaptador de \`IEventBus\` a NATS, Kafka o AWS SQS sin modificar una sola línea de código de dominio.
      `,
    },
    {
      id: "moduleAdopt",
      title: "Catálogo de Módulos Nativos y Motor de Adopción",
      category: "Arquitectura",
      content: `
# Catálogo de Módulos Nativos y Motor de Adopción

## Visión General

Para eliminar la fricción de crear módulos de dominio o componentes transversales de infraestructura desde cero, clrinf proporciona un catálogo oficial de **módulos nativos (built-in modules)**.

Los desarrolladores y agentes de IA pueden adoptar cualquiera de estos módulos en proyectos de destino existentes (.csproj, Cargo.toml, package.json) con **cero fricción bajo dos modalidades**:

1. **Modo Puro (\`--mode raw\` / Zero-Dependency):**
   - No inyecta ningún paquete externo (nuget/cargo/npm) ni impone acoplamiento a frameworks.
   - Genera contratos autocontenidos (\`OperationClaim\`, \`IRequest<T>\`), modelos de dominio puros, puertos de repositorio y dobles de prueba en memoria.
   - Emite políticas de autorización multinquilino independientes del lenguaje en Amazon Cedar (\`policies/<modulo>.cedar\`).

2. **Modo Integrado (\`--mode wired\` / Conectado):**
   - Inyecta la infraestructura completa de clrinf.
   - Registra automáticamente \`IServiceCollection.Add<Name>Module()\` en C#,
   - Vincula las declaraciones de árbol de módulos \`pub mod <name>;\` en tiempo de compilación en Rust,
   - Vincula exportaciones de barril \`export * as <name>\` en TypeScript.

---

## Catálogo de Módulos Nativos

Descubra los módulos disponibles desde la terminal o mediante herramientas MCP:

\`\`\`bash
clrinf-codegen catalog
\`\`\`

| ID del Módulo | Categoría | Descripción | Entidades Generadas |
|---|---|---|---|
| \`crm\` | Suite de Dominio | Suite B2B CRM completa (Deals + Contacts + Activities) | Deal, Contact, Activity |
| \`deals\` | Módulo de Dominio | Embudo de ventas (Pipeline) y reglas de transición | Deal |
| \`contacts\` | Módulo de Dominio | Gestión de clientes y prospectos, validación RFC de correo | Contact |
| \`activities\` | Módulo de Dominio | Seguimiento de reuniones, llamadas y tareas | Activity |
| \`authz\` | Infraestructura | Motor de autorización multinquilino basado en Amazon Cedar | ICedarAuthorizationService |
| \`caching\` | Infraestructura | Abstracción de caché en memoria y distribuida | ICacheService, MemoryCache |
| \`logging\` | Infraestructura | Registro estructurado y medición de tiempos de ejecución | LoggingBehavior / Middleware |
| \`transaction\` | Infraestructura | Límites de transacción atómica y Unit of Work | TransactionBehavior / UoW |
| \`idempotency\` | Infraestructura | Prevención de ejecución duplicada con aislamiento de inquilino | IdempotencyStore (claim/release) |
| \`outbox\` | Infraestructura | Outbox transaccional para publicación confiable de eventos | OutboxStore |

---

## Ejemplos de Comandos de Adopción

\`\`\`bash
# 1. Adoptar la suite CRM completa en un proyecto C# en modo puro (raw)
clrinf module adopt crm \\
  --to-project ./apps/BillingService/BillingService.csproj \\
  --mode raw

# 2. Adoptar CRM en un crate de Rust en modo integrado (wired) con política Cedar
clrinf module adopt crm \\
  --to-project ./crates/crm-service/Cargo.toml \\
  --mode wired

# 3. Adoptar el módulo Deals en C# bajo el alias 'Sales'
clrinf module adopt deals \\
  --to-project ./apps/BillingService/BillingService.csproj \\
  --target-dir src/Features/Sales \\
  --as Sales \\
  --mode raw

# 4. Adoptar el motor de autorización Cedar en un crate de Rust
clrinf module adopt authz \\
  --to-project ./crates/identity-worker/Cargo.toml \\
  --mode raw
\`\`\`
      `,
    },
    {
      id: "cedarAuthz",
      title: "Autorización Multinquilino con Amazon Cedar",
      category: "Seguridad",
      content: `
# Motor de Autorización Multinquilino con Amazon Cedar

## ¿Por qué Amazon Cedar?

El control de acceso basado en roles (RBAC) tradicional no es suficiente en arquitecturas B2B multinquilino (multi-tenant). Esparcir verificaciones \`if (user.TenantId == resource.TenantId)\` en controladores y consultas resulta frágil: una sola omisión por parte de un ingeniero o un agente de IA provoca **filtraciones masivas de datos entre inquilinos**.

**Amazon Cedar** es un lenguaje de políticas de acceso de código abierto y formalmente verificable desarrollado por AWS. En clrinf, la lógica de autorización se desacopla por completo del código de negocio y se delega a políticas Cedar (\`policies/*.cedar\`).

---

## Las 3 Reglas de Oro

1. **Multinquilino por Defecto (Multi-Tenant by Default):**
   El ID de inquilino del usuario debe coincidir con el del recurso para autorizar la acción:
   \`\`\`cedar
   permit (
       principal in Role::"TenantAdmin",
       action,
       resource in ResourceType::"Deals"
   ) when {
       resource.tenant_id == principal.tenant_id
   };
   \`\`\`

2. **Prioridad Absoluta de Forbid (Overriding Guard):**
   La regla \`forbid\` de Cedar anula cualquier regla \`permit\`. Cualquier intento de acceso a través de límites de inquilino se rechaza de forma categórica e incondicional:
   \`\`\`cedar
   forbid (
       principal,
       action,
       resource in ResourceType::"Deals"
   ) when {
       resource.tenant_id != principal.tenant_id
   };
   \`\`\`

3. **Super-Administrador (PlatformAdmin):**
   Las operaciones de plataforma y mantenimiento pueden cruzar límites de inquilinos de manera segura:
   \`\`\`cedar
   permit (
       principal in Role::"PlatformAdmin",
       action,
       resource
   );
   \`\`\`

---

## Implementación Multilenguaje

- **C#:** Atributo \`[CedarAuthorize("deals.create", "Deals")]\` o guardia de pipeline \`AuthorizationBehavior<TRequest, TResponse>\`.
- **Rust:** Verificación con cero pánicos mediante \`clrinfrs_auth::CedarAuthorizer::authorize(&claim)\`.
- **TypeScript:** Verificación funcional pura mediante \`MultiTenantCedarAuthorizer.authorize(claim)\`.
      `,
    },
    {
      id: "profilesDispatchers",
      title: "Perfiles de Proyecto y Despachadores",
      category: "Rendimiento",
      content: `
# Perfiles de Proyecto y Despachadores CQRS

## 1. Perfiles de Inicialización clrinf (\`--profile\`)

Adapte nuevos proyectos o integraciones a sus requisitos arquitectónicos:

\`\`\`bash
clrinf init --profile <minimal | standard | full>
\`\`\`

- **\`minimal\` (Puro / Ligero):**
  - Cero componentes transversales forzados.
  - Únicamente contratos y lógica de dominio puros.
  - Ideal para microservicios ultraligeros, herramientas CLI y bibliotecas embebidas.
- **\`standard\` (Equilibrado por Defecto):**
  - Configura comportamientos activos de \`logging\` y \`transaction\`.
  - Punto de partida recomendado para el 80% de aplicaciones empresariales.
- **\`full\` (Baterías Incluidas):**
  - Los 7 módulos de infraestructura activos: \`caching\`, \`logging\`, \`transaction\`, \`authentication\`, \`authorization\` (Cedar), \`idempotency\`, \`outbox\`.
  - Plataforma integral para sistemas empresariales críticos.

---

## 2. Despachadores CQRS de Alto Rendimiento (\`--dispatcher\`)

Para el ecosistema C#, clrinf ofrece tres opciones de despacho:

\`\`\`bash
clrinf init --dispatcher <native | mediatr | mediatornet>
\`\`\`

1. **\`native\` (ClrinfCS.Core.Dispatcher):**
   - Construido con mapas estáticos de enrutamiento \`FrozenDictionary\` en tiempo de compilación.
   - **Cero Reflexión:** Sin escaneo de ensamblados en tiempo de ejecución.
   - **Cero Asignaciones en Heap:** Sin creación de objetos por cada despacho.
   - Hasta un **40% más rápido** que MediatR estándar con latencia determinista.

2. **\`mediatr\`:**
   - Puente de compatibilidad total con MediatR.
   - Permite migrar pipelines existentes de MediatR a los contratos de clrinf.

3. **\`mediatornet\`:**
   - Alternativa ligera de integración con Mediator.Net.
      `,
    },
    {
      id: "cliReference",
      title: "Referencia de Comandos CLI",
      category: "Referencia",
      content: `
# Referencia de Comandos CLI

El ejecutable \`clrinf-codegen\` es el centro operativo del ecosistema.

---

## Comandos Principales

### 1. \`catalog\`
Lista los módulos nativos oficiales y suites de dominio disponibles.
\`\`\`bash
clrinf-codegen catalog
\`\`\`

### 2. \`module adopt\`
Adopta cualquier módulo del catálogo en un proyecto de destino (.csproj, Cargo.toml, package.json) en modo raw o wired.
\`\`\`bash
clrinf-codegen module adopt <MODULO> --to-project <PROYECTO> [--mode raw|wired] [--as <ALIAS>]
\`\`\`

### 3. \`init\`
Inicializa el manifiesto \`clrinf.toml\` en un proyecto con autodetección de lenguaje.
\`\`\`bash
clrinf-codegen init --profile standard --dispatcher native
\`\`\`

### 4. \`add module\` y \`add entity\`
Genera módulos de dominio protegidos por Cedar y entidades CRUD mediante workers de lenguaje.
\`\`\`bash
clrinf-codegen add module Orders
clrinf-codegen add entity OrderItem -m Orders --prop name:string --prop price:f64
\`\`\`

### 5. \`generate\` y \`generate-pubsub\`
Genera modelos tipados y esqueletos de Publisher/Subscriber CloudEvents 1.0 a partir de contratos JSON Schema.
\`\`\`bash
clrinf-codegen generate --schema-dir ./schemas --lang all --output ./src
clrinf-codegen generate-pubsub --lang all --output ./src/pubsub
\`\`\`

### 6. \`lint\` y \`topology check\`
Aplica reglas arquitectónicas mediante analizadores AST y verifica la topología de eventos.
\`\`\`bash
clrinf-codegen lint --lang all
clrinf-codegen topology check --schema-dir ./schemas
\`\`\`

### 7. \`mcp\`
Inicia el servidor Model Context Protocol (MCP) JSON-RPC 2.0 para asistentes de programación con IA.
\`\`\`bash
clrinf-codegen mcp
\`\`\`
*(Herramientas: \`clrinf_list_catalog\`, \`clrinf_adopt_module\`, \`clrinf_module_manage\`, \`clrinf_add_domain_module\`, \`clrinf_add_entity\`, \`clrinf_lint_architecture\`, \`clrinf_scaffold_rule\`)*
      `,
    },
  ],
};
