# Çalışma zamanı ve sürüm politikası

| Bileşen | Hedef | Politika |
|---|---|---|
| Rust | Stable toolchain, crate manifest gereksinimleri | Cargo.lock uygulamalar/araçlarda korunur |
| C# | .NET 10 | Preview olarak adlandırılmaz; gerekçesiz .NET 8 multi-target eklenmez |
| TypeScript | Bun + TypeScript strict | Bun lockfile korunur, CI frozen install yapar |
| Elixir | Elixir 1.16+, CI 1.17 / OTP 27 | Tam OTP, mix.lock; Docker 1.17 hattı |
| NATS | Yerel referans 2.10.29 | Core köprü ve JetStream yetenekleri ayrı belgelenir |

Destek, bağımlılıkların izin verdiği bütün sürümlerin ölçüldüğü anlamına gelmez.
CI yalnızca tanımladığı kombinasyonları kapsar. Minimum destek sürümü veya dil
kapsamı değişiklikleri kullanıcıya görünür uyumluluk kararıdır.

Root Compose imajlarında `latest` kullanılmaz. Bu bir tekrar üretilebilirlik
tabanıdır, güvenlik onayı değildir. Patch/digest güncellemeleri düzenli olarak
değerlendirilmelidir. Kalıcı veritabanı major sürümleri otomatik değiştirilmez.

Yeni runtime, framework veya lint aracı sırf aynı araç listesine sahip olmak
için eklenmez. Diller kendi doğal hata, async ve paketleme biçimlerini korur.
