// @clrinf:generated — TypeScript Entity Scaffolding Generator
import { mkdir, readFile, writeFile } from "node:fs/promises";
import { join } from "node:path";
import { ModuleGenerator } from "./module-generator.js";

export interface EntityProperty {
  name: string;
  type: string;
}

export interface EntityGeneratorOptions {
  projectRoot: string;
  moduleName: string;
  entityName: string;
  properties?: EntityProperty[];
}

function toKebabCase(str: string): string {
  return str
    .replace(/([a-z0-9]|(?=[A-Z]))([A-Z])/g, "$1-$2")
    .toLowerCase()
    .replace(/^-+|-+$/g, "");
}

function toPascalCase(str: string): string {
  return str
    .split(/[-_\s]+/)
    .map((w) => w.charAt(0).toUpperCase() + w.slice(1).toLowerCase())
    .join("");
}

export class EntityGenerator {
  static async generate(options: EntityGeneratorOptions): Promise<string[]> {
    const modSlug = toKebabCase(options.moduleName);
    const entitySlug = toKebabCase(options.entityName);
    const entityPascal = toPascalCase(options.entityName);
    const createdFiles: string[] = [];

    const moduleDir = join(options.projectRoot, "src", "modules", modSlug);

    // If module directory doesn't exist, generate module first
    try {
      await readFile(join(moduleDir, "index.ts"));
    } catch {
      await ModuleGenerator.generate({
        projectRoot: options.projectRoot,
        moduleName: options.moduleName,
      });
    }

    // 1. Build TypeScript property definitions
    const props = options.properties ?? [];
    let propDefs = "";
    for (const p of props) {
      const tsType = matchTsType(p.type);
      propDefs += `  ${p.name}: ${tsType};\n`;
    }

    // 2. Generate <entity-slug>.entity.ts
    const entityFilePath = join(moduleDir, `${entitySlug}.entity.ts`);
    const entityContent = `// @clrinf:generated — ${entityPascal} Entity, Repository Port, and CRUD Operations
import type { OperationClaim } from "./handlers.js";

/**
 * ${entityPascal} Domain Entity
 */
export interface ${entityPascal} {
  id: string;
  tenant_id: string;
${propDefs}  created_at: number;
  updated_at: number;
}

/**
 * ${entityPascal} Repository Port (Data Access Abstraction)
 */
export interface ${entityPascal}Repository {
  getById(tenantId: string, id: string): Promise<${entityPascal} | null>;
  listByTenant(tenantId: string): Promise<${entityPascal}[]>;
  insert(entity: ${entityPascal}): Promise<${entityPascal}>;
  update(entity: ${entityPascal}): Promise<${entityPascal}>;
  delete(tenantId: string, id: string): Promise<boolean>;
}

/**
 * In-Memory ${entityPascal} Repository Implementation
 */
export class InMemory${entityPascal}Repository implements ${entityPascal}Repository {
  private readonly storage = new Map<string, ${entityPascal}>();

  private key(tenantId: string, id: string): string {
    return \`\${tenantId}:\${id}\`;
  }

  async getById(tenantId: string, id: string): Promise<${entityPascal} | null> {
    return this.storage.get(this.key(tenantId, id)) ?? null;
  }

  async listByTenant(tenantId: string): Promise<${entityPascal}[]> {
    return Array.from(this.storage.values()).filter((e) => e.tenant_id === tenantId);
  }

  async insert(entity: ${entityPascal}): Promise<${entityPascal}> {
    this.storage.set(this.key(entity.tenant_id, entity.id), { ...entity });
    return entity;
  }

  async update(entity: ${entityPascal}): Promise<${entityPascal}> {
    this.storage.set(this.key(entity.tenant_id, entity.id), { ...entity, updated_at: Date.now() });
    return entity;
  }

  async delete(tenantId: string, id: string): Promise<boolean> {
    return this.storage.delete(this.key(tenantId, id));
  }
}

/**
 * Multi-Tenant Operation Claims for ${entityPascal}
 */
export const ${entityPascal}OperationClaims = {
  create: (tenantId: string): OperationClaim => ({
    resource: "${entitySlug}",
    action: "create",
    resource_tenant_id: tenantId,
  }),
  read: (tenantId: string): OperationClaim => ({
    resource: "${entitySlug}",
    action: "read",
    resource_tenant_id: tenantId,
  }),
  update: (tenantId: string): OperationClaim => ({
    resource: "${entitySlug}",
    action: "update",
    resource_tenant_id: tenantId,
  }),
  delete: (tenantId: string): OperationClaim => ({
    resource: "${entitySlug}",
    action: "delete",
    resource_tenant_id: tenantId,
  }),
};
`;

    await writeFile(entityFilePath, entityContent);
    createdFiles.push(entityFilePath);

    // 3. Register export in module index.ts
    const moduleIndex = join(moduleDir, "index.ts");
    const exportLine = `export * from "./${entitySlug}.entity.js";\n`;
    try {
      const existing = await readFile(moduleIndex, "utf-8");
      if (!existing.includes(exportLine)) {
        await writeFile(moduleIndex, existing + exportLine);
      }
    } catch {
      await writeFile(moduleIndex, exportLine);
    }

    return createdFiles;
  }

  async scaffoldEntity(options: EntityGeneratorOptions): Promise<string[]> {
    return EntityGenerator.generate(options);
  }

  async generate(options: EntityGeneratorOptions): Promise<string[]> {
    return EntityGenerator.generate(options);
  }
}

function matchTsType(type: string): string {
  switch (type.toLowerCase()) {
    case "string":
    case "str":
    case "text":
      return "string";
    case "int":
    case "i32":
    case "i64":
    case "number":
    case "float":
    case "double":
      return "number";
    case "bool":
    case "boolean":
      return "boolean";
    default:
      return type;
  }
}
