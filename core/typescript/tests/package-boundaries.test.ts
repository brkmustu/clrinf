import { expect, it } from "bun:test";

it("fails explicitly when conformance fixtures are missing", async () => {
  const process = Bun.spawn(["bun", "test", "tests/conformance.test.ts"], {
    cwd: new URL("..", import.meta.url).pathname,
    env: { ...Bun.env, CLRINF_CONFORMANCE_DIR: new URL("./missing-fixtures", import.meta.url).pathname },
    stdout: "pipe", stderr: "pipe",
  });
  expect(await process.exited).not.toBe(0);
  expect(await new Response(process.stderr).text()).toContain("Required conformance fixture missing:");
});

it("does not retain legacy abandoned cli directory", async () => {
  const cliExists = await Bun.file(new URL("../cli/package.json", import.meta.url)).exists();
  expect(cliExists).toBe(false);
});
