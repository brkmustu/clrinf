using System;
using System.ComponentModel;
using System.IO;
using System.Threading.Tasks;
using Application.Common.Configuration;
using Application.Common.EF;
using Spectre.Console;
using Spectre.Console.Cli;

namespace ClrinfCS.Commands.Migration;

public class AddMigrationCliCommand : AsyncCommand<AddMigrationCliCommand.Settings>
{
    public class Settings : CommandSettings
    {
        [CommandArgument(0, "<MigrationName>")]
        [Description("Eklenecek EF Core migrasyonunun adı (örnek: AddProductsTable)")]
        public string MigrationName { get; set; } = string.Empty;

        [CommandOption("-p|--project <ProjectPath>")]
        [Description("Hedef proje kök dizini (Varsayılan: mevcut dizin)")]
        public string? ProjectPath { get; set; }
    }

    public override async Task<int> ExecuteAsync(CommandContext context, Settings settings)
    {
        string currentDir = string.IsNullOrWhiteSpace(settings.ProjectPath)
            ? Environment.CurrentDirectory
            : Path.GetFullPath(settings.ProjectPath);

        string configPath = Path.Combine(currentDir, "codegen.toml");
        if (!File.Exists(configPath))
            configPath = Path.Combine(currentDir, "codegen.yaml");
        if (!File.Exists(configPath))
            configPath = Path.Combine(currentDir, "codegen.yml");

        string persistenceLayer = "Persistence";
        string startupLayer = "API";
        string? outputDir = null;

        if (File.Exists(configPath))
        {
            var config = ConfigLoader.Load(configPath);
            if (config?.Backend != null)
            {
                if (config.Backend.IsFlat)
                {
                    string hostProj = !string.IsNullOrEmpty(config.Backend.Layers?.Host)
                        ? config.Backend.Layers.Host
                        : config.Backend.Namespace;
                    persistenceLayer = hostProj;
                    startupLayer = hostProj;
                    outputDir = "Data/Migrations";
                }
                else if (config.Backend.IsLayered)
                {
                    string coreProj = !string.IsNullOrEmpty(config.Backend.Layers?.Core)
                        ? config.Backend.Layers.Core
                        : $"{config.Backend.Namespace}.Core";
                    string hostProj = !string.IsNullOrEmpty(config.Backend.Layers?.Host)
                        ? config.Backend.Layers.Host
                        : (!string.IsNullOrEmpty(config.Backend.Layers?.Api)
                            ? config.Backend.Layers.Api
                            : $"{config.Backend.Namespace}.Web");
                    persistenceLayer = coreProj;
                    startupLayer = hostProj;
                    outputDir = "Migrations";
                }
                else
                {
                    if (!string.IsNullOrEmpty(config.Backend.Layers?.Persistence))
                        persistenceLayer = config.Backend.Layers.Persistence;
                    if (!string.IsNullOrEmpty(config.Backend.Layers?.Host))
                        startupLayer = config.Backend.Layers.Host;
                    else if (!string.IsNullOrEmpty(config.Backend.Layers?.Api))
                        startupLayer = config.Backend.Layers.Api;
                }
            }
        }

        AnsiConsole.MarkupLine($"[blue]EF Core Migrasyon oluşturuluyor:[/] [yellow]{settings.MigrationName}[/]");
        AnsiConsole.MarkupLine($"[grey]Persistence:[/] {persistenceLayer} | [grey]Startup:[/] {startupLayer}");

        var result = await EfMigrationHelper.CreateMigrationAsync(
            currentDir,
            persistenceLayer,
            startupLayer,
            settings.MigrationName,
            outputDir
        );

        if (result.Success)
        {
            AnsiConsole.MarkupLine($"[green]✔ {result.Message}[/]");
            return 0;
        }

        AnsiConsole.MarkupLine($"[red]✖ {result.Message}[/]");
        return 1;
    }
}
