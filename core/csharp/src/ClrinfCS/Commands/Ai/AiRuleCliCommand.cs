using System;
using System.ComponentModel;
using System.IO;
using System.Threading.Tasks;
using ClrinfCS.Ai;
using Spectre.Console;
using Spectre.Console.Cli;

namespace ClrinfCS.Commands.Ai;

public class AiRuleCliCommand : AsyncCommand<AiRuleCliCommand.Settings>
{
    public class Settings : CommandSettings
    {
        [CommandArgument(0, "<RULE_DESCRIPTION>")]
        [Description("Doğal dilde kural tanımı (örn. 'İndirim oranı 50'den büyük olamaz')")]
        public string RuleDescription { get; set; } = string.Empty;

        [CommandOption("-m|--module <MODULE>")]
        [Description("Kuralın ekleneceği modül veya Entity adı (örn. Product)")]
        public string Module { get; set; } = string.Empty;

        [CommandOption("-n|--name <NAME>")]
        [Description("Kural sınıfı adı (örn. MaxDiscountLimitRule). Belirtilmezse otomatik türetilir")]
        public string? Name { get; set; }

        [CommandOption("-c|--command <COMMAND>")]
        [Description("Hedef komut sınıfı adı (örn. CreateProductCommand)")]
        public string? Command { get; set; }

        [CommandOption("-p|--project <PROJECT_PATH>")]
        [Description("Proje dizini (Varsayılan: mevcut dizin)")]
        public string? ProjectPath { get; set; }

        [CommandOption("--endpoint <LLM_ENDPOINT>")]
        [Description("Ollama veya yerel LLM uç noktası (örn. http://localhost:11434)")]
        public string? Endpoint { get; set; }

        [CommandOption("--model <MODEL>")]
        [Description("Kullanılacak model adı (Varsayılan: deepseek-coder)")]
        public string? Model { get; set; }
    }

    public override async Task<int> ExecuteAsync(CommandContext context, Settings settings)
    {
        if (string.IsNullOrWhiteSpace(settings.Module))
        {
            AnsiConsole.MarkupLine("[red]HATA: Lütfen modül adını belirtin: -m <Module>[/]");
            return 1;
        }

        string ruleName = settings.Name ?? DeriveRuleName(settings.RuleDescription, settings.Module);
        string commandName = settings.Command ?? $"Create{settings.Module}Command";
        string projectPath = string.IsNullOrWhiteSpace(settings.ProjectPath) ? Environment.CurrentDirectory : Path.GetFullPath(settings.ProjectPath);

        AnsiConsole.MarkupLine("[bold blue]=== ClrinfCS İzole AI Kural & Test Sentezi ===[/]");
        AnsiConsole.MarkupLine($"[grey]Modül:[/] [cyan]{settings.Module}[/]");
        AnsiConsole.MarkupLine($"[grey]Komut:[/] [cyan]{commandName}[/]");
        AnsiConsole.MarkupLine($"[grey]Kural Adı:[/] [yellow]{ruleName}[/]");
        AnsiConsole.MarkupLine($"[grey]Tanım:[/] {settings.RuleDescription.EscapeMarkup()}");
        AnsiConsole.WriteLine();

        var result = await AiRuleGenerator.GenerateRuleAsync(
            projectPath,
            settings.Module,
            ruleName,
            commandName,
            settings.RuleDescription,
            settings.Endpoint,
            settings.Model
        );

        if (!result.Success)
        {
            AnsiConsole.MarkupLine($"[red]✖ {result.Message.EscapeMarkup()}[/]");
            return 1;
        }

        AnsiConsole.MarkupLine("[bold green]✔ Kural ve Test Dosyaları Başarıyla Üretildi![/]");
        AnsiConsole.MarkupLine($"[grey]Kural Dosyası:[/] [green]{result.RuleFilePath}[/]");
        AnsiConsole.MarkupLine($"[grey]Test Dosyası:[/] [green]{result.TestFilePath}[/]");
        AnsiConsole.MarkupLine("[bold grey]Not: Mevcut hiçbir dosyada değişiklik yapılmadı (Sıfır regresyon riski).[/]");

        return 0;
    }

    private static string DeriveRuleName(string description, string module)
    {
        // Basit türetme mantığı
        return $"{module}BusinessRule";
    }
}
