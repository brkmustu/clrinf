// @clrinf:generated — TypeScript Project Scaffolding Generator
import { mkdir, writeFile } from "node:fs/promises";
import { join } from "node:path";

export interface ProjectGeneratorOptions {
  projectName: string;
  targetDir: string;
  arch?: "flat" | "layered" | "clean-cqrs";
}

export class ProjectGenerator {
  static async generate(options: ProjectGeneratorOptions): Promise<string[]> {
    const arch = options.arch ?? "clean-cqrs";
    const projectRoot = join(options.targetDir, options.projectName);
    const createdFiles: string[] = [];

    await mkdir(projectRoot, { recursive: true });

    // 1. package.json
    const packageJson = {
      name: options.projectName.toLowerCase(),
      version: "0.1.0",
      type: "module",
      scripts: {
        build: "tsc",
        start: "bun run src/index.ts",
        test: "bun test",
      },
      dependencies: {
        "@clrinf/core": "^0.1.0",
      },
      devDependencies: {
        "@types/bun": "latest",
        typescript: "^5.0.0",
      },
    };

    const pkgPath = join(projectRoot, "package.json");
    await writeFile(pkgPath, JSON.stringify(packageJson, null, 2) + "\n");
    createdFiles.push(pkgPath);

    // 2. tsconfig.json
    const tsconfigJson = {
      compilerOptions: {
        target: "ESNext",
        module: "ESNext",
        moduleResolution: "bundler",
        strict: true,
        esModuleInterop: true,
        skipLibCheck: true,
        outDir: "./dist",
      },
      include: ["src/**/*"],
    };

    const tsconfigPath = join(projectRoot, "tsconfig.json");
    await writeFile(tsconfigPath, JSON.stringify(tsconfigJson, null, 2) + "\n");
    createdFiles.push(tsconfigPath);

    // 3. clrinf.toml
    const clrinfToml = `[project]
name = "${options.projectName}"
lang = "typescript"
arch = "${arch}"
deployment = "monolith"

[modules.caching]
enabled = true
provider = "memory"

[modules.logging]
enabled = true
provider = "structured"

[modules.transaction]
enabled = true
provider = "native"

[modules.authentication]
enabled = true
provider = "jwt"

[modules.authorization]
enabled = true
provider = "cedar"
policy_path = "policies"

[modules.idempotency]
enabled = true
provider = "memory"

[modules.outbox]
enabled = true
provider = "memory"
`;

    const tomlPath = join(projectRoot, "clrinf.toml");
    await writeFile(tomlPath, clrinfToml);
    createdFiles.push(tomlPath);

    // 4. Source structure based on arch
    const srcDir = join(projectRoot, "src");
    await mkdir(srcDir, { recursive: true });

    if (arch === "flat") {
      const indexPath = join(srcDir, "index.ts");
      const indexContent = `// @clrinf:generated — ${options.projectName} Flat Entry Point
import { ConsoleLogger } from "@clrinf/core";

const logger = new ConsoleLogger("${options.projectName}");
logger.info("Application started");
`;
      await writeFile(indexPath, indexContent);
      createdFiles.push(indexPath);
    } else {
      // Layered / Clean-CQRS
      const modulesDir = join(srcDir, "modules");
      await mkdir(modulesDir, { recursive: true });

      const modIndexPath = join(modulesDir, "index.ts");
      await writeFile(modIndexPath, `// @clrinf:generated — Modules barrel\n`);
      createdFiles.push(modIndexPath);

      const indexPath = join(srcDir, "index.ts");
      const indexContent = `// @clrinf:generated — ${options.projectName} Clean-CQRS Entry Point
import { ConsoleLogger } from "@clrinf/core";
import "./modules/index.js";

const logger = new ConsoleLogger("${options.projectName}");
logger.info("${options.projectName} initialized (${arch})");
`;
      await writeFile(indexPath, indexContent);
      createdFiles.push(indexPath);
    }

    // 5. Policies directory with initial cedar config
    const policiesDir = join(projectRoot, "policies");
    await mkdir(policiesDir, { recursive: true });

    return createdFiles;
  }

  async scaffoldProject(options: ProjectGeneratorOptions): Promise<string[]> {
    return ProjectGenerator.generate(options);
  }

  async generate(options: ProjectGeneratorOptions): Promise<string[]> {
    return ProjectGenerator.generate(options);
  }
}

