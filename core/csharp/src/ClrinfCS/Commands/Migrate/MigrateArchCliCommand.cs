using System;
using System.ComponentModel;
using System.IO;
using System.Threading.Tasks;
using ClrinfCS.Migration;
using Spectre.Console;
using Spectre.Console.Cli;

namespace ClrinfCS.Commands.Migrate;

public class MigrateArchCliCommand : AsyncCommand<MigrateArchCliCommand.Settings>
{
    public class Settings : CommandSettings
    {
        [CommandOption("-t|--to <TARGET_ARCH>")]
        [Description("Hedef mimari profili: layered veya clean-cqrs (Zorunlu)")]
        public string TargetArch { get; set; } = string.Empty;

        [CommandOption("-p|--project <PROJECT_PATH>")]
        [Description("Dönüştürülecek proje dizini (Varsayılan: mevcut dizin)")]
        public string? ProjectPath { get; set; }

        [CommandOption("--dry-run")]
        [Description("Değişiklikleri diske yazmadan yapılacak işlemleri ve dosya taşımalarını listeler")]
        public bool DryRun { get; set; }
    }

    public override async Task<int> ExecuteAsync(CommandContext context, Settings settings)
    {
        if (string.IsNullOrWhiteSpace(settings.TargetArch))
        {
            AnsiConsole.MarkupLine("[red]HATA: Lütfen hedef mimariyi belirtin: --to layered veya --to clean-cqrs[/]");
            return 1;
        }

        string currentDir = string.IsNullOrWhiteSpace(settings.ProjectPath)
            ? Environment.CurrentDirectory
            : Path.GetFullPath(settings.ProjectPath);

        AnsiConsole.MarkupLine($"[bold blue]=== ClrinfCS Roslyn AST Mimari Dönüşüm Motoru ===[/]");
        AnsiConsole.MarkupLine($"[grey]Proje Dizini:[/] {currentDir}");
        AnsiConsole.MarkupLine($"[grey]Hedef Mimari:[/] [yellow]{settings.TargetArch}[/]");
        if (settings.DryRun)
        {
            AnsiConsole.MarkupLine("[magenta bold]MOD: Dry-run (Simülasyon - dosyalarda değişiklik yapılmayacak)[/]");
        }
        AnsiConsole.WriteLine();

        var result = await ProjectMigrationEngine.MigrateAsync(
            currentDir,
            settings.TargetArch,
            settings.DryRun
        );

        if (!result.Success)
        {
            AnsiConsole.MarkupLine($"[red]✖ {result.Message.EscapeMarkup()}[/]");
            return 1;
        }

        if (result.PlanItems.Count > 0)
        {
            var table = new Table();
            table.Border(TableBorder.Rounded);
            table.AddColumn("Kategori");
            table.AddColumn("Kaynak Dosya");
            table.AddColumn("Hedef Dosya");
            table.AddColumn("Yeni Ad Alanı");

            foreach (var item in result.PlanItems)
            {
                table.AddRow(
                    $"[cyan]{item.Category}[/]",
                    $"[grey]{Path.GetRelativePath(currentDir, item.SourceFilePath)}[/]",
                    $"[green]{Path.GetRelativePath(currentDir, item.DestinationFilePath)}[/]",
                    $"[yellow]{item.NewNamespace ?? "-"}[/]"
                );
            }

            AnsiConsole.Write(table);
            AnsiConsole.WriteLine();
        }

        if (result.CreatedProjects.Count > 0)
        {
            AnsiConsole.MarkupLine("[bold green]Oluşturulan Projeler:[/]");
            foreach (var p in result.CreatedProjects)
            {
                AnsiConsole.MarkupLine($"  • [bold]{p}[/]");
            }
            AnsiConsole.WriteLine();
        }

        AnsiConsole.MarkupLine($"[green bold]✔ {result.Message.EscapeMarkup()}[/]");
        return 0;
    }
}
