import { t } from "../i18n";

type Scenario = "valid" | "deadEvent" | "orphanSub" | "missingUpcaster";

export function renderTopologySimulator(): HTMLElement {
  const section = document.createElement("section");
  section.className = "section";
  section.id = "simulator";

  let activeScenario: Scenario = "valid";

  function renderContent() {
    const curT = t();

    const scenarioConfigs: Record<
      Scenario,
      {
        pubNode: { name: string; event: string; status: "ok" | "warn" | "error" };
        subNode: { name: string; event: string; status: "ok" | "warn" | "error" };
        linkStatus: "active" | "broken" | "warning";
        flowLabel: string;
      }
    > = {
      valid: {
        pubNode: { name: "order-service", event: "ordering.order.placed.v1", status: "ok" },
        subNode: { name: "billing-service", event: "ordering.order.placed.v1", status: "ok" },
        linkStatus: "active",
        flowLabel: "CloudEvent 1.0 (Delivered & Verified)",
      },
      deadEvent: {
        pubNode: { name: "payment-service", event: "payment.completed.v1", status: "warn" },
        subNode: { name: "(No Subscriber)", event: "None", status: "warn" },
        linkStatus: "warning",
        flowLabel: "⚠️ Dead Event: Lost in Void (No Consumers)",
      },
      orphanSub: {
        pubNode: { name: "(No Publisher)", event: "None", status: "error" },
        subNode: { name: "notification-service", event: "invoice.issued.v1", status: "error" },
        linkStatus: "broken",
        flowLabel: "❌ Orphan Sub: Waiting on Ghost Event",
      },
      missingUpcaster: {
        pubNode: { name: "order-service", event: "ordering.order.placed.v1", status: "warn" },
        subNode: { name: "shipping-service", event: "ordering.order.placed.v2", status: "error" },
        linkStatus: "broken",
        flowLabel: "❌ Schema Drift: Missing Upcaster (v1 ➔ v2)",
      },
    };

    const cfg = scenarioConfigs[activeScenario];
    const statusMsg = curT.simulator.statusMessages[activeScenario];

    section.innerHTML = `
      <div class="container">
        <div class="section-header">
          <span class="section-tag">${curT.simulator.tag}</span>
          <h2 class="section-title">${curT.simulator.title}</h2>
          <p class="section-subtitle">${curT.simulator.subtitle}</p>
        </div>

        <div class="simulator-container">
          <!-- Scenario Buttons -->
          <div class="sim-scenario-bar">
            <button class="sim-btn ${activeScenario === 'valid' ? 'active' : ''}" data-sc="valid">
              ✓ ${curT.simulator.scenarios.valid}
            </button>
            <button class="sim-btn ${activeScenario === 'deadEvent' ? 'active' : ''}" data-sc="deadEvent">
              ⚠️ ${curT.simulator.scenarios.deadEvent}
            </button>
            <button class="sim-btn ${activeScenario === 'orphanSub' ? 'active' : ''}" data-sc="orphanSub">
              ✖ ${curT.simulator.scenarios.orphanSub}
            </button>
            <button class="sim-btn ${activeScenario === 'missingUpcaster' ? 'active' : ''}" data-sc="missingUpcaster">
              ⚡ ${curT.simulator.scenarios.missingUpcaster}
            </button>
          </div>

          <!-- Visual Topology Canvas -->
          <div class="topology-canvas">
            <div class="canvas-grid">
              <!-- Publisher Node -->
              <div class="topo-node node-${cfg.pubNode.status}">
                <div class="node-header">
                  <span class="node-role">PUBLISHER</span>
                  <span class="node-badge badge-${cfg.pubNode.status}">${cfg.pubNode.status.toUpperCase()}</span>
                </div>
                <div class="node-service-name">${cfg.pubNode.name}</div>
                <div class="node-event-tag">
                  <span class="icon">➔</span>
                  <code>${cfg.pubNode.event}</code>
                </div>
              </div>

              <!-- Connection Flow Stream -->
              <div class="topo-stream stream-${cfg.linkStatus}">
                <div class="stream-line"></div>
                <div class="stream-label">${cfg.flowLabel}</div>
              </div>

              <!-- Subscriber Node -->
              <div class="topo-node node-${cfg.subNode.status}">
                <div class="node-header">
                  <span class="node-role">SUBSCRIBER</span>
                  <span class="node-badge badge-${cfg.subNode.status}">${cfg.subNode.status.toUpperCase()}</span>
                </div>
                <div class="node-service-name">${cfg.subNode.name}</div>
                <div class="node-event-tag">
                  <span class="icon">📥</span>
                  <code>${cfg.subNode.event}</code>
                </div>
              </div>
            </div>
          </div>

          <!-- Status / Diagnostic Panel -->
          <div class="sim-diagnostic status-${activeScenario === 'valid' ? 'ok' : activeScenario === 'deadEvent' ? 'warn' : 'err'}">
            <div class="diag-header">
              <span class="diag-indicator"></span>
              <strong style="color: #ffffff; font-size: 0.95rem;">${curT.simulator.statusTitle}</strong>
              <code class="diag-cmd">$ clrinf-codegen topology check</code>
            </div>
            <div class="diag-body">
              <p>${statusMsg}</p>
            </div>
          </div>
        </div>
      </div>
    `;

    // Attach listeners
    section.querySelectorAll<HTMLButtonElement>(".sim-btn").forEach((btn) => {
      btn.addEventListener("click", () => {
        const sc = btn.dataset.sc as Scenario;
        if (sc) {
          activeScenario = sc;
          renderContent();
        }
      });
    });
  }

  renderContent();
  return section;
}
