import ts from "typescript";
import { readdir, readFile, stat } from "node:fs/promises";
import { join } from "node:path";

export interface LintViolation {
  code: string;
  message: string;
  file: string;
  line: number;
  character: number;
}

export interface LintResult {
  totalFiles: number;
  violations: LintViolation[];
}

export interface LinterOptions {
  includePatterns?: RegExp[];
  excludePatterns?: RegExp[];
}

export class ArchitecturalLinter {
  private defaultIncludePatterns = [
    /\.rules?\.(ts|js)$/,
    /\.service\.(ts|js)$/,
    /rules\/.*\.ts$/,
  ];

  private defaultExcludePatterns = [
    /node_modules/,
    /\.git/,
    /\.test\.(ts|js)$/,
    /dist/,
  ];

  /**
   * Lints a single source string for architectural violations.
   */
  lintSource(sourceText: string, fileName: string): LintViolation[] {
    const sourceFile = ts.createSourceFile(
      fileName,
      sourceText,
      ts.ScriptTarget.Latest,
      true
    );

    const violations: LintViolation[] = [];
    const isRuleOrServiceFile = this.defaultIncludePatterns.some((p) =>
      p.test(fileName)
    );

    const visit = (node: ts.Node) => {
      // ARCH_TS_001: No raw throw in rules or domain services
      if (isRuleOrServiceFile && ts.isThrowStatement(node)) {
        const { line, character } =
          sourceFile.getLineAndCharacterOfPosition(node.getStart());
        violations.push({
          code: "ARCH_TS_001",
          message:
            "Raw 'throw' statement detected. Business rules and domain services must return 'Result<void, RuleViolation>' using 'ruleFailed()' or 'Result.err()'.",
          file: fileName,
          line: line + 1,
          character: character + 1,
        });
      }

      // ARCH_TS_003: No class-based Mediator/Dispatcher or reflection DI
      if (ts.isClassDeclaration(node) && node.name) {
        const className = node.name.text;
        if (
          className.includes("Dispatcher") ||
          className.includes("Mediator") ||
          className.includes("RulePipeline")
        ) {
          const { line, character } =
            sourceFile.getLineAndCharacterOfPosition(node.getStart());
          violations.push({
            code: "ARCH_TS_003",
            message: `Class-based '${className}' detected. Do not port C# reflection Mediator/Dispatcher to TypeScript. Use functional 'pipeRules' composition instead.`,
            file: fileName,
            line: line + 1,
            character: character + 1,
          });
        }
      }

      ts.forEachChild(node, visit);
    };

    visit(sourceFile);
    return violations;
  }

  /**
   * Lints a file on disk.
   */
  async lintFile(filePath: string): Promise<LintViolation[]> {
    try {
      const content = await readFile(filePath, "utf-8");
      return this.lintSource(content, filePath);
    } catch {
      return [];
    }
  }

  /**
   * Recursively scans a directory or single file and returns all violations.
   */
  async lintPath(
    targetPath: string,
    options?: LinterOptions
  ): Promise<LintResult> {
    const fileList: string[] = [];
    const includes = options?.includePatterns ?? this.defaultIncludePatterns;
    const excludes = options?.excludePatterns ?? this.defaultExcludePatterns;

    const collectFiles = async (currentPath: string) => {
      const st = await stat(currentPath);
      if (st.isDirectory()) {
        const entries = await readdir(currentPath);
        for (const entry of entries) {
          const fullPath = join(currentPath, entry);
          if (excludes.some((p) => p.test(fullPath))) continue;
          await collectFiles(fullPath);
        }
      } else if (st.isFile()) {
        if (
          !excludes.some((p) => p.test(currentPath)) &&
          includes.some((p) => p.test(currentPath))
        ) {
          fileList.push(currentPath);
        }
      }
    };

    try {
      await collectFiles(targetPath);
    } catch {
      // Return empty if path doesn't exist
    }

    const allViolations: LintViolation[] = [];
    for (const file of fileList) {
      const violations = await this.lintFile(file);
      allViolations.push(...violations);
    }

    return {
      totalFiles: fileList.length,
      violations: allViolations,
    };
  }
}
