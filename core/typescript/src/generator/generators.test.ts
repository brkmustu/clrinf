import { describe, expect, it } from "bun:test";
import { rm } from "node:fs/promises";
import { join } from "node:path";
import { tmpdir } from "node:os";
import { existsSync } from "node:fs";
import { ProjectGenerator, ModuleGenerator, EntityGenerator } from "./index";

describe("TypeScript Scaffolding Generators", () => {
  const testRoot = join(tmpdir(), `clrinf_ts_gen_test_${Date.now()}`);

  it("ProjectGenerator scaffolds project with manifest and source structure", async () => {
    const files = await ProjectGenerator.generate({
      projectName: "ShopApp",
      targetDir: testRoot,
      arch: "clean-cqrs",
    });

    const projRoot = join(testRoot, "ShopApp");
    expect(existsSync(join(projRoot, "package.json"))).toBe(true);
    expect(existsSync(join(projRoot, "tsconfig.json"))).toBe(true);
    expect(existsSync(join(projRoot, "clrinf.toml"))).toBe(true);
    expect(existsSync(join(projRoot, "src", "index.ts"))).toBe(true);
    expect(existsSync(join(projRoot, "src", "modules", "index.ts"))).toBe(true);
    expect(files.length).toBeGreaterThanOrEqual(4);
  });

  it("ModuleGenerator scaffolds module with multi-tenant Cedar policy", async () => {
    const projRoot = join(testRoot, "ShopApp");
    const files = await ModuleGenerator.generate({
      projectRoot: projRoot,
      moduleName: "Catalog",
    });

    expect(existsSync(join(projRoot, "src", "modules", "catalog", "models.ts"))).toBe(true);
    expect(existsSync(join(projRoot, "src", "modules", "catalog", "handlers.ts"))).toBe(true);
    expect(existsSync(join(projRoot, "src", "modules", "catalog", "rules", "index.ts"))).toBe(true);
    expect(existsSync(join(projRoot, "src", "modules", "catalog", "index.ts"))).toBe(true);
    expect(existsSync(join(projRoot, "policies", "catalog.cedar"))).toBe(true);
  });

  it("EntityGenerator scaffolds CRUD entity with repository port and tenant claims", async () => {
    const projRoot = join(testRoot, "ShopApp");
    const files = await EntityGenerator.generate({
      projectRoot: projRoot,
      moduleName: "Catalog",
      entityName: "Product",
      properties: [
        { name: "title", type: "string" },
        { name: "price", type: "number" },
        { name: "inStock", type: "boolean" },
      ],
    });

    const entityFile = join(projRoot, "src", "modules", "catalog", "product.entity.ts");
    expect(existsSync(entityFile)).toBe(true);

    const fileContent = await Bun.file(entityFile).text();
    expect(fileContent).toContain("title: string;");
    expect(fileContent).toContain("price: number;");
    expect(fileContent).toContain("inStock: boolean;");
    expect(fileContent).toContain("InMemoryProductRepository");
    expect(fileContent).toContain("ProductOperationClaims");

    // Clean up
    await rm(testRoot, { recursive: true, force: true });
  });
});
