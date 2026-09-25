export const es = {
  nav: {
    home: "Inicio",
    philosophy: "Filosofía",
    problems: "Problemas Clave",
    modules: "Catálogo de Módulos",
    simulator: "Simulador de Topología",
    codeMatrix: "Código Políglota",
    docs: "Documentación",
    github: "GitHub",
    version: "v0.1.0",
  },
  hero: {
    badge: "INTEGRIDAD ARQUITECTÓNICA EN LA ERA DE LA IA",
    titleStart: "Marco Arquitectónico",
    titleHighlight: "para Agentes de IA",
    titleEnd: "y Desarrollo Políglota",
    subtitle: "Aplique patrones de diseño rigurosos a agentes de codificación de IA (Claude, Cursor, Antigravity) desde el primer día. Una arquitectura basada en contratos y verificada por compiladores.",
    ctaQuickstart: "Inicio Rápido",
    ctaSimulator: "Simulador de Topología",
    ctaDocs: "Explorar Documentación",
  },
  terminal: {
    tabs: {
      lint: "Linters Arquitectónicos",
      topology: "Verificación de Topología",
      catalog: "Catálogo Incorporado (clrinf catalog)",
      mcp: "Servidor Meta MCP",
      pubsub: "Pub/Sub Mecánico",
    },
    lintOutput: `$ clrinf-codegen lint --lang all
🔍 [csharp] Ejecutando analizadores AST de Roslyn...
   ✓ ARCH001: La capa de dominio no tiene dependencias externas.
   ✓ ARCH002: Todas las reglas implementan IBusinessRule<T>.
   ✓ ARCH003: Se evitó la fuga de DbContext en los controladores API.
   ✓ ARCH004: Solicitudes CQRS cumplen con IRequest<T>.
🔍 [rust] Ejecutando análisis estático Syn AST...
   ✓ RUST_ARCH001: Garantía cero-pánico (sin unwrap/expect/panic).
🔍 [typescript] Ejecutando análisis de TypeScript Compiler API...
   ✓ ARCH_TS_001: Lanzamiento de excepciones bloqueado; mónada Result activa.
   ✓ ARCH_TS_003: Prohibida emulación de clases tipo MediatR; tuberías funcionales activas.
✅ REGLAS ARQUITECTÓNICAS VERIFICADAS CON ÉXITO EN TODOS LOS LENGUAJES.`,
    topologyOutput: `$ clrinf-codegen topology check --schema-dir clrinf-contracts/schemas
🌐 Mapa de Topología Pub/Sub Entre Servicios:
  📦 Servicio: order-service
     Publica:
       ➔ ordering.order.placed.v1
  📦 Servicio: billing-service
     Se suscribe:
       📥 ordering.order.placed.v1
──────────────────────────────────────────
Diagnósticos: 0 problema(s) fatal(es) detectado(s).
✅ Topología verificada: Sin eventos muertos ni suscriptores huérfanos.`,
    catalogOutput: `$ clrinf-codegen catalog
📦 clrinf Catálogo de Módulos Incorporados (Built-In Module Catalog)

🏢 Módulos y Suites de Dominio (Domain Features & Suites):
  • crm          — CRM Full Suite                   [raw ✅ | wired ✅]
    Descripción: Suite B2B CRM completa (Deals + Contacts + Activities + protección unificada Cedar).
  • deals        — CRM Deals / Embudo de Ventas     [raw ✅ | wired ✅]
  • contacts     — CRM Contacts / Clientes          [raw ✅ | wired ✅]
  • activities   — CRM Activities / Tareas          [raw ✅ | wired ✅]

⚙️  Módulos de Infraestructura (Cross-Cutting Concerns):
  • authz        — Multi-Tenant Cedar Authorization [raw ✅ | wired ✅]
  • caching      — Caching / In-Memory & Redis      [raw ✅ | wired ✅]
  • logging      — Structured Logging               [raw ✅ | wired ✅]
  • transaction  — Transaction & Unit of Work       [raw ✅ | wired ✅]
  • idempotency  — Idempotency Store                [raw ✅ | wired ✅]
  • outbox       — Transactional Outbox             [raw ✅ | wired ✅]
💡 Consejo: Ejecute 'clrinf module adopt <MÓDULO>' para trasplantarlo en modo raw o wired.`,
    mcpOutput: `$ clrinf-codegen mcp
[clrinf-meta-mcp] Servidor stdio JSON-RPC 2.0 de Model Context Protocol listo...
Herramientas Registradas:
  • clrinf_list_catalog (listar módulos oficiales y suites de dominio)
  • clrinf_adopt_module (trasplantar a proyecto en modo raw o wired)
  • clrinf_module_manage (caching, logging, authz, outbox)
  • clrinf_add_domain_module (módulo de dominio con política Cedar)
  • clrinf_add_entity (entidad CRUD y puerto repositorio)
  • clrinf_inspect_ecosystem & lint_architecture
🤖 Agentes Claude / Cursor / Antigravity pueden gobernar y trasplantar módulos de forma autónoma.`,
    pubsubOutput: `$ clrinf-codegen generate-pubsub --lang all --output ./generated
📡 Generando Publishers de CloudEvent y cascarones de Suscriptores Idempotentes...
  [Rust Publisher]  generated/rust/order_placed_publisher.rs (Atómico con OutboxStore)
  [Rust Subscriber] generated/rust/order_placed_subscriber.rs (Protección por Idempotencia)
  [C# Publisher]    generated/csharp/OrderPlacedPublisher.cs (Atómico con IOutboxStore)
  [C# Subscriber]   generated/csharp/OrderPlacedSubscriber.cs (IIdempotencyStore)
  [TS Publisher]    generated/typescript/order-placed-publisher.ts
  [TS Subscriber]   generated/typescript/order-placed-subscriber.ts
🎉 Envoltorios 100% mecánicos completados. Listos para implementar lógica de reacción.`,
  },
  problems: {
    tag: "PROBLEMAS CLAVE Y SOLUCIONES",
    title: "¿Cómo Evitamos la Degradación Arquitectónica con la IA?",
    subtitle: "Los agentes de IA codifican rápido; pero sin restricciones arquitectónicas estrictas, el código degenera en deuda técnica inmanejable. clrinf lo resuelve con 8 mecanismos:",
    items: [
      {
        icon: "🤖",
        title: "Generación Caótica con IA y Deuda Técnica",
        desc: "Sin límites arquitectónicos, los modelos de IA inventan patrones arbitrarios y acoplan la lógica de dominio con frameworks externos.",
        solution: "Generación controlada de código mediante plantillas arquitectónicas, herramientas Meta MCP y linters en tiempo de compilación para que el agente trabaje sobre patrones verificados.",
      },
      {
        icon: "📦",
        title: "Atadura Monolítica y Dependencias Pesadas",
        desc: "Intentar agregar una funcionalidad CRM o autorización Cedar a un proyecto existente arrastra árboles masivos de paquetes y dependencias rígidas.",
        solution: "Motor clrinf module adopt: Trasplante cualquier módulo sin dependencias externas en modo puro (raw), o cableado a la inyección de dependencias nativa (wired).",
      },
      {
        icon: "🛡️",
        title: "Fugas de Datos Multinquilino e Inseguridad",
        desc: "Cuando las comprobaciones de inquilino se reparten manualmente en el código, los desarrolladores o agentes de IA olvidan filtros y generan brechas graves.",
        solution: "Políticas Amazon Cedar multinquilino por defecto (policies/*.cedar): context.tenant_id != resource.tenant_id activa una guardia de denegación absoluta (forbid).",
      },
      {
        icon: "📐",
        title: "Desfase de Contratos Políglotas (Contract Drift)",
        desc: "Un desarrollador de C# actualiza un modelo y los equipos de Rust o TypeScript olvidan campos al escribir DTOs manuales.",
        solution: "Esquemas canónicos de CloudEvents 1.0 y JSON Schema Draft 2020-12 generan tipos 100% libres de errores para todos los lenguajes.",
      },
      {
        icon: "📡",
        title: "Pérdida Silenciosa de Eventos en Microservicios",
        desc: "Un servicio emite un evento que nadie escucha (Dead Event), o se suscribe a un evento inexistente. El fallo sólo se detecta en producción.",
        solution: "El motor topology check de clrinf detecta eventos muertos y suscriptores huérfanos antes de que el código llegue a CI.",
      },
      {
        icon: "⚡",
        title: "Imposición Forzada de Patrones Ajenos",
        desc: "Copiar clases MediatR de C# a Rust o TypeScript rompe la elegancia natural de esos lenguajes con apaños de reflexión.",
        solution: "Cada lenguaje mantiene sus modismos: C# con despachador nativo FrozenDictionary o MediatR, Rust sin pánicos, TypeScript funcional.",
      },
      {
        icon: "🔌",
        title: "Atadura a Proveedores y Agentes de Mensajería",
        desc: "Muchos marcos imponen Kafka, NATS o AWS SQS desde el primer día, dificultando las pruebas locales y el despliegue monolítico.",
        solution: "Puertos OutboxStore e IdempotencyStore agnósticos al transporte permiten ejecutar como monolito modular en memoria o escalar a colas distribuidas.",
      },
      {
        icon: "🛡️",
        title: "Violaciones de Capas y Acoplamientos Ocultos",
        desc: "Desarrolladores o IA conectan bases de datos a la capa de dominio, inyectan ORMs en controladores o lanzan pánicos no controlados.",
        solution: "Roslyn (ARCH001-004), Syn (RUST_ARCH001) y TS AST (ARCH_TS_001) imponen límites en tiempo de compilación y análisis.",
      },
    ],
  },
  moduleStudio: {
    tag: "TRASPLANTE DE MÓDULOS SIN FRICCIÓN",
    title: "Catálogo de Módulos Incorporados y Estudio de Adaptación",
    subtitle: "Trasplante suites de CRM e infraestructura incorporadas a CUALQUIER proyecto (.csproj, Cargo.toml, package.json) con cero fricción: en modo puro sin dependencias (raw) o integrado con todo el ecosistema (wired).",
    catalogBadge: "CATÁLOGO OFICIAL INCORPORADO",
    selectModule: "Seleccione el Módulo a Trasplantar:",
    selectMode: "Modo de Adaptación:",
    modes: {
      raw: {
        title: "Puro (Raw / Zero-Dependency)",
        desc: "Cero dependencias externas. Contratos autosuficientes (OperationClaim), lógica de dominio aislada, puertos de repositorio y política Cedar.",
      },
      wired: {
        title: "Cableado (Wired / Integración Completa)",
        desc: "Aprovecha todo clrinf. Conexión automática a DI (IServiceCollection), árbol de módulos (pub mod), exportaciones (export * as) y guardas Cedar.",
      },
    },
    selectTarget: "Tipo de Proyecto Destino:",
    targets: {
      csharp: "C# (.csproj)",
      rust: "Rust (Cargo.toml)",
      ts: "TypeScript (package.json)",
    },
    cliCommandLabel: "Comando de Adaptación Generado",
    generatedFilesLabel: "Archivos Generados / Conectados",
    codePreviewTabs: {
      code: "Código del Módulo",
      cedar: "Política de Seguridad Cedar",
      wiring: "Cableado / Inyección DI",
    },
    copyBtn: "Copiar Comando",
    copiedBtn: "¡Copiado!",
  },
  simulator: {
    tag: "EXPERIENCIA INTERACTIVA EN VIVO",
    title: "Simulador de Topología Entre Servicios",
    subtitle: "Compruebe cómo el motor de clrinf captura eventos muertos, suscriptores huérfanos y brechas de versiones en tiempo real.",
    scenarios: {
      valid: "Flujo Normal (Verificado)",
      deadEvent: "Evento Muerto (Sin Suscriptores)",
      orphanSub: "Suscriptor Huérfano (Sin Emisor)",
      missingUpcaster: "Falta Upcaster (v1 ➔ v2)",
    },
    statusTitle: "Diagnóstico de Topología",
    statusMessages: {
      valid: "✅ TOPOLOGÍA VERIFICADA: 'order-service' publica 'ordering.order.placed.v1', consumido por 'billing-service'. Cero riesgo de pérdida de eventos.",
      deadEvent: "⚠️ [TOPOLOGY_DEAD_EVENT] ADVERTENCIA: 'payment-service' publica 'payment.completed.v1' pero ningún servicio activo lo escucha. El evento se pierde en el vacío.",
      orphanSub: "❌ [TOPOLOGY_ORPHAN_SUBSCRIBER] ERROR FATAL: 'notification-service' se suscribe a 'invoice.issued.v1', ¡pero ningún servicio en el ecosistema lo produce!",
      missingUpcaster: "❌ [TOPOLOGY_UPCASTER_MISSING] ERROR FATAL: 'order-service' publica 'v1' mientras 'shipping-service' espera 'v2'. ¡No existe definición de migración upcaster 'v1 -> v2'!",
    },
  },
  codeMatrix: {
    tag: "DISEÑO IDIOMÁTICO MULTILENGUAJE",
    title: "Una Regla de Negocio, Nativa en Tres Lenguajes",
    subtitle: "Observe cómo la misma regla (Monto mínimo de pedido) se implementa de manera nativa e idiomática en C#, Rust y TypeScript.",
    tabs: {
      csharp: "C# (.NET 10)",
      rust: "Rust (Sin Pánicos)",
      ts: "TypeScript (Funcional)",
    },
  },
  docs: {
    searchPlaceholder: "Buscar en la documentación (ej. ARCH001, pipeRules, Outbox)...",
    chapters: {
      intro: "Introducción y Filosofía",
      quickstart: "Instalación y Guía Rápida",
      moduleAdopt: "Catálogo y Adaptación de Módulos",
      cedarAuthz: "Autorización con Amazon Cedar",
      profilesDispatchers: "Perfiles y Despachadores",
      aiGovernance: "Gobernanza de Agentes IA (Meta MCP)",
      linters: "Linters Arquitectónicos",
      rulesEngine: "Motor de Reglas de Negocio",
      topology: "Topología y Pub/Sub",
      monolithDist: "Monolito vs Distribuido",
      cliReference: "Referencia de Comandos CLI",
    },
  },
  footer: {
    desc: "Límites constitucionales para agentes de IA: núcleo de aplicación contract-first, políglota y sostenible.",
    sections: {
      ecosystem: "Ecosistema",
      tooling: "Herramientas e IA",
      governance: "Gobernanza",
    },
    links: {
      contracts: "clrinf-contracts",
      csharp: "clrinfcs (C#)",
      rust: "clrinfrs (Rust)",
      ts: "clrinfjs (TS)",
      cli: "clrinf-codegen CLI",
      mcp: "Servidor Meta MCP",
      scaffold: "Plantillas Arquitectónicas",
      topology: "Motor de Topología",
      constitution: "Constitución",
      versioning: "Política de Versiones",
      license: "Licencia BSL 1.1",
    },
    rights: "Todos los derechos reservados. Licenciado bajo BSL 1.1. Fecha de cambio: 2030-09-08 (Apache-2.0).",
  },
};
