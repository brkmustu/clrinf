# Ajan Korumaları (Agent Guardrails)

clrinf'in mimari korumaları ajan hatırlasa da hatırlamasa da çalışsın diye dört parça sunar. Hepsi tek bir kural dosyasından (`clrinf.rules.toml`) beslenir.

| Parça | Komut / MCP aracı | Ne yapar |
|---|---|---|
| Otomatik doğrulama | `clrinf hook install`, `clrinf verify` / `clrinf_verify` | Her dosya düzenlemesinden sonra yalnızca ilgili kontroller çalışır; ihlaller ajana geri döner |
| Boşluk doldurma planı | `clrinf plan` / `clrinf_plan_change` | Mekanik işleri yapar, ajana yalnızca doldurulacak sembolleri ve okunacak dosyaları söyler |
| Tek kaynaklı kurallar | `clrinf agents sync` | Kuralları `AGENTS.md`, `CLAUDE.md`, `GEMINI.md` ve Cursor kural dosyasına yazar |
| Dar MCP yüzeyi | `clrinf mcp --profile lean` | Oturum başına sabit araç şeması maliyetini düşürür |

## `clrinf.rules.toml`

```toml
[events]
schema_dir = "contracts/schemas"      # varsayılan: tools/clrinf-codegen/schemas

[events.services]                     # drift kontrolü için servis -> kaynak dizini
billing = "services/billing"
orders  = "services/orders"

[[forbid]]                            # bağımlılık kuralı: dizin + yasaklı metin
id      = "domain-no-infra"
paths   = ["services/orders/src/domain"]
pattern = "Infrastructure"
reason  = "Alan katmanı altyapıya bağımlı olamaz"
extensions = ["cs", "ts", "rs"]       # isteğe bağlı

[agents]
extra = ["Handler gövdelerinde yalnızca iş mantığı yaz."]

[mcp]
profile = "lean"                      # full | events | lean
```

Bilinmeyen anahtarlar hata verir (yazım hataları sessizce yutulmaz).

## Kurulum akışı

```bash
clrinf agents sync                    # talimat dosyalarını üret (CI'da: --check)
clrinf hook install --agent all       # Claude Code + Cursor + Antigravity + git pre-commit
clrinf verify                         # tüm proje; --changed ile yalnızca değişenler
```

* **Claude Code hook'u** `.claude/settings.json` içine eklenir; mevcut hook'lar korunur, tekrar çalıştırmak değişiklik yapmaz. Ajan bir dosya yazınca `clrinf hook run` çalışır. İhlal varsa çıkış kodu `2` ile stderr'e yazar; ajan mesajı görür ve düzeltir. Başarıda sessizdir.
* **Cursor hook'u** `.cursor/hooks.json` içine `postToolUse` olarak eklenir (`--format cursor`). Cursor'ın `afterFileEdit` hook'u çıktı kabul etmediği için geri bildirim yapılamaz; bu yüzden `postToolUse` seçildi. İhlal, çıkış kodu `0` ile stdout'a `{"additional_context": ...}` olarak döner. Cursor ayrıca "Third Party Hooks" ayarı açıksa `.claude/settings.json` hook'unu da okuyabilir.
* **Antigravity hook'u** `.agents/hooks.json` içine `clrinf` adlı hook olarak eklenir (`PostToolUse`, `--format antigravity`). Antigravity komut için **mutlak yol** istediğinden yükleyici çalışan `clrinf` ikilisinin mutlak yolunu yazar; ikiliyi taşırsanız `hook install --agent antigravity` komutunu yeniden çalıştırın. İhlal, çıkış kodu `0` ile stdout'a `{"decision":"block","reason":...}` olarak döner.
* **Git hook'u** `.git/hooks/pre-commit` olarak yazılır; başka bir hook varsa `--force` olmadan üzerine yazmaz.
> **Doğrulama durumu:** Claude Code ve Cursor biçimleri resmi dokümana göre yazıldı. Antigravity'nin hook şeması (`.agents/hooks.json`, `PostToolUse`, stdin/stdout JSON) resmi sayfadan doğrudan doğrulanamadı; ikincil kaynaklara dayanıyor. Antigravity'de gerçek bir oturumda denenmeden "çalışıyor" kabul edilmemeli; sorun bulursanız `tools/clrinf-codegen/src/hook.rs` içindeki `Format::Antigravity` ve eşleştirici sabiti tek noktada düzeltilir. Hook'lar ek güvencedir; asıl güvence `git pre-commit` ve CI'daki `clrinf verify`'dır.

* `verify` şunları çalıştırır: topoloji, `clrinf.rules.toml` içindeki servisler için kod↔sözleşme drift'i, `[[forbid]]` kuralları ve değişen dosyalardaki doğrudan outbox yazımları.

## Boşluk doldurma planı

```bash
clrinf plan orders.placed.v1 --service billing --role subscriber   # önizleme
clrinf plan orders.placed.v1 --service billing --role subscriber --apply
```

Çıktı (JSON): `schema_edit`, `fill_in` (dil başına doldurulacak sembol ve üretilmiş dosya yolu), `read_these` (okunması gereken elle yazılmış dosyalar), `do_not_edit` (üretilmiş dosyalar), `constraints` ve `open_issues`.

## MCP profilleri

| Profil | Araç sayısı | Kullanım |
|---|---|---|
| `full` (varsayılan) | 25 | Tüm araçlar |
| `events` | 12 | Olay odaklı geliştirme |
| `lean` | 8 | Niyet düzeyi araçlar (`clrinf_plan_change`, `clrinf_verify`, …); küçük/yerel modeller için |

Seçim önceliği: `--profile` > `CLRINF_MCP_PROFILE` > `clrinf.rules.toml` `[mcp]` > `full`. Profil dışı bir aracı çağırmak hata döndürür.
