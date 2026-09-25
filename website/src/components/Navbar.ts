import { t, getLang, setLang, Language } from "../i18n";

export interface NavbarProps {
  currentView: "home" | "docs";
  onNavigate: (view: "home" | "docs", hash?: string) => void;
}

export function renderNavbar(props: NavbarProps): HTMLElement {
  const nav = document.createElement("header");
  nav.className = "site-header";
  nav.id = "main-header";

  const currentT = t();
  const currentLang = getLang();

  nav.innerHTML = `
    <div class="header-inner">
      <a href="#home" class="brand" id="brand-link">
        <div class="brand-logo">
          <svg width="20" height="20" viewBox="0 0 32 32" fill="none">
            <path d="M16 2L2 9L16 16L30 9L16 2Z" fill="#3b82f6" fill-opacity="0.9"/>
            <path d="M2 23L16 30L30 23V16L16 23L2 16V23Z" fill="#10b981" fill-opacity="0.9"/>
            <path d="M2 9V16L16 23L30 16V9L16 16L2 9Z" fill="#8b5cf6" fill-opacity="0.9"/>
          </svg>
        </div>
        <div class="brand-name">
          <span>clrinf</span>
          <span class="version-badge">${currentT.nav.version}</span>
        </div>
      </a>

      <nav class="nav-links">
        <a href="#home" class="nav-link ${props.currentView === 'home' ? 'active' : ''}" data-nav="home">${currentT.nav.home}</a>
        <a href="#problems" class="nav-link" data-nav="problems">${currentT.nav.problems}</a>
        <a href="#module-studio" class="nav-link" data-nav="module-studio">${currentT.nav.modules}</a>
        <a href="#simulator" class="nav-link" data-nav="simulator">${currentT.nav.simulator}</a>
        <a href="#code-matrix" class="nav-link" data-nav="code-matrix">${currentT.nav.codeMatrix}</a>
        <a href="#docs" class="nav-link ${props.currentView === 'docs' ? 'active' : ''}" data-nav="docs">${currentT.nav.docs}</a>
      </nav>

      <div class="nav-actions">
        <div class="lang-segmented">
          <button class="lang-btn ${currentLang === 'tr' ? 'active' : ''}" data-lang="tr">TR</button>
          <button class="lang-btn ${currentLang === 'en' ? 'active' : ''}" data-lang="en">EN</button>
          <button class="lang-btn ${currentLang === 'es' ? 'active' : ''}" data-lang="es">ES</button>
        </div>

        <a href="https://github.com/brkmustu/clrinf" target="_blank" rel="noopener noreferrer" class="github-btn">
          <svg width="16" height="16" viewBox="0 0 24 24" fill="currentColor">
            <path fill-rule="evenodd" clip-rule="evenodd" d="M12 2C6.477 2 2 6.484 2 12.017c0 4.425 2.865 8.18 6.839 9.504.5.092.682-.217.682-.483 0-.237-.008-.868-.013-1.703-2.782.605-3.369-1.343-3.369-1.343-.454-1.158-1.11-1.466-1.11-1.466-.908-.62.069-.608.069-.608 1.003.07 1.53 1.032 1.53 1.032.892 1.53 2.341 1.088 2.91.832.092-.647.35-1.088.636-1.338-2.22-.253-4.555-1.113-4.555-4.951 0-1.093.39-1.988 1.029-2.688-.103-.253-.446-1.272.098-2.65 0 0 .84-.27 2.75 1.026A9.564 9.564 0 0112 6.844c.85.004 1.705.115 2.504.337 1.909-1.296 2.747-1.027 2.747-1.027.546 1.379.202 2.398.1 2.651.64.7 1.028 1.595 1.028 2.688 0 3.848-2.339 4.695-4.566 4.943.359.309.678.92.678 1.855 0 1.338-.012 2.419-.012 2.747 0 .268.18.58.688.482A10.019 10.019 0 0022 12.017C22 6.484 17.522 2 12 2z"/>
          </svg>
          <span>GitHub</span>
        </a>
      </div>
    </div>
  `;

  // Attach event handlers
  nav.querySelectorAll<HTMLButtonElement>(".lang-btn").forEach((btn) => {
    btn.addEventListener("click", () => {
      const targetLang = btn.dataset.lang as Language;
      if (targetLang) {
        setLang(targetLang);
      }
    });
  });

  nav.querySelectorAll<HTMLAnchorElement>("[data-nav]").forEach((link) => {
    link.addEventListener("click", (e) => {
      e.preventDefault();
      const target = link.dataset.nav;
      if (target === "docs") {
        props.onNavigate("docs");
      } else if (target === "home") {
        props.onNavigate("home");
      } else {
        props.onNavigate("home", target);
      }
    });
  });

  const brandLink = nav.querySelector<HTMLAnchorElement>("#brand-link");
  if (brandLink) {
    brandLink.addEventListener("click", (e) => {
      e.preventDefault();
      props.onNavigate("home");
    });
  }

  return nav;
}
