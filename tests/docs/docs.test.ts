import { describe, expect, test } from "bun:test";
import { existsSync, readdirSync, readFileSync } from "node:fs";
import { dirname, join, resolve } from "node:path";

const root = resolve(import.meta.dir, "../..");

function markdownFiles(directory: string): string[] {
  return readdirSync(directory, { withFileTypes: true }).flatMap(entry => {
    const path = join(directory, entry.name);
    return entry.isDirectory() ? markdownFiles(path) : entry.name.endsWith(".md") ? [path] : [];
  });
}

describe("public documentation", () => {
  const files = [join(root, "README.md"), ...markdownFiles(join(root, "docs"))];

  for (const file of files) {
    test(`${file.slice(root.length + 1)} has resolvable local links`, () => {
      const content = readFileSync(file, "utf8");
      expect(content).not.toContain("file:///");
      for (const match of content.matchAll(/\[[^\]]*\]\(([^)\s]+)(?:\s+"[^"]*")?\)/g)) {
        const target = match[1];
        if (/^(?:https?:|mailto:|#)/.test(target)) continue;
        const path = decodeURIComponent(target.split("#")[0]);
        expect(existsSync(resolve(dirname(file), path)), `Missing link in ${file}: ${target}`).toBe(true);
      }
    });
  }

  test("core schema directory excludes example business domains", () => {
    for (const name of ["eticaret", "kampanya", "katalog", "oms", "stok", "egitim"]) {
      expect(existsSync(join(root, "tools/clrinf-codegen/schemas", name))).toBe(false);
      expect(existsSync(join(root, "examples/contracts/schemas", name))).toBe(true);
    }
  });

  test("generic docs do not contain commerce blueprints", () => {
    for (const name of [
      "01-katalog-alan-rehberi.md",
      "02-eticaret-odeme-rehberi.md",
      "03-kampanya-promosyon-rehberi.md",
      "04-siparis-yonetimi-oms-rehberi.md",
    ]) {
      expect(existsSync(join(root, "docs/blueprints", name))).toBe(false);
      expect(existsSync(join(root, "templates/eticaret-showcase/docs", name))).toBe(true);
    }
  });
});
