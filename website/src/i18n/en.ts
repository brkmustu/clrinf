export const en = {
  nav: {
    home: "Home",
    philosophy: "Philosophy",
    problems: "Core Problems",
    modules: "Module Catalog",
    simulator: "Topology Simulator",
    codeMatrix: "Polyglot Code",
    docs: "Documentation",
    github: "GitHub",
    version: "v0.1.0",
  },
  hero: {
    badge: "ARCHITECTURAL INTEGRITY IN THE AI ERA",
    titleStart: "Architectural Guardrails",
    titleHighlight: "for AI Coding Agents",
    titleEnd: "and Polyglot Engineering",
    subtitle: "Enforce strict design patterns on AI coding agents (Claude, Cursor, Antigravity) from day zero. A contract-first, AST-verified application backbone for sustainable software quality.",
    ctaQuickstart: "Quickstart",
    ctaSimulator: "Topology Simulator",
    ctaDocs: "Explore Documentation",
  },
  terminal: {
    tabs: {
      lint: "Architectural Linter",
      topology: "Topology Check",
      catalog: "Built-In Catalog (clrinf catalog)",
      mcp: "Meta MCP Server",
      pubsub: "Mechanical Pub/Sub",
    },
    lintOutput: `$ clrinf-codegen lint --lang all
🔍 [csharp] Executing Roslyn AST analyzers...
   ✓ ARCH001: Domain layer has zero foreign framework dependencies.
   ✓ ARCH002: All rule classes implement IBusinessRule<T>.
   ✓ ARCH003: Prevented DbContext leakage into API Controllers.
   ✓ ARCH004: CQRS requests conform to IRequest<T>.
🔍 [rust] Executing Syn AST static analysis...
   ✓ RUST_ARCH001: Zero-panic guarantee (no unwrap/expect/panic detected).
🔍 [typescript] Executing TS Compiler API analysis...
   ✓ ARCH_TS_001: Raw throw prevented in rules; monadic Result active.
   ✓ ARCH_TS_003: Prohibited MediatR-style class emulation; functional pipes active.
✅ ARCHITECTURAL RULES SUCCESSFULLY VERIFIED ACROSS ALL LANGUAGES.`,
    topologyOutput: `$ clrinf-codegen topology check --schema-dir clrinf-contracts/schemas
🌐 Cross-Service Pub/Sub Topology Map:
  📦 Service: order-service
     Publishes:
       ➔ ordering.order.placed.v1
  📦 Service: billing-service
     Subscribes:
       📥 ordering.order.placed.v1
──────────────────────────────────────────
Diagnostics: 0 fatal issue(s) detected.
✅ Topology verified: No dead events or orphan subscribers detected.`,
    catalogOutput: `$ clrinf-codegen catalog
📦 clrinf Built-In Module Catalog

🏢 Domain Features & Suites:
  • crm          — CRM Full Suite                   [raw ✅ | wired ✅]
    Description: Full B2B CRM suite (Deals + Contacts + Activities + unified Cedar security).
  • deals        — CRM Deals / Sales Pipeline       [raw ✅ | wired ✅]
  • contacts     — CRM Contacts / Customers         [raw ✅ | wired ✅]
  • activities   — CRM Activities / Interactions    [raw ✅ | wired ✅]

⚙️  Cross-Cutting Concern Modules:
  • authz        — Multi-Tenant Cedar Authorization [raw ✅ | wired ✅]
  • caching      — Caching / In-Memory & Redis      [raw ✅ | wired ✅]
  • logging      — Structured Logging               [raw ✅ | wired ✅]
  • transaction  — Transaction & Unit of Work       [raw ✅ | wired ✅]
  • idempotency  — Idempotency Store                [raw ✅ | wired ✅]
  • outbox       — Transactional Outbox             [raw ✅ | wired ✅]
💡 Tip: Run 'clrinf module adopt <MODULE>' to adopt into any project in raw or wired mode.`,
    mcpOutput: `$ clrinf-codegen mcp
[clrinf-meta-mcp] Model Context Protocol JSON-RPC 2.0 stdio server ready...
Registered Tools:
  • clrinf_list_catalog (list official built-in modules & suites)
  • clrinf_adopt_module (adopt into project in raw or wired mode)
  • clrinf_module_manage (caching, logging, authz, outbox)
  • clrinf_add_domain_module (domain module with Cedar policy)
  • clrinf_add_entity (CRUD entity & repo port)
  • clrinf_inspect_ecosystem & lint_architecture
🤖 Claude / Cursor / Antigravity agents can autonomously govern and adopt modules.`,
    pubsubOutput: `$ clrinf-codegen generate-pubsub --lang all --output ./generated
📡 Generating CloudEvent Publisher & Idempotent Subscriber shells...
  [Rust Publisher]  generated/rust/order_placed_publisher.rs (OutboxStore atomic)
  [Rust Subscriber] generated/rust/order_placed_subscriber.rs (Idempotency protected)
  [C# Publisher]    generated/csharp/OrderPlacedPublisher.cs (IOutboxStore atomic)
  [C# Subscriber]   generated/csharp/OrderPlacedSubscriber.cs (IIdempotencyStore)
  [TS Publisher]    generated/typescript/order-placed-publisher.ts
  [TS Subscriber]   generated/typescript/order-placed-subscriber.ts
🎉 100% mechanical envelopes completed. Business reaction stubs ready for review.`,
  },
  problems: {
    tag: "CORE PROBLEMS & SOLUTIONS",
    title: "How Do We Prevent Architectural Decay in AI-Assisted Codebases?",
    subtitle: "AI agents code fast; but without strict architectural guardrails, your codebase rapidly spirals into unmaintainable spaghetti debt. clrinf solves this with 8 mechanical mechanisms:",
    items: [
      {
        icon: "🤖",
        title: "Unconstrained AI Generation & Spaghetti Drift",
        desc: "Without architectural constraints, AI models produce arbitrary abstractions, tangling domain logic with persistence frameworks.",
        solution: "Controlled code generation via architectural templates, Meta MCP tools, and compile-time linters keep AI agents aligned with verified patterns rather than inventing arbitrary structures.",
      },
      {
        icon: "📦",
        title: "Monolithic Lock-in & Heavy Package Dependencies",
        desc: "Trying to add a CRM domain feature or Cedar authorization to an existing project pulls massive library trees and rigid container assumptions.",
        solution: "clrinf module adopt engine: Transplant any module with zero external package dependencies (raw), or auto-wired into the project's native DI and module tree (wired).",
      },
      {
        icon: "🛡️",
        title: "Cross-Tenant Data Leakage & Security Oversights",
        desc: "When tenant checks are manually scattered across application code, developers or AI agents inevitably forget a tenant filter in one query, creating severe data breach risks.",
        solution: "Amazon Cedar multi-tenant by default policies (policies/*.cedar): context.tenant_id != resource.tenant_id triggers an absolute, overriding forbid guard.",
      },
      {
        icon: "📐",
        title: "Polyglot Contract Drift",
        desc: "A C# dev updates a schema, while Rust and TypeScript teams hand-craft DTOs and miss a field, causing silent serialization bugs.",
        solution: "CloudEvents 1.0 and JSON Schema Draft 2020-12 canonical schemas generate 100% error-free models across all languages in one command.",
      },
      {
        icon: "📡",
        title: "Silent Event & Data Loss in Microservices",
        desc: "A service publishes an event that nobody consumes (Dead Event), or subscribes to a nonexistent event. The bug is only noticed in production.",
        solution: "The clrinf-codegen topology check engine detects dead events and orphan subscribers before code ever reaches CI.",
      },
      {
        icon: "⚡",
        title: "Foreign Idiom Smuggling Across Stacks",
        desc: "Porting C# MediatR classes 1:1 into Rust or TypeScript pollutes their idiomatic elegance with reflection workarounds.",
        solution: "Each language stays idiomatic: C# native FrozenDictionary dispatcher or MediatR, Rust zero-panic BusinessRule, functional pipeRules in TypeScript.",
      },
      {
        icon: "🔌",
        title: "Message Broker & Cloud Vendor Lock-in",
        desc: "Many frameworks mandate Kafka, NATS, or AWS SQS up front, making local testing and modular monolith deployment painful.",
        solution: "Transport-agnostic OutboxStore and IdempotencyStore ports allow running as an in-memory modular monolith or scaling to distributed queues.",
      },
      {
        icon: "🛡️",
        title: "Layer Violations & Hidden Couplings",
        desc: "Developers or AI frequently bind databases to the domain layer, inject ORMs into controllers, or throw raw panics in handlers.",
        solution: "Roslyn (ARCH001-004), Syn (RUST_ARCH001), and TS AST (ARCH_TS_001) enforce boundaries at compile/analysis time.",
      },
    ],
  },
  moduleStudio: {
    tag: "ZERO-FRICTION MODULE TRANSPLANTATION",
    title: "Built-In Module Catalog & Adoption Studio",
    subtitle: "Transplant official built-in CRM suites and infrastructure concerns into ANY project (.csproj, Cargo.toml, package.json) with zero friction: in either pure zero-dependency raw mode or full ecosystem wired mode.",
    catalogBadge: "OFFICIAL BUILT-IN CATALOG",
    selectModule: "Select Module to Adopt:",
    selectMode: "Adoption Mode:",
    modes: {
      raw: {
        title: "Raw (Zero-Dependency & Pure Code)",
        desc: "Zero external package or library imports. Pure self-contained contracts (OperationClaim), isolated domain logic, repository ports, and Cedar policy.",
      },
      wired: {
        title: "Wired (Batteries-Included & Integrated)",
        desc: "Integrates with full clrinf capabilities. Auto-wires into DI (IServiceCollection), module tree (pub mod), barrel exports (export * as), and Cedar guardrails.",
      },
    },
    selectTarget: "Target Project Type:",
    targets: {
      csharp: "C# (.csproj)",
      rust: "Rust (Cargo.toml)",
      ts: "TypeScript (package.json)",
    },
    cliCommandLabel: "Generated Adoption Command",
    generatedFilesLabel: "Generated / Hooked Files",
    codePreviewTabs: {
      code: "Module Code",
      cedar: "Cedar Security Policy",
      wiring: "Wiring / DI",
    },
    copyBtn: "Copy Command",
    copiedBtn: "Copied!",
  },
  simulator: {
    tag: "LIVE INTERACTIVE EXPERIENCE",
    title: "Cross-Service Topology Simulator",
    subtitle: "Test how clrinf's distributed event engine catches dead events, orphan subscribers, and version gap upcaster chains in real time.",
    scenarios: {
      valid: "Normal Flow (Verified)",
      deadEvent: "Dead Event (No Subscribers)",
      orphanSub: "Orphan Subscriber (No Publisher)",
      missingUpcaster: "Missing Upcaster (v1 ➔ v2)",
    },
    statusTitle: "Topology Diagnostic Output",
    statusMessages: {
      valid: "✅ TOPOLOGY VERIFIED: 'order-service' publishes 'ordering.order.placed.v1', which is consumed by 'billing-service'. Zero risk of distributed event loss.",
      deadEvent: "⚠️ [TOPOLOGY_DEAD_EVENT] WARNING: 'payment-service' publishes 'payment.completed.v1' but no active subscriber exists in the topology! Event is lost in the ether.",
      orphanSub: "❌ [TOPOLOGY_ORPHAN_SUBSCRIBER] FATAL ERROR: 'notification-service' subscribes to 'invoice.issued.v1', but no service in the ecosystem publishes it! This handler will never fire.",
      missingUpcaster: "❌ [TOPOLOGY_UPCASTER_MISSING] FATAL ERROR: 'order-service' publishes 'v1', while 'shipping-service' expects 'v2'. No 'v1 -> v2' upcaster migration definition exists! Data loss risk.",
    },
  },
  codeMatrix: {
    tag: "IDIOMATIC MULTI-LANGUAGE DESIGN",
    title: "One Business Rule, Native in Three Languages",
    subtitle: "Inspect how the exact same validation rule (Minimum order amount) is implemented idiomatically across C#, Rust, and TypeScript.",
    tabs: {
      csharp: "C# (.NET 10)",
      rust: "Rust (Zero-Panic)",
      ts: "TypeScript (Functional)",
    },
  },
  docs: {
    searchPlaceholder: "Search docs (e.g., ARCH001, pipeRules, Outbox)...",
    chapters: {
      intro: "Introduction & Philosophy",
      quickstart: "Installation & Quickstart",
      moduleAdopt: "Module Catalog & Adoption",
      cedarAuthz: "Amazon Cedar Authorization",
      profilesDispatchers: "Profiles & Dispatchers",
      aiGovernance: "AI Agent Governance (Meta MCP)",
      linters: "Architectural Linters",
      rulesEngine: "Business Rules Engine",
      topology: "Topology & Pub/Sub",
      monolithDist: "Monolith vs Distributed",
      cliReference: "CLI Command Reference",
    },
  },
  footer: {
    desc: "Enforcing architectural guardrails on AI agents with a contract-first, polyglot enterprise application backbone.",
    sections: {
      ecosystem: "Ecosystem",
      tooling: "Tooling & AI",
      governance: "Governance",
    },
    links: {
      contracts: "clrinf-contracts",
      csharp: "clrinfcs (C#)",
      rust: "clrinfrs (Rust)",
      ts: "clrinfjs (TS)",
      cli: "clrinf-codegen CLI",
      mcp: "Meta MCP Server",
      scaffold: "Architectural Scaffolding",
      topology: "Topology Engine",
      constitution: "Constitution",
      versioning: "Versioning Policy",
      license: "BSL 1.1 License",
    },
    rights: "All rights reserved. Licensed under BSL 1.1. Change Date: 2030-09-08 (Apache-2.0).",
  },
};
