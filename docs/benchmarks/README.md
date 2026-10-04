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

Bu depodaki örnek şemalarla (146 dosya) ölçülen değer: baseline ≈ 110.600 token; `lean` MCP profili ≈ 1.167 token, `full` ≈ 3.391 token.

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
