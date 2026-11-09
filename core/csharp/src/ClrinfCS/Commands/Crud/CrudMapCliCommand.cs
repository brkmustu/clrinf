using System;
using System.ComponentModel;
using System.IO;
using System.Text.Json;
using System.Threading.Tasks;
using Spectre.Console;
using Spectre.Console.Cli;

namespace ClrinfCS.Commands.Crud;

public class CrudMapCliCommand : AsyncCommand<CrudMapCliCommand.Settings>
{
    public class Settings : CommandSettings
    {
        [CommandArgument(0, "[PROJECT_PATH]")]
        [Description("Project or source directory (default: current directory)")]
        public string? ProjectPath { get; set; }

        [CommandOption("--json")]
        [Description("Emit machine-readable JSON for agents")]
        public bool Json { get; set; }
    }

    public override Task<int> ExecuteAsync(CommandContext context, Settings settings)
    {
        var target = string.IsNullOrWhiteSpace(settings.ProjectPath)
            ? Environment.CurrentDirectory
            : Path.GetFullPath(settings.ProjectPath);

        if (!Directory.Exists(target))
        {
            AnsiConsole.MarkupLine($"[red]✖ Directory not found: {target.EscapeMarkup()}[/]");
            return Task.FromResult(1);
        }

        var map = CrudSourceScanner.Scan(target);
        if (settings.Json)
        {
            Console.WriteLine(JsonSerializer.Serialize(map, JsonOptions));
            return Task.FromResult(0);
        }

        if (map.Entities.Count == 0)
        {
            AnsiConsole.MarkupLine("[yellow]No [GenerateCrudEndpoints] entity found.[/]");
            return Task.FromResult(0);
        }

        var table = new Table()
            .Border(TableBorder.Rounded)
            .Title("[bold]CRUD map[/]")
            .AddColumn("Entity")
            .AddColumn("Dispatcher")
            .AddColumn("Operations")
            .AddColumn("Hooks");

        foreach (var entity in map.Entities)
        {
            var operations = string.Join(", ",
                entity.Operations.Select(static kv => $"{kv.Key}: {kv.Value}"));
            var hooks = entity.Hooks.Count == 0
                ? "-"
                : string.Join(", ", entity.Hooks.Select(static h => $"{h.Rule}({h.Operation})"));
            table.AddRow(
                $"[cyan]{entity.Name}[/]\n[grey]{entity.Namespace}[/]",
                entity.Dispatcher,
                operations,
                hooks);
        }

        AnsiConsole.Write(table);
        return Task.FromResult(0);
    }

    private static readonly JsonSerializerOptions JsonOptions = new()
    {
        WriteIndented = true,
        PropertyNamingPolicy = JsonNamingPolicy.CamelCase,
    };
}
