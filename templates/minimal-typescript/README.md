# Minimal TypeScript

Standalone application service / modular monolith starter with no package
dependencies. Use an installed Bun 1.x, or Node.js 18+ with TypeScript 5.x+
(`tsc`) already installed. No `npm install` or network access is needed.

From this directory, run directly with Bun:

```sh
bun run src/main.ts
bun run tests/greeting.test.ts
```

Or compile and run with TypeScript and Node:

```sh
tsc -p tsconfig.json
node dist/src/main.js
node dist/tests/greeting.test.js
```

The demo prints `Hello, World! [demo-request]`. Tests throw on failure.
`npm run build`, `npm start`, and `npm test` wrap the Node workflow when
`tsc` is on PATH. Bun execution does not type-check; run `tsc` for type checking.
The compiler's built-in DOM declarations supply `console` types only; this
project uses no browser APIs or downloaded Node type declarations.

`src/core.ts` defines a generic application module boundary, direct dispatch,
context, and discriminated success/failure results. `src/greeting.ts` owns its
typed request, response, and blank-name validation. `src/main.ts` is the
composition root. Add modules behind the interface without adding a broker.
External untyped input would need validation in a transport adapter.

These are local example types, not generated wire contracts or an SDK dependency.
The context ID is caller-supplied tracing metadata, not authentication or an
idempotency key. This console host does not listen on a network port. There is
no persistence, retry, transaction, outbox, delivery, or exactly-once guarantee.
Add explicit adapters and their tests only when your application needs them.
