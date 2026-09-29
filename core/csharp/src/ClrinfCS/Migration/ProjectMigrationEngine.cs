using System;
using System.Collections.Generic;
using System.Diagnostics;
using System.IO;
using System.Linq;
using System.Text;
using System.Threading.Tasks;
using Application.Common.Configuration;
using Domain.Configuration.Models;
using Spectre.Console;

namespace ClrinfCS.Migration;

public class MigrationPlanItem
{
    public string SourceFilePath { get; set; } = string.Empty;
    public string DestinationFilePath { get; set; } = string.Empty;
    public CodeElementCategory Category { get; set; }
    public string? OldNamespace { get; set; }
    public string? NewNamespace { get; set; }
}

public class MigrationResult
{
    public bool Success { get; set; }
    public string Message { get; set; } = string.Empty;
    public List<MigrationPlanItem> PlanItems { get; set; } = new();
    public List<string> CreatedProjects { get; set; } = new();
}

public class ProjectMigrationEngine
{
    public static async Task<MigrationResult> MigrateAsync(
        string projectPath,
        string targetArch,
        bool isDryRun = false)
    {
        targetArch = targetArch.Trim().ToLowerInvariant();
        if (targetArch != "layered" && targetArch != "clean-cqrs")
        {
            return new MigrationResult
            {
                Success = false,
                Message = $"Geçersiz hedef mimari: '{targetArch}'. Yalnızca 'layered' veya 'clean-cqrs' desteklenir."
            };
        }

        string configPath = Path.Combine(projectPath, "codegen.toml");
        if (!File.Exists(configPath))
            configPath = Path.Combine(projectPath, "codegen.yaml");

        var config = File.Exists(configPath) ? ConfigLoader.Load(configPath) : null;

        string projectName = config?.Backend?.Namespace ?? Path.GetFileName(projectPath.TrimEnd(Path.DirectorySeparatorChar, Path.AltDirectorySeparatorChar));
        string currentArch = config?.Backend?.ArchStyle?.ToLowerInvariant() ?? detectCurrentArch(projectPath, projectName);
        string hostType = config?.Backend?.HostType?.ToLowerInvariant() ?? "api";
        string hostSuffix = hostType == "mvc" ? "Web" : (hostType == "console" ? "Console" : "API");

        if (currentArch.Equals(targetArch, StringComparison.OrdinalIgnoreCase))
        {
            return new MigrationResult
            {
                Success = true,
                Message = $"Proje zaten '{targetArch}' mimarisinde bulunmaktadır. Herhangi bir değişiklik yapılmadı."
            };
        }

        if (currentArch == "clean-cqrs" && (targetArch == "layered" || targetArch == "flat"))
        {
            return new MigrationResult
            {
                Success = false,
                Message = "Clean-CQRS mimarisinden daha alt bir mimari profiline (downgrade) geçiş desteklenmemektedir."
            };
        }

        var plan = createMigrationPlan(projectPath, projectName, currentArch, targetArch, hostSuffix);

        if (isDryRun)
        {
            return new MigrationResult
            {
                Success = true,
                Message = $"[Dry Run] '{currentArch}' -> '{targetArch}' migrasyon planı başarıyla oluşturuldu. Toplam {plan.Count} dosya taşınacak/güncellenecek.",
                PlanItems = plan
            };
        }

        // Live Execution
        try
        {
            var createdProjects = await executeMigrationPlanAsync(projectPath, projectName, currentArch, targetArch, hostSuffix, config, plan);

            return new MigrationResult
            {
                Success = true,
                Message = $"Proje başarıyla '{currentArch}' mimarisinden '{targetArch}' mimarisine dönüştürüldü!",
                PlanItems = plan,
                CreatedProjects = createdProjects
            };
        }
        catch (Exception ex)
        {
            return new MigrationResult
            {
                Success = false,
                Message = $"Migrasyon sırasında hata oluştu: {ex.Message}"
            };
        }
    }

    private static string detectCurrentArch(string projectPath, string projectName)
    {
        string srcDir = Path.Combine(projectPath, "src");
        if (!Directory.Exists(srcDir))
            srcDir = projectPath;

        var csprojFiles = Directory.GetFiles(srcDir, "*.csproj", SearchOption.AllDirectories)
            .Where(f => !f.Contains("ClrinfCS.Core"))
            .ToList();

        if (csprojFiles.Count <= 1) return "flat";
        if (csprojFiles.Count <= 3) return "layered";
        return "clean-cqrs";
    }

    private static List<MigrationPlanItem> createMigrationPlan(
        string projectPath,
        string projectName,
        string currentArch,
        string targetArch,
        string hostSuffix)
    {
        var plan = new List<MigrationPlanItem>();
        string srcDir = Path.Combine(projectPath, "src");
        if (!Directory.Exists(srcDir)) srcDir = projectPath;

        var allFiles = Directory.GetFiles(srcDir, "*.*", SearchOption.AllDirectories)
            .Where(f => !f.Contains("/bin/") && !f.Contains("/obj/") &&
                        !f.Contains("\\bin\\") && !f.Contains("\\obj\\") &&
                        !f.Contains("ClrinfCS.Core") && !f.EndsWith(".csproj"))
            .ToList();

        foreach (var file in allFiles)
        {
            string content = file.EndsWith(".cs") ? File.ReadAllText(file) : string.Empty;
            var category = file.EndsWith(".cs") ? ClassClassifier.Classify(file, content) : CodeElementCategory.Other;
            string relPath = Path.GetRelativePath(srcDir, file);

            // Strip initial project folder name if present
            var parts = relPath.Split(new[] { '/', '\\' }, StringSplitOptions.RemoveEmptyEntries);
            string subPath = parts.Length > 1 ? string.Join(Path.DirectorySeparatorChar, parts.Skip(1)) : relPath;

            string destFile = file;
            string? newNs = null;

            if (targetArch == "layered")
            {
                // Target: {ProjectName}.Core and {ProjectName}.{hostSuffix}
                if (category == CodeElementCategory.Entity || category == CodeElementCategory.ValueObject)
                {
                    destFile = Path.Combine(srcDir, $"{projectName}.Core", "Entities", Path.GetFileName(file));
                    newNs = $"{projectName}.Core.Entities";
                }
                else if (category == CodeElementCategory.DbContext || category == CodeElementCategory.EntityConfiguration)
                {
                    destFile = Path.Combine(srcDir, $"{projectName}.Core", "Data", Path.GetFileName(file));
                    newNs = $"{projectName}.Core.Data";
                }
                else if (category == CodeElementCategory.Migration)
                {
                    destFile = Path.Combine(srcDir, $"{projectName}.Core", "Migrations", Path.GetFileName(file));
                    newNs = $"{projectName}.Core.Migrations";
                }
                else if (category == CodeElementCategory.ServiceInterface || category == CodeElementCategory.Service)
                {
                    destFile = Path.Combine(srcDir, $"{projectName}.Core", "Services", Path.GetFileName(file));
                    newNs = $"{projectName}.Core.Services";
                }
                else if (category == CodeElementCategory.Controller)
                {
                    destFile = Path.Combine(srcDir, $"{projectName}.{hostSuffix}", "Controllers", Path.GetFileName(file));
                    newNs = $"{projectName}.{hostSuffix}.Controllers";
                }
                else if (category == CodeElementCategory.Endpoint)
                {
                    destFile = Path.Combine(srcDir, $"{projectName}.{hostSuffix}", "Endpoints", Path.GetFileName(file));
                    newNs = $"{projectName}.{hostSuffix}.Endpoints";
                }
                else if (category == CodeElementCategory.HostStartup || file.EndsWith("appsettings.json") || file.EndsWith("appsettings.Development.json"))
                {
                    destFile = Path.Combine(srcDir, $"{projectName}.{hostSuffix}", Path.GetFileName(file));
                }
                else if (file.EndsWith(".cshtml"))
                {
                    destFile = Path.Combine(srcDir, $"{projectName}.{hostSuffix}", subPath);
                }
                else if (relPath.Contains("wwwroot"))
                {
                    destFile = Path.Combine(srcDir, $"{projectName}.{hostSuffix}", subPath);
                }
                else
                {
                    // Default to Core
                    destFile = Path.Combine(srcDir, $"{projectName}.Core", subPath);
                }
            }
            else if (targetArch == "clean-cqrs")
            {
                // Target: Domain, Application, Persistence, Host
                if (category == CodeElementCategory.Entity || category == CodeElementCategory.ValueObject)
                {
                    destFile = Path.Combine(srcDir, $"{projectName}.Domain", "Entities", Path.GetFileName(file));
                    newNs = $"{projectName}.Domain.Entities";
                }
                else if (category == CodeElementCategory.DbContext || category == CodeElementCategory.EntityConfiguration)
                {
                    destFile = Path.Combine(srcDir, $"{projectName}.Persistence", "Contexts", Path.GetFileName(file));
                    newNs = $"{projectName}.Persistence.Contexts";
                }
                else if (category == CodeElementCategory.Migration)
                {
                    destFile = Path.Combine(srcDir, $"{projectName}.Persistence", "Migrations", Path.GetFileName(file));
                    newNs = $"{projectName}.Persistence.Migrations";
                }
                else if (category == CodeElementCategory.ServiceInterface || category == CodeElementCategory.Service ||
                         category == CodeElementCategory.Command || category == CodeElementCategory.Query ||
                         category == CodeElementCategory.Handler || category == CodeElementCategory.Dto)
                {
                    destFile = Path.Combine(srcDir, $"{projectName}.Application", "Services", Path.GetFileName(file));
                    newNs = $"{projectName}.Application.Services";
                }
                else if (category == CodeElementCategory.Controller)
                {
                    destFile = Path.Combine(srcDir, $"{projectName}.{hostSuffix}", "Controllers", Path.GetFileName(file));
                    newNs = $"{projectName}.{hostSuffix}.Controllers";
                }
                else if (category == CodeElementCategory.Endpoint)
                {
                    destFile = Path.Combine(srcDir, $"{projectName}.{hostSuffix}", "Endpoints", Path.GetFileName(file));
                    newNs = $"{projectName}.{hostSuffix}.Endpoints";
                }
                else if (category == CodeElementCategory.HostStartup || file.EndsWith("appsettings.json") || file.EndsWith("appsettings.Development.json"))
                {
                    destFile = Path.Combine(srcDir, $"{projectName}.{hostSuffix}", Path.GetFileName(file));
                }
                else if (file.EndsWith(".cshtml") || relPath.Contains("wwwroot"))
                {
                    destFile = Path.Combine(srcDir, $"{projectName}.{hostSuffix}", subPath);
                }
                else
                {
                    destFile = Path.Combine(srcDir, $"{projectName}.Application", subPath);
                }
            }

            plan.Add(new MigrationPlanItem
            {
                SourceFilePath = file,
                DestinationFilePath = destFile,
                Category = category,
                NewNamespace = newNs
            });
        }

        return plan;
    }

    private static async Task<List<string>> executeMigrationPlanAsync(
        string projectPath,
        string projectName,
        string currentArch,
        string targetArch,
        string hostSuffix,
        CodeGenConfig? config,
        List<MigrationPlanItem> plan)
    {
        string srcDir = Path.Combine(projectPath, "src");
        if (!Directory.Exists(srcDir)) srcDir = projectPath;
        var createdProjects = new List<string>();

        // Find .sln file
        var slnFiles = Directory.GetFiles(projectPath, "*.sln*");
        string? slnFile = slnFiles.Length > 0 ? slnFiles[0] : null;

        if (targetArch == "layered")
        {
            string coreDir = Path.Combine(srcDir, $"{projectName}.Core");
            string hostDir = Path.Combine(srcDir, $"{projectName}.{hostSuffix}");

            if (!Directory.Exists(coreDir))
            {
                await runProcessAsync(srcDir, "dotnet", $"new classlib -n {projectName}.Core -f net10.0");
                createdProjects.Add($"{projectName}.Core");
                // Remove Class1.cs if generated
                string c1 = Path.Combine(coreDir, "Class1.cs");
                if (File.Exists(c1)) File.Delete(c1);
            }

            if (!Directory.Exists(hostDir))
            {
                string template = hostSuffix == "Web" ? "web" : (hostSuffix == "Console" ? "console" : "webapi");
                await runProcessAsync(srcDir, "dotnet", $"new {template} -n {projectName}.{hostSuffix} -f net10.0");
                createdProjects.Add($"{projectName}.{hostSuffix}");
            }

            // Reference Core from Host
            string coreCsproj = Path.Combine(coreDir, $"{projectName}.Core.csproj");
            string hostCsproj = Path.Combine(hostDir, $"{projectName}.{hostSuffix}.csproj");
            await runProcessAsync(projectPath, "dotnet", $"add \"{hostCsproj}\" reference \"{coreCsproj}\"");

            // Add packages to Core
            await runProcessAsync(projectPath, "dotnet", $"add \"{coreCsproj}\" package Microsoft.EntityFrameworkCore");
            await runProcessAsync(projectPath, "dotnet", $"add \"{coreCsproj}\" package Microsoft.EntityFrameworkCore.Sqlite");
            await runProcessAsync(projectPath, "dotnet", $"add \"{coreCsproj}\" package Microsoft.EntityFrameworkCore.InMemory");
            await runProcessAsync(projectPath, "dotnet", $"add \"{coreCsproj}\" package Microsoft.EntityFrameworkCore.Design");
            await runProcessAsync(projectPath, "dotnet", $"add \"{hostCsproj}\" package Microsoft.EntityFrameworkCore.InMemory");
            await runProcessAsync(projectPath, "dotnet", $"add \"{hostCsproj}\" package Microsoft.EntityFrameworkCore.Design");

            // Add ClrinfCS.Core reference if it exists
            string clrinfCorePath = Path.Combine(srcDir, "ClrinfCS.Core", "ClrinfCS.Core.csproj");
            if (File.Exists(clrinfCorePath))
            {
                await runProcessAsync(projectPath, "dotnet", $"add \"{coreCsproj}\" reference \"{clrinfCorePath}\"");
            }
        }
        else if (targetArch == "clean-cqrs")
        {
            string domainDir = Path.Combine(srcDir, $"{projectName}.Domain");
            string appDir = Path.Combine(srcDir, $"{projectName}.Application");
            string persDir = Path.Combine(srcDir, $"{projectName}.Persistence");
            string hostDir = Path.Combine(srcDir, $"{projectName}.{hostSuffix}");

            if (!Directory.Exists(domainDir))
            {
                await runProcessAsync(srcDir, "dotnet", $"new classlib -n {projectName}.Domain -f net10.0");
                createdProjects.Add($"{projectName}.Domain");
                string c1 = Path.Combine(domainDir, "Class1.cs");
                if (File.Exists(c1)) File.Delete(c1);
            }
            if (!Directory.Exists(appDir))
            {
                await runProcessAsync(srcDir, "dotnet", $"new classlib -n {projectName}.Application -f net10.0");
                createdProjects.Add($"{projectName}.Application");
                string c1 = Path.Combine(appDir, "Class1.cs");
                if (File.Exists(c1)) File.Delete(c1);
            }
            if (!Directory.Exists(persDir))
            {
                await runProcessAsync(srcDir, "dotnet", $"new classlib -n {projectName}.Persistence -f net10.0");
                createdProjects.Add($"{projectName}.Persistence");
                string c1 = Path.Combine(persDir, "Class1.cs");
                if (File.Exists(c1)) File.Delete(c1);
            }
            if (!Directory.Exists(hostDir))
            {
                string template = hostSuffix == "Web" ? "web" : (hostSuffix == "Console" ? "console" : "webapi");
                await runProcessAsync(srcDir, "dotnet", $"new {template} -n {projectName}.{hostSuffix} -f net10.0");
                createdProjects.Add($"{projectName}.{hostSuffix}");
            }

            string domainCsproj = Path.Combine(domainDir, $"{projectName}.Domain.csproj");
            string appCsproj = Path.Combine(appDir, $"{projectName}.Application.csproj");
            string persCsproj = Path.Combine(persDir, $"{projectName}.Persistence.csproj");
            string hostCsproj = Path.Combine(hostDir, $"{projectName}.{hostSuffix}.csproj");

            await runProcessAsync(projectPath, "dotnet", $"add \"{appCsproj}\" reference \"{domainCsproj}\"");
            await runProcessAsync(projectPath, "dotnet", $"add \"{persCsproj}\" reference \"{appCsproj}\"");
            await runProcessAsync(projectPath, "dotnet", $"add \"{hostCsproj}\" reference \"{persCsproj}\"");
            await runProcessAsync(projectPath, "dotnet", $"add \"{hostCsproj}\" reference \"{appCsproj}\"");

            await runProcessAsync(projectPath, "dotnet", $"add \"{persCsproj}\" package Microsoft.EntityFrameworkCore");
            await runProcessAsync(projectPath, "dotnet", $"add \"{persCsproj}\" package Microsoft.EntityFrameworkCore.Sqlite");
            await runProcessAsync(projectPath, "dotnet", $"add \"{persCsproj}\" package Microsoft.EntityFrameworkCore.Design");
            await runProcessAsync(projectPath, "dotnet", $"add \"{hostCsproj}\" package Microsoft.EntityFrameworkCore.Design");

            string clrinfCorePath = Path.Combine(srcDir, "ClrinfCS.Core", "ClrinfCS.Core.csproj");
            if (File.Exists(clrinfCorePath))
            {
                await runProcessAsync(projectPath, "dotnet", $"add \"{appCsproj}\" reference \"{clrinfCorePath}\"");
            }
        }

        // Prepare namespace replacements mapping
        var nsReplacements = new Dictionary<string, string>();
        if (targetArch == "layered")
        {
            nsReplacements[$"{projectName}.Entities"] = $"{projectName}.Core.Entities";
            nsReplacements[$"{projectName}.Data"] = $"{projectName}.Core.Data";
            nsReplacements[$"{projectName}.Services"] = $"{projectName}.Core.Services";
        }
        else if (targetArch == "clean-cqrs")
        {
            nsReplacements[$"{projectName}.Entities"] = $"{projectName}.Domain.Entities";
            nsReplacements[$"{projectName}.Core.Entities"] = $"{projectName}.Domain.Entities";
            nsReplacements[$"{projectName}.Data"] = $"{projectName}.Persistence.Contexts";
            nsReplacements[$"{projectName}.Core.Data"] = $"{projectName}.Persistence.Contexts";
            nsReplacements[$"{projectName}.Services"] = $"{projectName}.Application.Services";
            nsReplacements[$"{projectName}.Core.Services"] = $"{projectName}.Application.Services";
        }

        // Move and rewrite files
        foreach (var item in plan)
        {
            if (item.SourceFilePath == item.DestinationFilePath) continue;

            string destDir = Path.GetDirectoryName(item.DestinationFilePath)!;
            if (!Directory.Exists(destDir))
                Directory.CreateDirectory(destDir);

            if (item.SourceFilePath.EndsWith(".cs") && File.Exists(item.SourceFilePath))
            {
                string code = await File.ReadAllTextAsync(item.SourceFilePath);
                var additionalUsings = new List<string>();

                if (item.Category == CodeElementCategory.Controller || item.Category == CodeElementCategory.HostStartup)
                {
                    if (targetArch == "layered")
                    {
                        additionalUsings.Add($"{projectName}.Core.Entities");
                        additionalUsings.Add($"{projectName}.Core.Data");
                        additionalUsings.Add($"{projectName}.Core.Services");
                    }
                    else if (targetArch == "clean-cqrs")
                    {
                        additionalUsings.Add($"{projectName}.Domain.Entities");
                        additionalUsings.Add($"{projectName}.Persistence.Contexts");
                        additionalUsings.Add($"{projectName}.Application.Services");
                    }
                }

                string rewrittenCode = NamespaceRewriter.Rewrite(
                    code,
                    item.NewNamespace,
                    nsReplacements,
                    additionalUsings
                );

                await File.WriteAllTextAsync(item.DestinationFilePath, rewrittenCode);
                File.Delete(item.SourceFilePath);
            }
            else if (File.Exists(item.SourceFilePath))
            {
                if (item.SourceFilePath.EndsWith(".cshtml", StringComparison.OrdinalIgnoreCase))
                {
                    string razorContent = await File.ReadAllTextAsync(item.SourceFilePath);
                    foreach (var kvp in nsReplacements)
                    {
                        razorContent = razorContent.Replace(kvp.Key, kvp.Value);
                    }
                    await File.WriteAllTextAsync(item.DestinationFilePath, razorContent);
                }
                else
                {
                    File.Copy(item.SourceFilePath, item.DestinationFilePath, true);
                }
                File.Delete(item.SourceFilePath);
            }
        }

        // Clean up empty directories from old project if flat
        if (currentArch == "flat")
        {
            string oldProjectDir = Path.Combine(srcDir, projectName);
            if (Directory.Exists(oldProjectDir))
            {
                // Delete remaining .csproj or empty folders
                try { Directory.Delete(oldProjectDir, true); } catch { }
            }
        }

        // Add created projects to .sln
        if (!string.IsNullOrEmpty(slnFile) && File.Exists(slnFile))
        {
            foreach (var proj in createdProjects)
            {
                string projPath = Path.Combine(srcDir, proj, $"{proj}.csproj");
                if (File.Exists(projPath))
                {
                    await runProcessAsync(projectPath, "dotnet", $"sln \"{slnFile}\" add \"{projPath}\"");
                }
            }

            if (currentArch == "flat")
            {
                string oldCsproj = Path.Combine(srcDir, projectName, $"{projectName}.csproj");
                await runProcessAsync(projectPath, "dotnet", $"sln \"{slnFile}\" remove \"{oldCsproj}\"");
            }
        }

        // Update codegen.toml
        string tomlPath = Path.Combine(projectPath, "codegen.toml");
        if (File.Exists(tomlPath))
        {
            string tomlContent = await File.ReadAllTextAsync(tomlPath);
            tomlContent = System.Text.RegularExpressions.Regex.Replace(
                tomlContent,
                @"arch_style\s*=\s*""[^""]*""",
                $"arch_style = \"{targetArch}\""
            );

            if (targetArch == "layered")
            {
                if (!tomlContent.Contains("core ="))
                {
                    tomlContent = tomlContent.Replace(
                        "[backend.layers]",
                        $"[backend.layers]\ncore = \"{projectName}.Core\"\nhost = \"{projectName}.{hostSuffix}\""
                    );
                }
            }
            else if (targetArch == "clean-cqrs")
            {
                if (tomlContent.Contains("[backend.layers]"))
                {
                    string newLayers = $"[backend.layers]\n" +
                        $"domain = \"{projectName}.Domain\"\n" +
                        $"application = \"{projectName}.Application\"\n" +
                        $"persistence = \"{projectName}.Persistence\"\n" +
                        $"core = \"ClrinfCS.Core\"\n" +
                        $"host = \"{projectName}.{hostSuffix}\"\n" +
                        $"api = \"{projectName}.{hostSuffix}\"";

                    tomlContent = System.Text.RegularExpressions.Regex.Replace(
                        tomlContent,
                        @"\[backend\.layers\][\s\S]*?(?=\n\[|\Z)",
                        newLayers
                    );
                }
            }

            await File.WriteAllTextAsync(tomlPath, tomlContent);
        }

        return createdProjects;
    }

    private static async Task<(int ExitCode, string Output, string Error)> runProcessAsync(
        string workingDirectory,
        string fileName,
        string arguments)
    {
        var psi = new ProcessStartInfo
        {
            FileName = fileName,
            Arguments = arguments,
            WorkingDirectory = workingDirectory,
            RedirectStandardOutput = true,
            RedirectStandardError = true,
            UseShellExecute = false,
            CreateNoWindow = true
        };

        using var process = new Process { StartInfo = psi };
        process.Start();

        string output = await process.StandardOutput.ReadToEndAsync();
        string error = await process.StandardError.ReadToEndAsync();
        await process.WaitForExitAsync();

        return (process.ExitCode, output, error);
    }
}
