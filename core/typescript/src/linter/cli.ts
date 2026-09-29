import { ArchitecturalLinter } from "./arch-linter";
import { resolve } from "node:path";

async function main() {
  const args = process.argv.slice(2);
  const targetPath = resolve(process.cwd(), args[0] ?? ".");

  console.log(`[clrinf-linter] Scanning architectural boundaries in: ${targetPath}`);
  const linter = new ArchitecturalLinter();
  const result = await linter.lintPath(targetPath);

  console.log(`[clrinf-linter] Scanned ${result.totalFiles} matching files.`);

  if (result.violations.length === 0) {
    console.log("✓ No architectural violations detected. TigerStyle compliance verified.");
    process.exit(0);
  }

  console.error(`\nFound ${result.violations.length} architectural violation(s):\n`);
  for (const v of result.violations) {
    console.error(`[${v.code}] ${v.file}:${v.line}:${v.character}`);
    console.error(`  ${v.message}\n`);
  }

  process.exit(1);
}

if (import.meta.main) {
  await main();
}
