using System;
using System.Diagnostics;
using System.IO;
using System.Threading.Tasks;

namespace Application.Common.EF;

public static class EfMigrationHelper
{
    public static async Task<(bool Success, string Message)> CreateMigrationAsync(
        string projectPath,
        string persistenceLayer,
        string startupLayer,
        string migrationName,
        string? outputDir = null)
    {
        if (string.IsNullOrEmpty(projectPath) || !System.IO.Directory.Exists(projectPath))
            return (false, "Invalid project path.");

        string persistenceCsproj = findCsproj(projectPath, persistenceLayer);
        string startupCsproj = findCsproj(projectPath, startupLayer);

        if (!string.IsNullOrEmpty(startupCsproj) && System.IO.File.Exists(startupCsproj))
        {
            await runProcessAsync(projectPath, "dotnet", $"restore \"{startupCsproj}\"");
        }

        string outputDirArg = !string.IsNullOrWhiteSpace(outputDir) ? $" --output-dir \"{outputDir}\"" : string.Empty;
        string args = $"ef migrations add {migrationName} --project \"{persistenceCsproj}\" --startup-project \"{startupCsproj}\"{outputDirArg}";

        var result = await runProcessAsync(projectPath, "dotnet", args);
        if (result.ExitCode == 0)
        {
            return (true, $"Migration '{migrationName}' successfully generated in {persistenceLayer}/Migrations.");
        }

        if (result.Error.Contains("dotnet-ef") || result.Output.Contains("dotnet-ef"))
        {
            // Auto install dotnet-ef globally if missing
            await runProcessAsync(projectPath, "dotnet", "tool install --global dotnet-ef --version 10.0.0");
            result = await runProcessAsync(projectPath, "dotnet", args);
            if (result.ExitCode == 0)
            {
                return (true, $"Migration '{migrationName}' successfully generated in {persistenceLayer}/Migrations.");
            }
        }

        string errDetail = !string.IsNullOrWhiteSpace(result.Error) ? result.Error : result.Output;
        if (string.IsNullOrWhiteSpace(errDetail))
        {
            errDetail = $"Process exited with code {result.ExitCode}";
        }

        return (false, $"Failed to generate migration '{migrationName}': {errDetail.Trim()}");
    }

    public static async Task<(bool Success, string Message)> ApplyMigrationAsync(
        string projectPath,
        string persistenceLayer,
        string startupLayer)
    {
        if (string.IsNullOrEmpty(projectPath) || !System.IO.Directory.Exists(projectPath))
            return (false, "Invalid project path.");

        string persistenceCsproj = findCsproj(projectPath, persistenceLayer);
        string startupCsproj = findCsproj(projectPath, startupLayer);

        if (!string.IsNullOrEmpty(startupCsproj) && System.IO.File.Exists(startupCsproj))
        {
            await runProcessAsync(projectPath, "dotnet", $"restore \"{startupCsproj}\"");
        }

        string args = $"ef database update --project \"{persistenceCsproj}\" --startup-project \"{startupCsproj}\"";

        var result = await runProcessAsync(projectPath, "dotnet", args);
        if (result.ExitCode == 0)
        {
            return (true, "Database updated successfully.");
        }

        string errDetail = !string.IsNullOrWhiteSpace(result.Error) ? result.Error : result.Output;
        if (string.IsNullOrWhiteSpace(errDetail))
        {
            errDetail = $"Process exited with code {result.ExitCode}";
        }

        return (false, $"Failed to apply migrations: {errDetail.Trim()}");
    }

    private static string findCsproj(string rootPath, string layerNameOrPath)
    {
        if (string.IsNullOrEmpty(layerNameOrPath))
            return string.Empty;

        if (layerNameOrPath.EndsWith(".csproj", StringComparison.OrdinalIgnoreCase) && System.IO.File.Exists(layerNameOrPath))
            return layerNameOrPath;

        string fullDir = Path.IsPathRooted(layerNameOrPath)
            ? layerNameOrPath
            : Path.Combine(rootPath, layerNameOrPath);

        if (!System.IO.Directory.Exists(fullDir))
        {
            string srcDir = Path.Combine(rootPath, "src", layerNameOrPath);
            if (System.IO.Directory.Exists(srcDir))
                fullDir = srcDir;
        }

        if (!System.IO.Directory.Exists(fullDir) && System.IO.Directory.Exists(rootPath))
        {
            string targetFolderName = Path.GetFileName(layerNameOrPath.TrimEnd(Path.DirectorySeparatorChar, Path.AltDirectorySeparatorChar));
            var matchingDirs = System.IO.Directory.GetDirectories(rootPath, targetFolderName, SearchOption.AllDirectories);
            if (matchingDirs.Length > 0)
                fullDir = matchingDirs[0];
        }

        if (System.IO.Directory.Exists(fullDir))
        {
            var csprojFiles = System.IO.Directory.GetFiles(fullDir, "*.csproj");
            if (csprojFiles.Length > 0)
                return csprojFiles[0];
        }

        return Path.Combine(layerNameOrPath, $"{Path.GetFileName(layerNameOrPath)}.csproj");
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
