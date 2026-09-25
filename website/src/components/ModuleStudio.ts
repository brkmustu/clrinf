import { t } from "../i18n";

type ModuleId =
  | "crm"
  | "deals"
  | "contacts"
  | "activities"
  | "authz"
  | "caching"
  | "logging"
  | "transaction"
  | "idempotency"
  | "outbox";

type AdoptMode = "raw" | "wired";
type TargetLang = "csharp" | "rust" | "ts";
type CodeTab = "code" | "cedar" | "wiring";

interface ModuleMeta {
  id: ModuleId;
  name: string;
  icon: string;
  category: "domain" | "crossCutting";
  desc: string;
  entities: string[];
}

const MODULES: ModuleMeta[] = [
  {
    id: "crm",
    name: "CRM Full Suite",
    icon: "🏢",
    category: "domain",
    desc: "B2B CRM Paketi (Deals + Contacts + Activities + birleşik Cedar güvenlik muhafızı).",
    entities: ["Deal", "Contact", "Activity"],
  },
  {
    id: "deals",
    name: "CRM Deals / Pipeline",
    icon: "💼",
    category: "domain",
    desc: "Fırsat hunisi (Prospect, Qualified, Won, Lost), aşama kuralları ve Cedar politikası.",
    entities: ["Deal"],
  },
  {
    id: "contacts",
    name: "CRM Contacts / Müşteriler",
    icon: "👥",
    category: "domain",
    desc: "Müşteri ve aday yönetimi, RFC e-posta doğrulaması, kiracı mükerrerlik denetimi.",
    entities: ["Contact"],
  },
  {
    id: "activities",
    name: "CRM Activities / Görevler",
    icon: "📅",
    category: "domain",
    desc: "Görüşmeler, aramalar ve görev takibi, kayıp anlaşma koruma kuralı.",
    entities: ["Activity"],
  },
  {
    id: "authz",
    name: "Cedar Multi-Tenant Authz",
    icon: "🛡️",
    category: "crossCutting",
    desc: "Amazon Cedar tabanlı, varsayılan çok kiracılı (multi-tenant by default) yetkilendirme motoru.",
    entities: [],
  },
  {
    id: "caching",
    name: "Caching / In-Memory & Redis",
    icon: "⚡",
    category: "crossCutting",
    desc: "Yüksek başarımlı önbellekleme soyutlaması (ICacheService, MemoryCache).",
    entities: [],
  },
  {
    id: "logging",
    name: "Structured Logging",
    icon: "📝",
    category: "crossCutting",
    desc: "Yapılandırılmış loglama ve işlem yürütme zamanı ölçüm hattı.",
    entities: [],
  },
  {
    id: "transaction",
    name: "Transaction & Unit of Work",
    icon: "🔒",
    category: "crossCutting",
    desc: "İşlem sınırları ve atomik Unit of Work yönetimi.",
    entities: [],
  },
  {
    id: "idempotency",
    name: "Idempotency Store",
    icon: "🔁",
    category: "crossCutting",
    desc: "Kiracı-yalıtımlı mükerrer işlem engelleme portu (claim/complete/release).",
    entities: [],
  },
  {
    id: "outbox",
    name: "Transactional Outbox",
    icon: "📬",
    category: "crossCutting",
    desc: "Güvenilir, kayıpsız olay dağıtımı için transactional outbox kuyruğu.",
    entities: [],
  },
];

export function renderModuleStudio(): HTMLElement {
  const section = document.createElement("section");
  section.className = "section";
  section.id = "module-studio";

  let activeModule: ModuleId = "crm";
  let activeMode: AdoptMode = "raw";
  let activeLang: TargetLang = "csharp";
  let activeTab: CodeTab = "code";

  function getCliCommand(): string {
    const mod = activeModule;
    let projArg = "";
    let targetDirArg = "";

    switch (activeLang) {
      case "csharp":
        projArg = "./apps/BillingService/BillingService.csproj";
        targetDirArg = mod === "crm" ? "src/Features/Crm" : `src/Features/${capitalize(mod)}`;
        break;
      case "rust":
        projArg = "./crates/crm-service/Cargo.toml";
        targetDirArg = mod === "crm" ? "src/crm" : `src/${mod}`;
        break;
      case "ts":
        projArg = "./packages/storefront/package.json";
        targetDirArg = mod === "crm" ? "src/modules/crm" : `src/modules/${mod}`;
        break;
    }

    let cmd = `cargo run --manifest-path tools/clrinf-codegen/Cargo.toml -- module adopt ${mod} \\\n  --to-project ${projArg} \\\n  --target-dir ${targetDirArg} \\\n  --mode ${activeMode}`;
    return cmd;
  }

  function getGeneratedFiles(): { name: string; type: "code" | "policy" | "wiring" }[] {
    const files: { name: string; type: "code" | "policy" | "wiring" }[] = [];

    if (activeLang === "csharp") {
      if (activeModule === "crm") {
        files.push({ name: "src/Features/Crm/Deals/DealsModule.cs", type: "code" });
        files.push({ name: "src/Features/Crm/Contacts/ContactsModule.cs", type: "code" });
        files.push({ name: "src/Features/Crm/Activities/ActivitiesModule.cs", type: "code" });
        if (activeMode === "wired") {
          files.push({ name: "src/Features/Crm/CrmServiceRegistration.cs", type: "wiring" });
        }
      } else {
        files.push({ name: `src/Features/${capitalize(activeModule)}/${capitalize(activeModule)}Module.cs`, type: "code" });
      }
      files.push({ name: `policies/${activeModule}.cedar`, type: "policy" });
      if (activeMode === "wired") {
        files.push({ name: "src/Common/ServiceRegistration.cs [güncellendi]", type: "wiring" });
      }
    } else if (activeLang === "rust") {
      if (activeModule === "crm") {
        files.push({ name: "src/crm/mod.rs", type: "code" });
        files.push({ name: "src/crm/deals/mod.rs", type: "code" });
        files.push({ name: "src/crm/contacts/mod.rs", type: "code" });
        files.push({ name: "src/crm/activities/mod.rs", type: "code" });
      } else {
        files.push({ name: `src/${activeModule}/mod.rs`, type: "code" });
      }
      files.push({ name: `policies/${activeModule}.cedar`, type: "policy" });
      if (activeMode === "wired") {
        files.push({ name: "src/lib.rs [pub mod ...; güncellendi]", type: "wiring" });
      }
    } else {
      if (activeModule === "crm") {
        files.push({ name: "src/modules/crm/deals.ts", type: "code" });
        files.push({ name: "src/modules/crm/contacts.ts", type: "code" });
        files.push({ name: "src/modules/crm/activities.ts", type: "code" });
        files.push({ name: "src/modules/crm/index.ts", type: "code" });
      } else {
        files.push({ name: `src/modules/${activeModule}/index.ts`, type: "code" });
      }
      files.push({ name: `policies/${activeModule}.cedar`, type: "policy" });
      if (activeMode === "wired") {
        files.push({ name: "src/index.ts [export * as ...; güncellendi]", type: "wiring" });
      }
    }

    return files;
  }

  function getCodeSnippet(): string {
    const isRaw = activeMode === "raw";

    if (activeLang === "csharp") {
      if (activeModule === "crm" || activeModule === "deals") {
        return `// @clrinf:generated — ${isRaw ? 'Sıfır Harici Bağımlılık (Pure Raw Mode)' : 'Tam Ekosistem Bağlantısı (Wired Mode)'}
using System;
using System.Collections.Concurrent;
using System.Threading.Tasks;

namespace BillingService.Features.Sales;

// ─── 1. Kendi Kendine Yeten Sözleşmeler (Harici Nuget/Framework Dayatması Yok) ──
public sealed record OperationClaim(string Resource, string Action, string? TenantId = null)
{
    public static OperationClaim WithTenant(string resource, string action, string tenantId) =>
        new(resource, action, tenantId);
}

public interface IRequireOperationClaim
{
    OperationClaim GetRequiredClaim();
}

public interface IRequest<out TResponse> { }

// ─── 2. Domain Varlığı & CQRS Komutları ─────────────────────────────────────────
public sealed record Deal(string Id, string TenantId, string Title, decimal Amount, string Stage);

public sealed record CreateDealCommand(string TenantId, string Title, decimal Amount)
    : IRequest<Deal>, IRequireOperationClaim
{
    public OperationClaim GetRequiredClaim() =>
        OperationClaim.WithTenant("Deals", "deals.create", TenantId);
}

// ─── 3. Bellek-İçi Test Edilebilir Repository Portu ─────────────────────────────
public interface IDealRepository
{
    Task<Deal?> GetByIdAsync(string tenantId, string id);
    Task SaveAsync(Deal deal);
}

public sealed class InMemoryDealRepository : IDealRepository
{
    private readonly ConcurrentDictionary<string, Deal> _store = new();
    public Task<Deal?> GetByIdAsync(string tenantId, string id) =>
        Task.FromResult(_store.TryGetValue($"\${tenantId}:\${id}", out var d) ? d : null);
    public Task SaveAsync(Deal deal)
    {
        _store[$"\${deal.TenantId}:\${deal.Id}"] = deal;
        return Task.CompletedTask;
    }
}`;
      } else if (activeModule === "authz") {
        return `// @clrinf:generated — Amazon Cedar Multi-Tenant Yetkilendirme Motoru (C#)
using System.Threading.Tasks;

namespace BillingService.Common.Authz;

public sealed record CedarClaim(
    string Role,
    string PrincipalTenantId,
    string Action,
    string ResourceType,
    string ResourceTenantId,
    string ContextTenantId
);

public interface ICedarAuthorizationService
{
    Task<bool> AuthorizeAsync(CedarClaim claim);
}

public sealed class DefaultCedarAuthorizationService : ICedarAuthorizationService
{
    public Task<bool> AuthorizeAsync(CedarClaim claim)
    {
        // 1. Süper kullanıcı PlatformAdmin tüm kiracı sınırlarını aşabilir
        if (claim.Role == "PlatformAdmin") return Task.FromResult(true);

        // 2. KATI KURAL: Kiracılar arası veri sızıntısı anında reddedilir (Forbid Guard)
        if (claim.PrincipalTenantId != claim.ResourceTenantId || claim.ContextTenantId != claim.PrincipalTenantId)
            return Task.FromResult(false);

        // 3. Kiracı içi rol denetimi
        return Task.FromResult(true);
    }
}`;
      } else {
        return `// @clrinf:generated — ${capitalize(activeModule)} Modülü (${isRaw ? 'Raw' : 'Wired'})
namespace BillingService.Common.${capitalize(activeModule)};

public interface I${capitalize(activeModule)}Service
{
    void Execute(string tenantId);
}`;
      }
    } else if (activeLang === "rust") {
      if (activeModule === "crm" || activeModule === "deals") {
        return `// @clrinf:generated — Rust Sıfır-Panik Modülü (${isRaw ? 'Raw (Zero clrinf_core Dependency)' : 'Wired'})
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

// ─── 1. Kendi Kendine Yeten Sözleşmeler ─────────────────────────────────────────
#[derive(Debug, Clone, PartialEq)]
pub struct OperationClaim {
    pub resource: String,
    pub action: String,
    pub tenant_id: Option<String>,
}

impl OperationClaim {
    pub fn with_tenant(resource: &str, action: &str, tenant_id: &str) -> Self {
        Self {
            resource: resource.to_string(),
            action: action.to_string(),
            tenant_id: Some(tenant_id.to_string()),
        }
    }
}

// ─── 2. Domain Varlığı ve Komutları ─────────────────────────────────────────────
#[derive(Debug, Clone, PartialEq)]
pub struct Deal {
    pub id: String,
    pub tenant_id: String,
    pub title: String,
    pub amount: f64,
    pub stage: String,
}

// ─── 3. In-Memory Repository Portu (Zero-Panic) ─────────────────────────────────
#[derive(Clone, Default)]
pub struct InMemoryDealRepository {
    store: Arc<RwLock<HashMap<String, Deal>>>,
}

impl InMemoryDealRepository {
    pub fn new() -> Self {
        Self { store: Arc::new(RwLock::new(HashMap::new())) }
    }
    pub async fn get_by_id(&self, tenant_id: &str, id: &str) -> Option<Deal> {
        let key = format!("{}:{}", tenant_id, id);
        let guard = self.store.read().await;
        guard.get(&key).cloned()
    }
}`;
      } else {
        return `// @clrinf:generated — Rust ${capitalize(activeModule)} Modülü
pub struct ${capitalize(activeModule)}Module;

impl ${capitalize(activeModule)}Module {
    pub fn init() -> Self { Self }
}`;
      }
    } else {
      // TypeScript
      return `// @clrinf:generated — TypeScript Monadic Modülü (${isRaw ? 'Raw / Zero-Dependency' : 'Wired'})
export interface OperationClaim {
  resource: string;
  action: string;
  tenantId?: string;
}

export interface Deal {
  id: string;
  tenantId: string;
  title: string;
  amount: number;
  stage: string;
}

export class InMemoryDealRepository {
  private store = new Map<string, Deal>();

  async getById(tenantId: string, id: string): Promise<Deal | null> {
    return this.store.get(\`\${tenantId}:\${id}\`) || null;
  }

  async save(deal: Deal): Promise<void> {
    this.store.set(\`\${deal.tenantId}:\${deal.id}\`, deal);
  }
}`;
    }
  }

  function getCedarPolicy(): string {
    const cap = capitalize(activeModule);
    return `// @clrinf:generated — Multi-Tenant Cedar Security Policy: ${cap}
// Dil ve çalışma ortamından bağımsız Amazon Cedar doğrulaması

// 1. Platform Yöneticisi: Tüm kiracılarda tam yetki (Superuser bypass)
permit (
    principal in Role::"PlatformAdmin",
    action,
    resource
);

// 2. Kiracı Yöneticisi: Sadece kendi kiracısında tam yönetim
permit (
    principal in Role::"TenantAdmin",
    action,
    resource in ResourceType::"${cap}"
) when {
    resource.tenant_id == principal.tenant_id
};

// 3. Standart Kullanıcı: Kendi kiracısında okuma/yazma
permit (
    principal in Role::"StandardUser",
    action in [Action::"${activeModule}.read", Action::"${activeModule}.create"],
    resource in ResourceType::"${cap}"
) when {
    context.tenant_id == principal.tenant_id &&
    resource.tenant_id == principal.tenant_id
};

// 4. KATI KURAL: Kiracılar Arası Veri Sızıntısına Karşı Mutlak Savunma Hattı
// Forbid kuralı Cedar'da tüm permit kurallarını ezer!
forbid (
    principal,
    action,
    resource in ResourceType::"${cap}"
) when {
    resource.tenant_id != principal.tenant_id
};`;
  }

  function getWiringSnippet(): string {
    if (activeMode === "raw") {
      return `// Raw Mod devrede:
// Harici hiçbir DI konteyneri veya modül ağacı kaydı zorunlu değildir.
// Modül tamamen saf ve izole olarak derlenir.`;
    }

    if (activeLang === "csharp") {
      return `// C# DI Kablolaması (ServiceRegistration.cs)
using Microsoft.Extensions.DependencyInjection;
using BillingService.Features.${capitalize(activeModule)};

public static class ServiceRegistration
{
    public static IServiceCollection AddClrinfModules(this IServiceCollection services)
    {
        services.Add${capitalize(activeModule)}Module();
        return services;
    }
}`;
    } else if (activeLang === "rust") {
      return `// Rust Derleme Zamanı Modül Ağacı (src/lib.rs)
pub mod ${activeModule.toLowerCase()};

// Açık struct & trait kompozisyonu ile bağlanır (sıfır runtime DI overhead)`;
    } else {
      return `// TypeScript Barrel Export (src/index.ts)
export * as ${activeModule.toLowerCase()} from "./modules/${activeModule.toLowerCase()}/index.js";`;
    }
  }

  function renderContent() {
    const curT = t();
    const studioT = curT.moduleStudio;

    const moduleButtonsHtml = MODULES.map(
      (m) => `
      <button class="mod-studio-btn ${m.id === activeModule ? "active" : ""}" data-mod="${m.id}">
        <span class="mod-icon">${m.icon}</span>
        <span class="mod-name">${m.name}</span>
        ${m.category === "domain" ? '<span class="mod-badge">Domain</span>' : '<span class="mod-badge concern">Concern</span>'}
      </button>
    `
    ).join("");

    const fileListHtml = getGeneratedFiles()
      .map((f) => {
        const icon = f.type === "policy" ? "🛡️" : f.type === "wiring" ? "⚙️" : "📄";
        const cls = f.type === "policy" ? "file-policy" : f.type === "wiring" ? "file-wiring" : "file-code";
        return `
          <div class="studio-file-item ${cls}">
            <span>${icon}</span>
            <span>${f.name}</span>
          </div>
        `;
      })
      .join("");

    let codeToShow = "";
    if (activeTab === "code") {
      codeToShow = getCodeSnippet();
    } else if (activeTab === "cedar") {
      codeToShow = getCedarPolicy();
    } else {
      codeToShow = getWiringSnippet();
    }

    section.innerHTML = `
      <div class="container">
        <div class="section-header">
          <span class="section-tag">${studioT.tag}</span>
          <h2 class="section-title">${studioT.title}</h2>
          <p class="section-subtitle">${studioT.subtitle}</p>
        </div>

        <div class="studio-card">
          <!-- Studio Top Controls -->
          <div class="studio-controls-grid">
            <!-- 1. Module Selector -->
            <div class="studio-control-group">
              <label class="studio-label">
                <span>${studioT.selectModule}</span>
                <span class="badge-tag">${studioT.catalogBadge}</span>
              </label>
              <div class="mod-studio-list">
                ${moduleButtonsHtml}
              </div>
            </div>

            <!-- 2. Mode & Target Config -->
            <div class="studio-config-panel">
              <!-- Mode Selection (Raw vs Wired) -->
              <div class="studio-control-group">
                <label class="studio-label">${studioT.selectMode}</label>
                <div class="studio-mode-toggle">
                  <button class="mode-btn ${activeMode === "raw" ? "active raw" : ""}" data-mode="raw">
                    <span class="mode-dot raw"></span>
                    <span class="mode-title">${studioT.modes.raw.title}</span>
                  </button>
                  <button class="mode-btn ${activeMode === "wired" ? "active wired" : ""}" data-mode="wired">
                    <span class="mode-dot wired"></span>
                    <span class="mode-title">${studioT.modes.wired.title}</span>
                  </button>
                </div>
                <p class="mode-desc">
                  ${activeMode === "raw" ? studioT.modes.raw.desc : studioT.modes.wired.desc}
                </p>
              </div>

              <!-- Language / Project Selection -->
              <div class="studio-control-group" style="margin-top: 20px;">
                <label class="studio-label">${studioT.selectTarget}</label>
                <div class="studio-lang-segmented">
                  <button class="lang-btn-studio ${activeLang === "csharp" ? "active" : ""}" data-target-lang="csharp">
                    🔷 ${studioT.targets.csharp}
                  </button>
                  <button class="lang-btn-studio ${activeLang === "rust" ? "active" : ""}" data-target-lang="rust">
                    🦀 ${studioT.targets.rust}
                  </button>
                  <button class="lang-btn-studio ${activeLang === "ts" ? "active" : ""}" data-target-lang="ts">
                    🟨 ${studioT.targets.ts}
                  </button>
                </div>
              </div>
            </div>
          </div>

          <!-- Studio Terminal CLI Command Bar -->
          <div class="studio-cli-bar">
            <div class="cli-label-wrap">
              <span class="cli-icon">❯_</span>
              <span class="cli-label">${studioT.cliCommandLabel}</span>
            </div>
            <div class="cli-code-wrap">
              <code>${escapeHtml(getCliCommand())}</code>
              <button class="btn btn-secondary btn-sm" id="studio-copy-cmd">
                <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                  <rect x="9" y="9" width="13" height="13" rx="2" ry="2"></rect>
                  <path d="M5 15H4a2 2 0 0 1-2-2V4a2 2 0 0 1 2-2h9a2 2 0 0 1 2 2v1"></path>
                </svg>
                <span id="copy-btn-text">${studioT.copyBtn}</span>
              </button>
            </div>
          </div>

          <!-- Studio Bottom Panels: File Tree + Code/Cedar Viewer -->
          <div class="studio-preview-grid">
            <!-- Left: Generated Files Tree -->
            <div class="studio-file-tree">
              <div class="tree-header">
                <span>📁 ${studioT.generatedFilesLabel}</span>
                <span class="badge-tag">${getGeneratedFiles().length} dosya</span>
              </div>
              <div class="tree-body">
                ${fileListHtml}
              </div>
            </div>

            <!-- Right: Code / Cedar Tabs & Preview -->
            <div class="studio-code-panel">
              <div class="studio-code-header">
                <div class="code-tabs">
                  <button class="code-tab-btn ${activeTab === "code" ? "active" : ""}" data-code-tab="code">
                    ${studioT.codePreviewTabs.code}
                  </button>
                  <button class="code-tab-btn ${activeTab === "cedar" ? "active" : ""}" data-code-tab="cedar">
                    🛡️ ${studioT.codePreviewTabs.cedar}
                  </button>
                  <button class="code-tab-btn ${activeTab === "wiring" ? "active" : ""}" data-code-tab="wiring">
                    ⚙️ ${studioT.codePreviewTabs.wiring}
                  </button>
                </div>
              </div>
              <div class="studio-code-body">
                <pre><code>${escapeHtml(codeToShow)}</code></pre>
              </div>
            </div>
          </div>
        </div>
      </div>
    `;

    // Attach event listeners
    section.querySelectorAll<HTMLButtonElement>(".mod-studio-btn").forEach((btn) => {
      btn.addEventListener("click", () => {
        const id = btn.dataset.mod as ModuleId;
        if (id) {
          activeModule = id;
          renderContent();
        }
      });
    });

    section.querySelectorAll<HTMLButtonElement>(".mode-btn").forEach((btn) => {
      btn.addEventListener("click", () => {
        const mode = btn.dataset.mode as AdoptMode;
        if (mode) {
          activeMode = mode;
          renderContent();
        }
      });
    });

    section.querySelectorAll<HTMLButtonElement>(".lang-btn-studio").forEach((btn) => {
      btn.addEventListener("click", () => {
        const lang = btn.dataset.targetLang as TargetLang;
        if (lang) {
          activeLang = lang;
          renderContent();
        }
      });
    });

    section.querySelectorAll<HTMLButtonElement>(".code-tab-btn").forEach((btn) => {
      btn.addEventListener("click", () => {
        const tab = btn.dataset.codeTab as CodeTab;
        if (tab) {
          activeTab = tab;
          renderContent();
        }
      });
    });

    const copyBtn = section.querySelector<HTMLButtonElement>("#studio-copy-cmd");
    const copyText = section.querySelector<HTMLElement>("#copy-btn-text");
    if (copyBtn && copyText) {
      copyBtn.addEventListener("click", () => {
        navigator.clipboard.writeText(getCliCommand().replace(/\\\n\s*/g, " ")).then(() => {
          copyBtn.classList.add("btn-primary");
          copyText.textContent = studioT.copiedBtn;
          setTimeout(() => {
            copyBtn.classList.remove("btn-primary");
            copyText.textContent = studioT.copyBtn;
          }, 1800);
        });
      });
    }
  }

  renderContent();
  return section;
}

function capitalize(s: string): string {
  if (!s) return s;
  return s.charAt(0).toUpperCase() + s.slice(1);
}

function escapeHtml(str: string): string {
  return str
    .replace(/&/g, "&amp;")
    .replace(/</g, "&lt;")
    .replace(/>/g, "&gt;");
}
