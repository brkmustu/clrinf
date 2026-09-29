using System;
using System.ComponentModel;
using System.IO;
using System.Text.Json;
using System.Threading.Tasks;
using ClrinfCS.Lint;
using Spectre.Console;
using Spectre.Console.Cli;

namespace ClrinfCS.Commands.Lint;

public class LintCliCommand : AsyncCommand<LintCliCommand.Settings>
{
    public class Settings : CommandSettings
    {
        [CommandArgument(0, "[PROJECT_PATH]")]
        [Description("Analiz edilecek proje veya kaynak dizini (Varsayılan: mevcut dizin)")]
        public string? ProjectPath { get; set; }

        [CommandOption("--strict")]
        [Description("Uyarıları (Warning) da hata (Error) sayarak sıfır tolerans uygular")]
        public bool Strict { get; set; }

        [CommandOption("--json")]
        [Description("Sonuçları JSON formatında döner (CI/CD veya MCP entegrasyonu için)")]
        public bool Json { get; set; }
    }

    public override async Task<int> ExecuteAsync(CommandContext context, Settings settings)
    {
        string targetDir = string.IsNullOrWhiteSpace(settings.ProjectPath)
            ? Environment.CurrentDirectory
            : Path.GetFullPath(settings.ProjectPath);

        if (!settings.Json)
        {
            AnsiConsole.MarkupLine("[bold blue]=== ClrinfCS Roslyn AST Mimari Linter ===[/]");
            AnsiConsole.MarkupLine($"[grey]Hedef Dizin:[/] {targetDir}");
            AnsiConsole.WriteLine();
        }

        var violations = await ArchAnalyzer.AnalyzeDirectoryAsync(targetDir);

        if (settings.Json)
        {
            var json = JsonSerializer.Serialize(violations, new JsonSerializerOptions { WriteIndented = true });
            Console.WriteLine(json);
            bool hasFailures = settings.Strict
                ? violations.Count > 0
                : violations.Any(v => v.Severity == "Error");
            return hasFailures ? 1 : 0;
        }

        if (violations.Count == 0)
        {
            AnsiConsole.MarkupLine("[bold green]✔ Tebrikler! Hiçbir mimari veya kural ihlali bulunamadı.[/]");
            return 0;
        }

        var table = new Table()
            .Border(TableBorder.Rounded)
            .Title("[bold red]Tespit Edilen Mimari İhlaller[/]")
            .AddColumn(new TableColumn("[bold]Kural[/]").Centered())
            .AddColumn(new TableColumn("[bold]Önem[/]").Centered())
            .AddColumn(new TableColumn("[bold]Konum[/]"))
            .AddColumn(new TableColumn("[bold]Açıklama[/]"));

        foreach (var v in violations)
        {
            string severityColor = v.Severity switch
            {
                "Error" => "red bold",
                "Warning" => "yellow",
                _ => "grey"
            };

            string relFile = Path.GetRelativePath(targetDir, v.FilePath);
            table.AddRow(
                $"[cyan]{v.RuleId}[/]",
                $"[{severityColor}]{v.Severity}[/]",
                $"[dim]{relFile}:{v.LineNumber}[/]",
                v.Message.EscapeMarkup()
            );
        }

        AnsiConsole.Write(table);
        AnsiConsole.WriteLine();

        int errorCount = violations.Count(v => v.Severity == "Error");
        int warningCount = violations.Count(v => v.Severity == "Warning");
        int infoCount = violations.Count(v => v.Severity == "Info");

        AnsiConsole.MarkupLine($"Toplam: [red]{errorCount} Hata[/], [yellow]{warningCount} Uyarı[/], [grey]{infoCount} Bilgi[/]");

        bool shouldFail = errorCount > 0 || (settings.Strict && warningCount > 0);
        return shouldFail ? 1 : 0;
    }
}
