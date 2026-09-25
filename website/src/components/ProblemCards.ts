import { t } from "../i18n";

export function renderProblemCards(): HTMLElement {
  const section = document.createElement("section");
  section.className = "section";
  section.id = "problems";

  const curT = t();

  const itemsHtml = curT.problems.items
    .map(
      (item) => `
    <div class="problem-card">
      <div>
        <div class="card-icon-wrap">
          <span>${item.icon}</span>
        </div>
        <h3 class="problem-title" style="margin-top: 14px;">${item.title}</h3>
      </div>

      <div class="problem-box">
        <div class="problem-box-header">
          <svg width="13" height="13" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5">
            <circle cx="12" cy="12" r="10"></circle>
            <line x1="12" y1="8" x2="12" y2="12"></line>
            <line x1="12" y1="16" x2="12.01" y2="16"></line>
          </svg>
          <span>Problem</span>
        </div>
        <p>${item.desc}</p>
      </div>

      <div class="solution-box">
        <div class="solution-box-header">
          <svg width="13" height="13" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5">
            <polyline points="20 6 9 17 4 12"></polyline>
          </svg>
          <span>clrinf Çözümü</span>
        </div>
        <p>${item.solution}</p>
      </div>
    </div>
  `
    )
    .join("");

  section.innerHTML = `
    <div class="container">
      <div class="section-header">
        <span class="section-tag">${curT.problems.tag}</span>
        <h2 class="section-title">${curT.problems.title}</h2>
        <p class="section-subtitle">${curT.problems.subtitle}</p>
      </div>

      <div class="problem-grid">
        ${itemsHtml}
      </div>
    </div>
  `;

  return section;
}
