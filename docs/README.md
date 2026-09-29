# clrinf dokümantasyonu

clrinf herhangi bir iş alanı için sözleşme öncelikli, çok dilli dağıtık uygulama
ve modüler monolith altyapısıdır. Aşağıdaki belgeler ürün sınırlarını, mevcut
yetenekleri ve adaptör seçimlerini anlatır; hedef tasarım ile uygulanmış
özelliği birbirinin yerine kullanmaz.

## Başlangıç

- [Rust](getting-started/01-rust-clrinfrs.md)
- [C#](getting-started/02-csharp-clrinfcs.md)
- [TypeScript](getting-started/03-typescript-clrinfjs.md)
- [Elixir](getting-started/04-elixir-clrinfex.md)
- [Modüler monolith](getting-started/modular-monolith.md)
- [Mevcut projeye aşamalı entegrasyon](getting-started/existing-project.md)

## Mimari

- [Çekirdek, adaptör ve örnek sınırları](architecture/core-and-adapters.md)
- [Yetenek matrisi](architecture/parity-matrix.md)
- [Dayanıklılık, outbox ve idempotency](architecture/reliability.md)
- [Yerel işlemler, dağıtık tutarlılık ve Saga](architecture/distributed-transaction-antipatterns.md)
- [Event Inspector](architecture/event-inspector-guide.md)
- [Terminoloji](blueprints/00-terminoloji-ve-alan-sozlugu.md)

## Operasyon ve yönetişim

- [Yerel geliştirme profilleri](operations/local-development.md)
- [Desteklenen sürümler](operations/support-policy.md)
- [Güvenlik ve üretime hazırlık](operations/security-checklist.md)
- [İsteğe bağlı Nomad dağıtımı](operations/nomad-deployment-guide.md)
- [Ajan bağlamı](governance/agent-context-guide.md)
- [EARS gereksinimleri](governance/ears-requirements-guide.md)
- [Yeniden organizasyon durumu](governance/reorganization-status.md)
- [Anayasa](governance/constitution.md)
- [Sürümleme](governance/versioning.md)

Örnek iş alanı tasarım notları [showcase altında](../templates/eticaret-showcase/docs/)
yer alır.
