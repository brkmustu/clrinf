# Benchmark Protokolü

"clrinf ile daha az token, daha az mimari ihlal" iddiası iki katmanda ölçülür.

## 1. Deterministik bağlam maliyeti (`clrinf bench context`)

```bash
clrinf bench context <event> --schema-dir <schemas> --src <kaynak-kökleri...>
```

Ölçtükleri:

* **Baseline**: `--src` altındaki tüm kaynak dosyalar + tüm şemalar (ajanın "keşfet" yaklaşımı).
* **Slice**: olay şeması + olayı anan elle yazılmış dosyalar + üretilmiş kural bloğu.
* **MCP şema maliyeti**: `full`, `events`, `lean` profillerinin oturum başına sabit token yükü.

Token tahmini `~4 karakter/token` kuralıdır; mutlak değerden çok oranlar anlamlıdır.

> [!WARNING]
> Bu yalnızca **okuma** tarafını ölçer. Ajan yeni kod da yazar; yazım maliyeti ve düzeltme turları ancak aşağıdaki A/B çalıştırmasıyla görülür. Olayı henüz hiçbir elle yazılmış dosya anmıyorsa slice yalnızca şemadır ve oran olduğundan iyi görünür. Bu sayıyı tek başına pazarlama iddiası olarak kullanmayın.

### Ölçülen Deterministik Sonuçlar (`scripts/bench-guardrails.ps1` / `.sh`)

Aşağıdaki değerler `clrinf` CLI'ı ile doğrudan bu repoda ölçülmüş, deterministik olarak tekrarlanabilir sonuçlardır:

#### 1. Çok Dilli Olay Dilimleme & Hata Yakalama (`examples/guardrails-benchmark/`)
- **Temiz taban çizgisi (`clrinf verify`):** 0 hata (Rust + TypeScript + C# servisleri).
- **Bağlam maliyeti (`orders.placed.v1`):**
  - Baseline (`services/` + tüm şemalar): **2.015 token** (15 dosya)
  - `clrinf` slice (ilgili şema + el yazımı 3 servis dosyası + kural bloğu): **1.182 token** (4 dosya) → **%41,3 azalma**
- **Oturum başı sabit MCP araç şeması maliyeti:**
  - `full` (27 araç): **3.698 token**
  - `events` (14 araç): **1.983 token**
  - `lean` (10 araç): **1.440 token** (`full`'a göre **%61,1 daha az sabit yük**)
- **Enjekte edilen ajan hatalarının yakalanması (3/3 deterministik):**
  1. Domain katmanında yasaklı import (`RULE_FORBIDDEN_DEPENDENCY`, `scope = "imports"`) + Claude Code `PostToolUse` hook'undan **çıkış kodu `2`** geri bildirimi.
  2. Doğrudan outbox yazımı (`DRIFT_RAW_OUTBOX_WRITE`).
  3. Upcaster olmadan kırıcı `v2` şema ekleme (`TOPOLOGY_UPCASTER_MISSING`).

#### 2. Derleme Anı AOT CRUD vs. Disk Üretimli CRUD (`Category` + `Brand`)
- **Geleneksel disk üretimi (`clrinf add entity --pattern vertical-slice`):** **3.850 token** (16 `.cs` dosyası).
- **Native AOT CRUD (`core/csharp/examples/AotCrud/Domain/*.cs`):** **356 token** (3 `.cs` dosyası) → diskteki ajan bağlamında **%90,8 token azalması**.

Tekrarlamak için:
```powershell
.\scripts\bench-guardrails.ps1   # Windows
bash scripts/bench-guardrails.sh # Linux / macOS
```

## 2. Uçtan uca ajan A/B çalıştırması

Aynı görev, aynı model, aynı başlangıç commit'i; yalnızca clrinf açık/kapalı.

| Kol | Kurulum |
|---|---|
| A (kontrol) | clrinf yok; ajan repoyu kendisi keşfeder |
| B (clrinf) | `clrinf agents sync`, `clrinf hook install`, MCP `lean` profili |

### Araç bazlı B kolu kurulumu

Gönüllüler farklı araçlar kullanabilir; B kolunu hepsi aynı kural dosyasından kurar. Çalıştırdığınız aracı sonuç tablosuna mutlaka yazın (araç değişkeni sonuçları etkiler; kollar aynı araçta karşılaştırılmalıdır).

| Araç | Talimat dosyası | Hook kurulumu | MCP |
|---|---|---|---|
| Claude Code | CLAUDE.md | `clrinf hook install --agent claude` | `clrinf mcp --profile lean` |
| Cursor | `.cursor/rules/clrinf.mdc` | `clrinf hook install --agent cursor` | `.cursor/mcp.json` içinde `clrinf mcp --profile lean` |
| Antigravity | `GEMINI.md` / `AGENTS.md` | `clrinf hook install --agent antigravity` | MCP ayarlarında `clrinf mcp --profile lean` |
| Diğer / hook'suz | `AGENTS.md` | `clrinf hook install --agent git` (commit anında) | `clrinf mcp --profile lean` |

Tüm araçlar için ilk adım `clrinf agents sync`'tir. `clrinf hook install --agent all` hepsini birden kurar. Hook'un çalıştığını doğrulamak için alan katmanına yasaklı bir import ekletip ajanın ihlal mesajını görüp görmediğine bakın; görmüyorsa sonuçlara "hook etkisiz" notu düşün (Antigravity hook şeması resmi dokümandan doğrulanamadı, bkz. `docs/guides/agent-guardrails.md`).
Önerilen görevler (her biri en az 5 tekrar):

1. Mevcut bir olaya yeni bir servis abone olsun.
2. Bir olay şemasına zorunlu alan eklensin (v1 → v2 geçişi, upcaster dahil).
3. Alan katmanına altyapı bağımlılığı ekleme baskısı olan bir özellik istensin.

Kaydedilecek metrikler:

| Metrik | Nasıl alınır |
|---|---|
| Toplam token (girdi + çıktı) | Ajan/sağlayıcı kullanım raporu |
| Tur / araç çağrısı sayısı | Oturum kaydı |
| Mimari ihlal sayısı | Görev sonunda `clrinf verify --json` → `errorsCount` |
| Görev başarısı | Önceden yazılmış kabul testleri |
| Elle müdahale | İnsanın düzeltmek zorunda kaldığı satır sayısı |

Sonuçları medyan ve aralıkla raporlayın; modeli ve sürümünü belirtin.
