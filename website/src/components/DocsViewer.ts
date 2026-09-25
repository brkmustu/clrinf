import { t, getLang } from "../i18n";
import { docsContent, DocChapter } from "../data/docsData";

export interface DocsViewerProps {
  initialArticleId?: string;
  onNavigate: (view: "home" | "docs", hash?: string) => void;
}

export function renderDocsViewer(props: DocsViewerProps): HTMLElement {
  const container = document.createElement("div");
  container.className = "docs-wrapper";
  container.id = "docs-viewer";

  let activeArticleId = props.initialArticleId || "intro";
  let searchQuery = "";

  function parseMarkdown(md: string): string {
    const lines = md.trim().split("\n");
    let html = "";
    let inCodeBlock = false;
    let codeBlockLang = "";
    let codeContent = "";
    let inTable = false;
    let inUl = false;
    let inOl = false;

    for (let i = 0; i < lines.length; i++) {
      let line = lines[i];

      // Code blocks
      if (line.startsWith("```")) {
        if (inCodeBlock) {
          html += `<div class="doc-code-block"><div class="doc-code-header"><span>${codeBlockLang || 'code'}</span></div><pre><code>${escapeHtml(codeContent.trim())}</code></pre></div>\n`;
          inCodeBlock = false;
          codeContent = "";
          codeBlockLang = "";
        } else {
          inCodeBlock = true;
          codeBlockLang = line.replace("```", "").trim();
        }
        continue;
      }

      if (inCodeBlock) {
        codeContent += line + "\n";
        continue;
      }

      // Indented text inside list item
      if ((inUl || inOl) && (line.startsWith("   ") || line.startsWith("  ") || line.startsWith("\t"))) {
        html += `<div class="doc-sub-text">${formatInline(line.trim())}</div>\n`;
        continue;
      }

      // Close open elements if line is empty
      if (!line.trim()) {
        if (inUl) {
          html += "</ul>\n";
          inUl = false;
        }
        if (inOl) {
          html += "</ol>\n";
          inOl = false;
        }
        if (inTable) {
          html += "</tbody></table></div>\n";
          inTable = false;
        }
        continue;
      }

      // Headers
      if (line.startsWith("# ")) {
        if (inUl) { html += "</ul>\n"; inUl = false; }
        if (inOl) { html += "</ol>\n"; inOl = false; }
        html += `<h1 class="doc-h1">${formatInline(line.substring(2))}</h1>\n`;
        continue;
      }
      if (line.startsWith("## ")) {
        if (inUl) { html += "</ul>\n"; inUl = false; }
        if (inOl) { html += "</ol>\n"; inOl = false; }
        html += `<h2 class="doc-h2">${formatInline(line.substring(3))}</h2>\n`;
        continue;
      }
      if (line.startsWith("### ")) {
        if (inUl) { html += "</ul>\n"; inUl = false; }
        if (inOl) { html += "</ol>\n"; inOl = false; }
        html += `<h3 class="doc-h3">${formatInline(line.substring(4))}</h3>\n`;
        continue;
      }

      // Tables
      if (line.startsWith("|") && line.endsWith("|")) {
        if (inUl) { html += "</ul>\n"; inUl = false; }
        if (inOl) { html += "</ol>\n"; inOl = false; }
        const cells = line
          .split("|")
          .slice(1, -1)
          .map((c) => c.trim());

        // Check if it's separator row
        if (cells.every((c) => /^:?-+:?$/.test(c))) {
          continue;
        }

        if (!inTable) {
          inTable = true;
          html += `<div class="doc-table-wrap"><table class="doc-table"><thead><tr>`;
          cells.forEach((c) => {
            html += `<th>${formatInline(c)}</th>`;
          });
          html += `</tr></thead><tbody>\n`;
        } else {
          html += `<tr>`;
          cells.forEach((c) => {
            html += `<td>${formatInline(c)}</td>`;
          });
          html += `</tr>\n`;
        }
        continue;
      }

      // Lists
      if (line.trim().startsWith("- ") || line.trim().startsWith("• ")) {
        if (inOl) { html += "</ol>\n"; inOl = false; }
        if (!inUl) {
          html += `<ul class="doc-list">\n`;
          inUl = true;
        }
        const itemText = line.trim().substring(2);
        html += `<li>${formatInline(itemText)}</li>\n`;
        continue;
      }

      // Ordered list
      const numMatch = line.trim().match(/^(\d+)\.\s+(.*)$/);
      if (numMatch) {
        if (inUl) { html += "</ul>\n"; inUl = false; }
        if (!inOl) {
          html += `<ol class="doc-ordered-list">\n`;
          inOl = true;
        }
        html += `<li value="${numMatch[1]}">${formatInline(numMatch[2])}</li>\n`;
        continue;
      }

      // Horizontal Rule
      if (line.trim() === "---" || line.trim() === "***") {
        if (inUl) { html += "</ul>\n"; inUl = false; }
        if (inOl) { html += "</ol>\n"; inOl = false; }
        html += `<hr class="doc-divider" />\n`;
        continue;
      }

      // Regular Paragraph
      if (inUl) { html += "</ul>\n"; inUl = false; }
      if (inOl) { html += "</ol>\n"; inOl = false; }
      html += `<p class="doc-p">${formatInline(line)}</p>\n`;
    }

    if (inUl) html += "</ul>\n";
    if (inOl) html += "</ol>\n";
    if (inTable) html += "</tbody></table></div>\n";

    return html;
  }

  function formatInline(text: string): string {
    return text
      .replace(/`([^`]+)`/g, '<code class="doc-inline-code">$1</code>')
      .replace(/\*\*([^*]+)\*\*/g, "<strong>$1</strong>")
      .replace(/\*([^*]+)\*/g, "<em>$1</em>");
  }

  function renderContent() {
    const curLang = getLang();
    const curT = t();
    const chapters = docsContent[curLang] || docsContent.tr;

    const filteredChapters = chapters.filter(
      (c) =>
        c.title.toLowerCase().includes(searchQuery.toLowerCase()) ||
        c.category.toLowerCase().includes(searchQuery.toLowerCase()) ||
        c.content.toLowerCase().includes(searchQuery.toLowerCase())
    );

    const activeArticle =
      chapters.find((c) => c.id === activeArticleId) || chapters[0];
    const currentIndex = chapters.findIndex((c) => c.id === activeArticle.id);
    const prevArticle = currentIndex > 0 ? chapters[currentIndex - 1] : null;
    const nextArticle =
      currentIndex < chapters.length - 1 ? chapters[currentIndex + 1] : null;

    // Group chapters by category
    const categories: Record<string, DocChapter[]> = {};
    filteredChapters.forEach((ch) => {
      if (!categories[ch.category]) categories[ch.category] = [];
      categories[ch.category].push(ch);
    });

    const sidebarCategoriesHtml = Object.entries(categories)
      .map(
        ([cat, items]) => `
      <div class="sidebar-category">
        <div class="cat-title">${cat}</div>
        <div class="cat-items">
          ${items
            .map(
              (item) => `
            <button class="doc-sidebar-link ${item.id === activeArticle.id ? 'active' : ''}" data-id="${item.id}">
              <span>${item.title}</span>
            </button>
          `
            )
            .join("")}
        </div>
      </div>
    `
      )
      .join("");

    container.innerHTML = `
      <div class="docs-container">
        <!-- Sidebar Navigation -->
        <aside class="docs-sidebar">
          <div class="sidebar-search">
            <svg class="search-icon" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
              <circle cx="11" cy="11" r="8"></circle>
              <line x1="21" y1="21" x2="16.65" y2="16.65"></line>
            </svg>
            <input 
              type="text" 
              class="search-input" 
              id="docs-search-input" 
              placeholder="${curT.docs.searchPlaceholder}" 
              value="${searchQuery}" 
            />
          </div>

          <nav class="sidebar-nav">
            ${sidebarCategoriesHtml || '<div class="no-results">Sonuç bulunamadı / No results</div>'}
          </nav>
        </aside>

        <!-- Main Content Area -->
        <main class="docs-main">
          <div class="doc-breadcrumb">
            <a href="#home" id="doc-back-home">${curT.nav.home}</a>
            <span class="crumb-sep">/</span>
            <span>${curT.nav.docs}</span>
            <span class="crumb-sep">/</span>
            <span class="crumb-current">${activeArticle.category}</span>
          </div>

          <article class="doc-article">
            ${parseMarkdown(activeArticle.content)}
          </article>

          <!-- Prev / Next Navigation Footer -->
          <div class="doc-article-nav">
            ${
              prevArticle
                ? `
              <button class="doc-nav-btn prev" data-goto="${prevArticle.id}">
                <span class="nav-direction">← Önceki / Previous</span>
                <span class="nav-title">${prevArticle.title}</span>
              </button>
            `
                : "<div></div>"
            }

            ${
              nextArticle
                ? `
              <button class="doc-nav-btn next" data-goto="${nextArticle.id}">
                <span class="nav-direction">Sonraki / Next →</span>
                <span class="nav-title">${nextArticle.title}</span>
              </button>
            `
                : "<div></div>"
            }
          </div>
        </main>
      </div>
    `;

    // Event handlers
    const searchInput = container.querySelector<HTMLInputElement>("#docs-search-input");
    if (searchInput) {
      searchInput.addEventListener("input", (e) => {
        searchQuery = (e.target as HTMLInputElement).value;
        renderContent();
        const newSearchInput = container.querySelector<HTMLInputElement>("#docs-search-input");
        if (newSearchInput) {
          newSearchInput.focus();
          newSearchInput.selectionStart = newSearchInput.selectionEnd = searchQuery.length;
        }
      });
    }

    container.querySelectorAll<HTMLButtonElement>(".doc-sidebar-link").forEach((btn) => {
      btn.addEventListener("click", () => {
        const id = btn.dataset.id;
        if (id) {
          activeArticleId = id;
          renderContent();
          window.scrollTo({ top: 0, behavior: "smooth" });
        }
      });
    });

    container.querySelectorAll<HTMLButtonElement>(".doc-nav-btn").forEach((btn) => {
      btn.addEventListener("click", () => {
        const id = btn.dataset.goto;
        if (id) {
          activeArticleId = id;
          renderContent();
          window.scrollTo({ top: 0, behavior: "smooth" });
        }
      });
    });

    container.querySelector("#doc-back-home")?.addEventListener("click", (e) => {
      e.preventDefault();
      props.onNavigate("home");
    });
  }

  renderContent();
  return container;
}

function escapeHtml(str: string): string {
  return str
    .replace(/&/g, "&amp;")
    .replace(/</g, "&lt;")
    .replace(/>/g, "&gt;");
}
