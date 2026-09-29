#!/usr/bin/env bun
// @clrinf:generated — TypeScript Generator CLI
import { resolve } from "node:path";
import { ProjectGenerator } from "./project-generator.js";
import { ModuleGenerator } from "./module-generator.js";
import { EntityGenerator } from "./entity-generator.js";
import { RuleGenerator } from "./rule-generator.js";

function printUsage() {
  console.log(`clrinfjs-gen: TypeScript Scaffolding Generator for clrinf

Kullanım:
  bun run src/generator/cli.ts new <name> [--path <dir>]
  bun run src/generator/cli.ts add module <name> [--project <path>]
  bun run src/generator/cli.ts add entity <name> -m <module> [--prop name:type ...] [--project <path>]
  bun run src/generator/cli.ts add rule <name> [--target <dir>] [--context <type>] [--error-code <code>] [--message <msg>]
`);
}

async function main() {
  const args = process.argv.slice(2);
  if (args.length === 0 || args.includes("--help") || args.includes("-h")) {
    printUsage();
    process.exit(0);
  }

  const cmd = args[0];

  if (cmd === "new") {
    const name = args[1];
    if (!name) {
      console.error("Hata: Proje adı belirtilmelidir. Örnek: bun run src/generator/cli.ts new StoreApp");
      process.exit(1);
    }
    let targetDir = ".";
    const pathIdx = args.findIndex((a) => a === "--path" || a === "-p");
    if (pathIdx !== -1) {
      const val = args[pathIdx + 1];
      if (val) targetDir = val;
    }

    const generator = new ProjectGenerator();
    const files = await generator.scaffoldProject({
      projectName: name,
      targetDir: resolve(process.cwd(), targetDir),
    });
    console.log(`✔ Proje '${name}' başarıyla oluşturuldu (${files.length} dosya).`);
    process.exit(0);
  }

  if (cmd === "add") {
    const subCmd = args[1];
    if (!subCmd) {
      console.error("Hata: Alt komut belirtilmelidir: module, entity, rule");
      process.exit(1);
    }

    if (subCmd === "module") {
      const name = args[2];
      if (!name) {
        console.error("Hata: Modül adı belirtilmelidir. Örnek: bun run src/generator/cli.ts add module Orders");
        process.exit(1);
      }
      let projectRoot = ".";
      const projIdx = args.findIndex((a) => a === "--project" || a === "-p");
      if (projIdx !== -1) {
        const val = args[projIdx + 1];
        if (val) projectRoot = val;
      }

      const generator = new ModuleGenerator();
      const files = await generator.scaffoldModule({
        moduleName: name,
        projectRoot: resolve(process.cwd(), projectRoot),
      });
      console.log(`✔ Modül '${name}' başarıyla oluşturuldu (${files.length} dosya).`);
      process.exit(0);
    }

    if (subCmd === "entity") {
      const name = args[2];
      if (!name) {
        console.error("Hata: Entity adı belirtilmelidir. Örnek: bun run src/generator/cli.ts add entity OrderItem -m Orders");
        process.exit(1);
      }
      let moduleName = "";
      const modIdx = args.findIndex((a) => a === "-m" || a === "--module");
      if (modIdx !== -1) {
        const val = args[modIdx + 1];
        if (val) moduleName = val;
      }
      if (!moduleName) {
        console.error("Hata: '-m' veya '--module' ile ana modül adı belirtilmelidir.");
        process.exit(1);
      }

      let projectRoot = ".";
      const projIdx = args.findIndex((a) => a === "--project");
      if (projIdx !== -1) {
        const val = args[projIdx + 1];
        if (val) projectRoot = val;
      }

      const properties: Array<{ name: string; type: string }> = [];
      for (let i = 0; i < args.length; i++) {
        if ((args[i] === "-p" || args[i] === "--prop") && args[i + 1]) {
          const propArg = args[i + 1];
          if (propArg) {
            const [pName, pType] = propArg.split(":");
            if (pName) {
              properties.push({ name: pName, type: pType || "string" });
            }
          }
        }
      }

      const generator = new EntityGenerator();
      const files = await generator.scaffoldEntity({
        entityName: name,
        moduleName,
        projectRoot: resolve(process.cwd(), projectRoot),
        properties,
      });
      console.log(`✔ Entity '${name}' başarıyla '${moduleName}' modülüne eklendi (${files.length} dosya).`);
      process.exit(0);
    }

    if (subCmd === "rule") {
      const name = args[2];
      if (!name) {
        console.error("Hata: Kural adı belirtilmelidir. Örnek: bun run src/generator/cli.ts add rule CheckBalance");
        process.exit(1);
      }
      let targetDir = "src/rules";
      const tIdx = args.findIndex((a) => a === "-t" || a === "--target");
      if (tIdx !== -1) {
        const val = args[tIdx + 1];
        if (val) targetDir = val;
      }

      let contextType = "EntityContext";
      const cIdx = args.findIndex((a) => a === "-c" || a === "--context");
      if (cIdx !== -1) {
        const val = args[cIdx + 1];
        if (val) contextType = val;
      }

      let errorCode = `${name.toUpperCase()}_FAILED`;
      const eIdx = args.findIndex((a) => a === "-e" || a === "--error-code");
      if (eIdx !== -1) {
        const val = args[eIdx + 1];
        if (val) errorCode = val;
      }

      let message = `Validation failed for ${name}`;
      const mIdx = args.findIndex((a) => a === "-m" || a === "--message");
      if (mIdx !== -1) {
        const val = args[mIdx + 1];
        if (val) message = val;
      }

      const generator = new RuleGenerator();
      const res = await generator.scaffoldRule({
        ruleName: name,
        targetDir: resolve(process.cwd(), targetDir),
        contextType,
        errorCode,
        defaultMessage: message,
      });
      console.log(`✔ Kural '${name}' oluşturuldu: ${res.rulePath}`);
      process.exit(0);
    }
  }

  console.error(`Bilinmeyen komut: ${cmd}`);
  printUsage();
  process.exit(1);
}

if (import.meta.main) {
  await main();
}
