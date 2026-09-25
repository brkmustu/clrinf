import { t } from "../i18n";

export interface HeroProps {
  onNavigate: (view: "home" | "docs", hash?: string) => void;
}

export function renderHero(props: HeroProps): HTMLElement {
  const hero = document.createElement("section");
  hero.className = "hero";
  hero.id = "home";

  const curT = t();

  hero.innerHTML = `
    <div class="hero-inner">
      <div class="hero-badge">
        <span class="pulse-dot"></span>
        <span>${curT.hero.badge}</span>
      </div>

      <h1 class="hero-title">
        ${curT.hero.titleStart} 
        <span class="gradient-text">${curT.hero.titleHighlight}</span>
        <br />${curT.hero.titleEnd}
      </h1>

      <p class="hero-subtitle">
        ${curT.hero.subtitle}
      </p>

      <div class="hero-actions">
        <button class="btn btn-primary" id="hero-btn-quickstart">
          <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
            <polygon points="5 3 19 12 5 21 5 3"></polygon>
          </svg>
          <span>${curT.hero.ctaQuickstart}</span>
        </button>

        <button class="btn btn-secondary" id="hero-btn-simulator">
          <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
            <circle cx="18" cy="5" r="3"></circle>
            <circle cx="6" cy="12" r="3"></circle>
            <circle cx="18" cy="19" r="3"></circle>
            <line x1="8.59" y1="13.51" x2="15.42" y2="17.49"></line>
            <line x1="15.41" y1="6.51" x2="8.59" y2="10.49"></line>
          </svg>
          <span>${curT.hero.ctaSimulator}</span>
        </button>

        <button class="btn btn-secondary" id="hero-btn-docs">
          <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
            <path d="M4 19.5A2.5 2.5 0 0 1 6.5 17H20"></path>
            <path d="M6.5 2H20v20H6.5A2.5 2.5 0 0 1 4 19.5v-15A2.5 2.5 0 0 1 6.5 2z"></path>
          </svg>
          <span>${curT.hero.ctaDocs}</span>
        </button>
      </div>

      <!-- Terminal Interactive Preview -->
      <div class="terminal-card" id="hero-terminal">
        <div class="terminal-header">
          <div class="terminal-controls">
            <span class="terminal-dot dot-red"></span>
            <span class="terminal-dot dot-yellow"></span>
            <span class="terminal-dot dot-green"></span>
          </div>

          <div class="terminal-tabs">
            <button class="terminal-tab active" data-tab="lint">${curT.terminal.tabs.lint}</button>
            <button class="terminal-tab" data-tab="catalog">${curT.terminal.tabs.catalog}</button>
            <button class="terminal-tab" data-tab="topology">${curT.terminal.tabs.topology}</button>
            <button class="terminal-tab" data-tab="mcp">${curT.terminal.tabs.mcp}</button>
            <button class="terminal-tab" data-tab="pubsub">${curT.terminal.tabs.pubsub}</button>
          </div>

          <button class="terminal-copy-btn" id="terminal-copy" title="Copy output">
            <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
              <rect x="9" y="9" width="13" height="13" rx="2" ry="2"></rect>
              <path d="M5 15H4a2 2 0 0 1-2-2V4a2 2 0 0 1 2-2h9a2 2 0 0 1 2 2v1"></path>
            </svg>
          </button>
        </div>

        <div class="terminal-body">
          <pre><code id="terminal-code">${escapeHtml(curT.terminal.lintOutput)}</code></pre>
        </div>
      </div>
    </div>
  `;

  // Attach button navigation
  hero.querySelector("#hero-btn-quickstart")?.addEventListener("click", () => {
    props.onNavigate("docs", "quickstart");
  });

  hero.querySelector("#hero-btn-simulator")?.addEventListener("click", () => {
    props.onNavigate("home", "simulator");
  });

  hero.querySelector("#hero-btn-docs")?.addEventListener("click", () => {
    props.onNavigate("docs", "intro");
  });

  // Terminal tab switching logic
  const tabOutputs: Record<string, string> = {
    lint: curT.terminal.lintOutput,
    catalog: curT.terminal.catalogOutput,
    topology: curT.terminal.topologyOutput,
    mcp: curT.terminal.mcpOutput,
    pubsub: curT.terminal.pubsubOutput,
  };

  const codeEl = hero.querySelector<HTMLElement>("#terminal-code");
  const tabBtns = hero.querySelectorAll<HTMLButtonElement>(".terminal-tab");
  let activeTab = "lint";

  tabBtns.forEach((btn) => {
    btn.addEventListener("click", () => {
      tabBtns.forEach((b) => b.classList.remove("active"));
      btn.classList.add("active");
      const tabKey = btn.dataset.tab || "lint";
      activeTab = tabKey;
      if (codeEl && tabOutputs[tabKey]) {
        codeEl.innerHTML = escapeHtml(tabOutputs[tabKey]);
      }
    });
  });

  const copyBtn = hero.querySelector<HTMLButtonElement>("#terminal-copy");
  if (copyBtn) {
    copyBtn.addEventListener("click", () => {
      const textToCopy = tabOutputs[activeTab] || "";
      navigator.clipboard.writeText(textToCopy).then(() => {
        copyBtn.classList.add("copied");
        setTimeout(() => copyBtn.classList.remove("copied"), 1500);
      });
    });
  }

  return hero;
}

function escapeHtml(str: string): string {
  return str
    .replace(/&/g, "&amp;")
    .replace(/</g, "&lt;")
    .replace(/>/g, "&gt;");
}
