import "./styles/tokens.css";
import "./styles/layout.css";
import "./styles/components.css";
import "./styles/docs.css";

import { onLangChange, getLang } from "./i18n";
import { renderNavbar } from "./components/Navbar";
import { renderHero } from "./components/Hero";
import { renderProblemCards } from "./components/ProblemCards";
import { renderTopologySimulator } from "./components/TopologySimulator";
import { renderModuleStudio } from "./components/ModuleStudio";
import { renderCodeMatrix } from "./components/CodeMatrix";
import { renderDocsViewer } from "./components/DocsViewer";
import { renderFooter } from "./components/Footer";

type ViewState = "home" | "docs";

class App {
  private root: HTMLElement;
  private currentView: ViewState = "home";
  private activeDocChapter: string = "intro";

  constructor() {
    const el = document.getElementById("app");
    if (!el) throw new Error("#app element not found");
    this.root = el;

    this.parseHash();
    this.initListeners();
    this.render();
  }

  private parseHash() {
    const hash = window.location.hash.replace(/^#/, "");
    if (hash.startsWith("docs")) {
      this.currentView = "docs";
      const parts = hash.split(":");
      if (parts[1]) {
        this.activeDocChapter = parts[1];
      }
    } else {
      this.currentView = "home";
    }
  }

  private initListeners() {
    window.addEventListener("hashchange", () => {
      this.parseHash();
      this.render();
      this.handleScrollAfterRender();
    });

    onLangChange(() => {
      this.render();
    });
  }

  private handleScrollAfterRender() {
    const hash = window.location.hash.replace(/^#/, "");
    if (this.currentView === "home" && hash && !hash.startsWith("docs")) {
      const target = document.getElementById(hash);
      if (target) {
        target.scrollIntoView({ behavior: "smooth" });
      }
    }
  }

  public navigate = (view: ViewState, target?: string) => {
    this.currentView = view;
    if (view === "docs") {
      this.activeDocChapter = target || "intro";
      window.location.hash = `docs:${this.activeDocChapter}`;
    } else {
      if (target && target !== "home") {
        window.location.hash = target;
      } else {
        window.location.hash = "home";
        window.scrollTo({ top: 0, behavior: "smooth" });
      }
    }
    this.render();
  };

  public render() {
    this.root.innerHTML = "";

    // Set page document title based on current language & view
    const lang = getLang();
    const titleMap: Record<string, string> = {
      tr: "clrinf — Yapay Zeka Ajanları İçin Mimari Güvence ve Çok Dilli Omurga",
      en: "clrinf — Architectural Guardrails for AI Coding Agents & Polyglot Backbone",
      es: "clrinf — Gobernanza Arquitectónica para Agentes IA y Núcleo Políglota",
    };
    document.title = titleMap[lang] || titleMap.tr;

    // 1. Navbar
    const navbar = renderNavbar({
      currentView: this.currentView,
      onNavigate: this.navigate,
    });
    this.root.appendChild(navbar);

    // 2. Main Body Container
    const mainContainer = document.createElement("main");
    mainContainer.className = "app-content";

    if (this.currentView === "home") {
      // Home View Sections
      mainContainer.appendChild(renderHero({ onNavigate: this.navigate }));
      mainContainer.appendChild(renderProblemCards());
      mainContainer.appendChild(renderModuleStudio());
      mainContainer.appendChild(renderTopologySimulator());
      mainContainer.appendChild(renderCodeMatrix());
    } else {
      // Docs View
      mainContainer.appendChild(
        renderDocsViewer({
          initialArticleId: this.activeDocChapter,
          onNavigate: this.navigate,
        })
      );
    }

    this.root.appendChild(mainContainer);

    // 3. Footer
    const footer = renderFooter({
      onNavigate: this.navigate,
    });
    this.root.appendChild(footer);
  }
}

// Boot application
new App();
