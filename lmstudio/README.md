# LM Studio ile clrinf Yerel Yapay Zeka Geliştirme Rehberi

Bu rehber, **LM Studio** üzerinden çalışan yerel LLM'lerin (özellikle Qwen 2.5 Coder veya DeepSeek Coder) `clrinf` araçlarını, modül adaptasyon motorunu, Amazon Cedar güvenlik politikalarını ve AST linter'larını en yüksek verim ve minimum token harcamasıyla kullanmasını sağlayan mimariyi açıklar.

---

## 1. Neden Klasik RAG Değil, "İsteğe Bağlı Dil Bağlamı + Graf"?

Yerel modellerin (Local LLMs) en hassas noktası **sınırlı bağlam penceresi (context window)** ve çok fazla doküman yüklendiğinde dikkatin (attention) dağılmasıdır.

Klasik vektör tabanlı RAG yönteminde:
- C#, Rust ve TypeScript dokümanları birbirine karışır.
- Model C# yazarken Rust'ın `Result` veya TypeScript'in `pipeRules` örneklerini üretmeye çalışabilir.
- Binlerce tokenlık genel dokümanlar modeli yavaşlatır.

### `clrinf` Çözümü:
1. **İsteğe Bağlı İzole Dil Bağlamı (`clrinf_get_docs`):** Model C# projesinde çalışırken **sadece C#** kurallarını ve Roslyn linter sınırlarını (~250 token) çeker. Rust ve TypeScript dokümanları asla bağlama girmez.
2. **Graft Bilgi Grafı Entegrasyonu (`clrinf_graft_ask`, `clrinf_graft_skeleton`):** Model tüm dosyaları okumak yerine yalnızca hedeflenen sembolün en kritik ≤8 satırlık çekirdeğini (crux) veya iskeletini inceler (~10 kat token tasarrufu).
3. **Otonom Mimari Düzeltme Döngüsü (Self-Healing Loop):** Model kod ürettikten sonra doğrudan `clrinf_lint_architecture` aracını çağırarak derleme zamanı AST kurallarını denetler ve hatalarını anında düzeltir.

---

## 2. Model ve LM Studio Yapılandırması

### Önerilen Modeller
- **1. Tercih (En İyi Sonuç):** `Qwen 2.5 Coder 14B Instruct` veya `32B Instruct` (GGUF Q4_K_M veya Q5_K_M)
  - *Neden:* Kod üretimi, JSON Tool Calling ve talimat takibinde açık kaynak dünyasının en kararlı modelidir.
- **Alternatif:** `DeepSeek Coder V2 Lite` veya `Llama 3.3 70B Instruct` (24GB+ VRAM için).

### LM Studio Parametreleri
- **Context Length:** `16384` veya `32768`
- **Temperature:** `0.1` - `0.2` (Deterministik kod ve tool call için)
- **Chat Template:** `ChatML / Qwen` (Jinja şablonunun tool calling formatını desteklediğinden emin olun)

---

## 3. Kurulum ve MCP (Model Context Protocol) Yapılandırması

### 3.1. Linux / CachyOS / Arch Üzerine Üniversal Kurulum

Linux FHS ve XDG standartlarına (`~/.local/bin`) uygun olarak aracı tek bir komutla kurabilirsiniz:

```bash
# Kullanıcı dizinine kurar (~/.local/bin) ve LM Studio mcp.json dosyasını otomatik yapılandırır
./install.sh

# Alternatif: Sistem geneline kurmak isterseniz (/usr/local/bin)
sudo ./install.sh --system
```

Kurulum tamamlandığında hem `clrinf-codegen` hem de `clrinf` kısayolu PATH'inize eklenir.

### 3.2. LM Studio & Ajan İstemcileri için Üniversal MCP Konfigürasyonu

Kurulum scripti `~/.lmstudio/mcp.json` dosyasını otomatik olarak günceller. Başka bir istemcide (Roo Code, Continue, Cursor vb.) manuel eklemek isterseniz, `lmstudio/mcp_config.json` dosyasındaki üniversal tanımı kullanabilirsiniz:

```json
{
  "mcpServers": {
    "clrinf": {
      "command": "clrinf-codegen",
      "args": ["mcp"],
      "env": {
        "RUST_LOG": "info"
      }
    }
  }
}
```
*(Not: `clrinf-codegen` PATH üzerinde olduğundan herhangi bir kullanıcı adı veya hardcoded dizin içermez.)*

---

## 4. Kullanılabilir MCP Araçları Referansı

| MCP Aracı | Açıklama | Token Tasarrufu |
|---|---|:---:|
| **`clrinf_get_docs`** | Belirtilen dile (`csharp`, `rust`, `typescript`, `cedar`, `cli`) ait izole kuralları getirir. `project_path` verilirse otomatik algılar. | **%90 Tasarruf** (Yalnızca hedef dil) |
| **`clrinf_graft_ask`** | Repo içindeki sembol veya kavramı arar, yalnızca en alakalı kod çekirdeklerini getirir. | **%80 Tasarruf** (Tüm dosyayı okumaz) |
| **`clrinf_graft_skeleton`** | Bir dosyanın fonksiyon ve tip iskeletini çıkarır. | **%90 Tasarruf** |
| **`clrinf_list_catalog`** | Yerleşik domain modüllerini (`crm`, `deals`, `contacts`, `activities`) ve altyapı modüllerini listeler. | Kompakt JSON |
| **`clrinf_adopt_module`** | Modülü hedef projeye (`.csproj`, `Cargo.toml`, `package.json`) ham (`raw`) veya kablolanmış (`wired`) aktarır. | Otonom Entegrasyon |
| **`clrinf_lint_architecture`**| C# (Roslyn), Rust (Syn) ve TypeScript AST analizörlerini çalıştırarak kural ihlallerini döner. | Sıfır Hata Garantisi |
| **`clrinf_scaffold_rule`** | Dilden bağımsız iş kuralı ve birim test şablonu üretir. | Şablon Hızı |
| **`clrinf_add_domain_module`**| Çok kiracılı Amazon Cedar politikasıyla korunan yeni domain modülü üretir. | Standart İskelet |
| **`clrinf_add_entity`** | CRUD repository portları ve bellek içi adaptörleriyle entity üretir. | Standart İskelet |

---

## 5. Sistem Promptu (LM Studio Preset)

Modelinizin sistem promptu alanına [`lmstudio/system_prompt.txt`](file:///home/burak/Projeler/clrinf/lmstudio/system_prompt.txt) dosyasının içeriğini yapıştırın:

```markdown
You are the senior clrinf software architect and engineering assistant.
You operate alongside the clrinf ecosystem to develop polyglot enterprise software (C#, Rust, TypeScript) with zero architectural drift.

1. DYNAMIC & TOKEN-SAVING CONTEXT:
   - Before writing or refactoring code in any project, call `clrinf_get_docs(lang)` (e.g. 'csharp', 'rust', 'typescript', or 'cedar') or provide `project_path`.
   - Never guess language rules or load other language manuals into your context window. Only read the documentation for the language you are actively targeting.

2. REPOSITORY NAVIGATION VIA GRAFT:
   - To find how something works or locate code spans, call `clrinf_graft_ask(query: "...")`.
   - To inspect a file's API surface or structure, call `clrinf_graft_skeleton(file: "...")`.

3. BUILT-IN MODULES & ADOPTION:
   - Call `clrinf_list_catalog` to inspect official modules.
   - Call `clrinf_adopt_module(module, to_project, mode: 'raw' | 'wired')`.

4. CONSTITUTIONAL ARCHITECTURAL BOUNDARIES:
   - C#: Clean Architecture (ARCH001), IBusinessRule (ARCH002), Controller Isolation (ARCH003), CQRS (ARCH004).
   - Rust: Zero-Panic Policy (RUST_ARCH001), never .unwrap() / .expect(). Return Result<T, DomainError>.
   - TypeScript: Functional domain, Result.ok/err (ARCH_TS_001), pipeRules (ARCH_TS_003).
   - Cedar: Multi-tenant by default, forbid precedence, platform admin override.

5. SELF-CORRECTION LOOP:
   - After writing or modifying code, IMMEDIATELY call `clrinf_lint_architecture(lang)`.
   - If the AST analyzer flags any rule violation, fix it immediately before completing your task.
```

---

## 6. Örnek Geliştirme Senaryoları

### Senaryo 1: C# Projesine Satış Fırsatları Modülü Ekleme
1. **Geliştirici:** "Fatura servisime CRM Deals modülünü 'Sales' adıyla ekle ve bir iskonto kontrol kuralı yaz."
2. **Modelin Adımları (Otomatik):**
   - `clrinf_get_docs(lang: "csharp")` çağırır ➔ C# kural setini öğrenir (~200 token).
   - `clrinf_adopt_module(module: "deals", to_project: "./apps/BillingService.csproj", mode: "raw", as: "Sales")` çağırır ➔ Modülü aktarır.
   - Kural sınıfını `IBusinessRule<Deal>` arayüzüne ve `RuleResult` sözleşmesine uygun yazar.
   - `clrinf_lint_architecture(lang: "csharp")` çağırır ➔ Roslyn ARCH kurallarının yeşil olduğunu doğrular.
   - Görevi tamamlar.

### Senaryo 2: Rust Crate'inde Sıfır-Panik İş Kuralı Yazma
1. **Geliştirici:** "Müşteri bakiyesi için Rust tarafında bir kural ekle."
2. **Modelin Adımları (Otomatik):**
   - `clrinf_get_docs(lang: "rust")` çağırır ➔ `RUST_ARCH001` kuralını ve `Result<T, DomainError>` yapısını alır.
   - Kuralı yazar; asla `.unwrap()` veya `panic!()` kullanmaz.
   - `clrinf_lint_architecture(lang: "rust")` ile doğrular.
