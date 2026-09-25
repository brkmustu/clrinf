import { t } from "../i18n";

export interface FooterProps {
  onNavigate: (view: "home" | "docs", hash?: string) => void;
}

export function renderFooter(props: FooterProps): HTMLElement {
  const footer = document.createElement("footer");
  footer.className = "site-footer";

  const curT = t();

  footer.innerHTML = `
    <div class="footer-inner">
      <div class="footer-top">
        <div class="footer-brand">
          <div class="brand">
            <div class="brand-logo">
              <svg width="20" height="20" viewBox="0 0 32 32" fill="none">
                <path d="M16 2L2 9L16 16L30 9L16 2Z" fill="#3b82f6" fill-opacity="0.9"/>
                <path d="M2 23L16 30L30 23V16L16 23L2 16V23Z" fill="#10b981" fill-opacity="0.9"/>
                <path d="M2 9V16L16 23L30 16V9L16 16L2 9Z" fill="#8b5cf6" fill-opacity="0.9"/>
              </svg>
            </div>
            <div class="brand-name">
              <span>clrinf</span>
              <span class="version-badge">${curT.nav.version}</span>
            </div>
          </div>
          <p class="footer-desc">${curT.footer.desc}</p>
        </div>

        <!-- Ecosystem -->
        <div>
          <h4 class="footer-col-title">${curT.footer.sections.ecosystem}</h4>
          <ul class="footer-link-list">
            <li><a href="https://github.com/brkmustu/clrinf/tree/main/clrinf-contracts" target="_blank" rel="noopener">${curT.footer.links.contracts}</a></li>
            <li><a href="https://github.com/brkmustu/clrinf/tree/main/clrinfcs" target="_blank" rel="noopener">${curT.footer.links.csharp}</a></li>
            <li><a href="https://github.com/brkmustu/clrinf/tree/main/clrinfrs" target="_blank" rel="noopener">${curT.footer.links.rust}</a></li>
            <li><a href="https://github.com/brkmustu/clrinf/tree/main/clrinfjs" target="_blank" rel="noopener">${curT.footer.links.ts}</a></li>
          </ul>
        </div>

        <!-- Tooling & AI -->
        <div>
          <h4 class="footer-col-title">${curT.footer.sections.tooling}</h4>
          <ul class="footer-link-list">
            <li><a href="#docs:cliReference" data-doc="cliReference">${curT.footer.links.cli}</a></li>
            <li><a href="#docs:aiGovernance" data-doc="aiGovernance">${curT.footer.links.mcp}</a></li>
            <li><a href="#docs:rulesEngine" data-doc="rulesEngine">${curT.footer.links.scaffold}</a></li>
            <li><a href="#simulator" data-nav="simulator">${curT.footer.links.topology}</a></li>
          </ul>
        </div>

        <!-- Governance -->
        <div>
          <h4 class="footer-col-title">${curT.footer.sections.governance}</h4>
          <ul class="footer-link-list">
            <li><a href="#docs:intro" data-doc="intro">${curT.footer.links.constitution}</a></li>
            <li><a href="#docs:topology" data-doc="topology">${curT.footer.links.versioning}</a></li>
            <li><a href="https://github.com/brkmustu/clrinf/blob/main/LICENSE" target="_blank" rel="noopener">${curT.footer.links.license}</a></li>
          </ul>
        </div>
      </div>

      <div class="footer-bottom">
        <p class="footer-copy">© 2026 clrinf Project. ${curT.footer.rights}</p>
        <div class="footer-badges">
          <span class="badge-lang">Rust</span>
          <span class="badge-lang">.NET 10</span>
          <span class="badge-lang">TypeScript</span>
          <span class="badge-lang">CloudEvents 1.0</span>
          <span class="badge-lang">JSON Schema</span>
        </div>
      </div>
    </div>
  `;

  footer.querySelectorAll<HTMLAnchorElement>("[data-doc]").forEach((link) => {
    link.addEventListener("click", (e) => {
      e.preventDefault();
      const docId = link.dataset.doc;
      props.onNavigate("docs", docId);
    });
  });

  footer.querySelectorAll<HTMLAnchorElement>("[data-nav]").forEach((link) => {
    link.addEventListener("click", (e) => {
      e.preventDefault();
      const target = link.dataset.nav;
      props.onNavigate("home", target);
    });
  });

  return footer;
}
