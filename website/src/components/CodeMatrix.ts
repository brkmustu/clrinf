import { t } from "../i18n";

type LangKey = "csharp" | "rust" | "ts";

export function renderCodeMatrix(): HTMLElement {
  const section = document.createElement("section");
  section.className = "section";
  section.id = "code-matrix";

  let activeLang: LangKey = "csharp";

  const codeSnippets: Record<
    LangKey,
    {
      title: string;
      desc: string;
      code: string;
      tags: string[];
    }
  > = {
    csharp: {
      title: "C# (.NET 10 / Roslyn Enforced)",
      desc: "IBusinessRule<T> implementasyonu. ARCH002 ile derleme anında doğrulanır; ARCH001 ile domain katmanına yabancı kütüphane sızması önlenir.",
      tags: ["Roslyn ARCH002", "Pure Domain", "Non-Throwing", "Clean Architecture"],
      code: `using ClrInf.Core.Domain;
using ClrInf.Core.Primitives;

namespace Ordering.Domain.Rules;

/// <summary>
/// Sipariş tutarının minimum limitin üzerinde olduğunu doğrular.
/// </summary>
public sealed class OrderMinimumAmountRule : IBusinessRule<Order>
{
    private readonly decimal _minAmount;

    public OrderMinimumAmountRule(decimal minAmount)
    {
        _minAmount = minAmount;
    }

    public Result Check(Order order)
    {
        if (order.TotalAmount < _minAmount)
        {
            return Result.Failure(
                OrderErrors.BelowMinimumAmount(order.TotalAmount, _minAmount)
            );
        }

        return Result.Success();
    }
}`,
    },
    rust: {
      title: "Rust (Zero-Panic / Syn AST Enforced)",
      desc: "clrinfrs::BusinessRule trait'i. RUST_ARCH001 ile unwrap, expect veya panic! kesinlikle yasaktır; tüm hatalar monadic DomainError ile döner.",
      tags: ["Syn RUST_ARCH001", "Zero-Panic", "Result<T, E>", "Memory Safe"],
      code: `use clrinfrs_core::domain::{BusinessRule, DomainError};
use crate::models::Order;

/// Sipariş tutarının minimum limitin üzerinde olduğunu doğrular.
pub struct OrderMinimumAmountRule {
    pub min_amount: f64,
}

impl BusinessRule<Order> for OrderMinimumAmountRule {
    fn check(&self, entity: &Order) -> Result<(), DomainError> {
        if entity.total_amount < self.min_amount {
            return Err(DomainError::RuleViolation(format!(
                "Order amount {:.2} is below the required minimum of {:.2}",
                entity.total_amount, self.min_amount
            )));
        }
        Ok(())
    }
}`,
    },
    ts: {
      title: "TypeScript (Functional & Monadic / TS AST Enforced)",
      desc: "Saf fonksiyonlar ve pipeRules kompozisyonu. ARCH_TS_001 ile throw new Error yasaktır; ARCH_TS_003 ile C# MediatR taklitleri engellenir.",
      tags: ["TS AST ARCH_TS_001", "ARCH_TS_003", "pipeRules()", "Monadic Result"],
      code: `import { RuleFn, Result, pipeRules } from "@clrinf/core";
import { Order, DomainError } from "../domain";

/**
 * Sipariş tutarının minimum limitin üzerinde olduğunu doğrular.
 * Saf fonksiyon olarak tasarlanmıştır.
 */
export const validateMinimumAmount = (minAmount: number): RuleFn<Order> => {
  return (order: Order): Result<Order, DomainError> => {
    if (order.totalAmount < minAmount) {
      return Result.err(
        new DomainError(\`Order amount \${order.totalAmount} is below minimum \${minAmount}\`)
      );
    }
    return Result.ok(order);
  };
};

// Fonksiyonel kompozisyon örneği:
export const validateOrderSubmission = pipeRules<Order>(
  validateMinimumAmount(100),
  validateCustomerStatus,
  validateInventoryAvailability
);`,
    },
  };

  function renderContent() {
    const curT = t();
    const data = codeSnippets[activeLang];

    section.innerHTML = `
      <div class="container">
        <div class="section-header">
          <span class="section-tag">${curT.codeMatrix.tag}</span>
          <h2 class="section-title">${curT.codeMatrix.title}</h2>
          <p class="section-subtitle">${curT.codeMatrix.subtitle}</p>
        </div>

        <div class="matrix-container">
          <!-- Language Tabs -->
          <div class="matrix-tabs">
            <button class="matrix-tab ${activeLang === 'csharp' ? 'active' : ''}" data-lang="csharp">
              <span class="lang-icon">🔷</span>
              <span>${curT.codeMatrix.tabs.csharp}</span>
            </button>
            <button class="matrix-tab ${activeLang === 'rust' ? 'active' : ''}" data-lang="rust">
              <span class="lang-icon">🦀</span>
              <span>${curT.codeMatrix.tabs.rust}</span>
            </button>
            <button class="matrix-tab ${activeLang === 'ts' ? 'active' : ''}" data-lang="ts">
              <span class="lang-icon">🟨</span>
              <span>${curT.codeMatrix.tabs.ts}</span>
            </button>
          </div>

          <!-- Code Card -->
          <div>
            <div class="matrix-card-header">
              <div>
                <h3 class="matrix-lang-title">${data.title}</h3>
                <p class="matrix-lang-desc">${data.desc}</p>
              </div>
              <div class="matrix-tags">
                ${data.tags.map((tag) => `<span class="badge-tag">${tag}</span>`).join("")}
              </div>
            </div>

            <div class="matrix-code-body">
              <pre><code class="language-${activeLang}">${escapeHtml(data.code)}</code></pre>
            </div>
          </div>
        </div>
      </div>
    `;

    // Attach listeners
    section.querySelectorAll<HTMLButtonElement>(".matrix-tab").forEach((btn) => {
      btn.addEventListener("click", () => {
        const lang = btn.dataset.lang as LangKey;
        if (lang) {
          activeLang = lang;
          renderContent();
        }
      });
    });
  }

  renderContent();
  return section;
}

function escapeHtml(str: string): string {
  return str
    .replace(/&/g, "&amp;")
    .replace(/</g, "&lt;")
    .replace(/>/g, "&gt;");
}
