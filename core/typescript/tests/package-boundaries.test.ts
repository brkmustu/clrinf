import { expect, it } from "bun:test";
import { fileURLToPath } from "node:url";

it("fails explicitly when conformance fixtures are missing", async () => {
  const proc = Bun.spawn([process.execPath, "test", "tests/conformance.test.ts"], {
    cwd: fileURLToPath(new URL("..", import.meta.url)),
    env: { ...Bun.env, CLRINF_CONFORMANCE_DIR: fileURLToPath(new URL("./missing-fixtures", import.meta.url)) },
    stdout: "pipe", stderr: "pipe",
  });
  expect(await proc.exited).not.toBe(0);
  expect(await new Response(proc.stderr).text()).toContain("Required conformance fixture missing:");
});

it("does not retain legacy abandoned cli directory", async () => {
  const cliExists = await Bun.file(new URL("../cli/package.json", import.meta.url)).exists();
  expect(cliExists).toBe(false);
});
