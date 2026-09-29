import { startInspector } from "./inspector";
export { startInspector } from "./inspector";

if (import.meta.main) {
  const inspector = await startInspector({
    port: Number(process.env.PORT ?? 4200),
    hostname: process.env.HOST ?? "127.0.0.1",
    ...(process.env.NATS_URL ? { natsUrl: process.env.NATS_URL } : {}),
    ...(process.env.INSPECTOR_TOKEN ? { token: process.env.INSPECTOR_TOKEN } : {}),
    allowUnauthenticated: process.env.INSPECTOR_ALLOW_UNAUTHENTICATED === "true",
  });
  console.log(`[clrinfjs] Development Event Inspector: ${inspector.server.url}`);
  const shutdown = () => {
    void inspector.stop().catch(error => { console.error("Inspector shutdown failed", error); process.exitCode = 1; });
  };
  process.once("SIGINT", shutdown);
  process.once("SIGTERM", shutdown);
}
