using System;
using System.Text;
using ClrinfCS;
using ClrinfCS.Commands.Migrate;
using ClrinfCS.Commands.Migration;
using ClrinfCS.IoC.SpectreConsoleCli;
using Microsoft.Extensions.DependencyInjection;
using Spectre.Console;
using Spectre.Console.Cli;

#region Console Configuration

Console.OutputEncoding = Encoding.UTF8;
Console.InputEncoding = Encoding.UTF8;

#endregion

#region IoC

IServiceCollection services = new ServiceCollection();
services.AddClrinfCSServices();
TypeRegistrar registrar = new(services);

#endregion

CommandApp app = new(registrar);
app.Configure(config =>
{
    config.SetApplicationName("clrinfcs");

    #region Migration Branch
    config.AddBranch(
        name: "migration",
        action: config =>
        {
            config.SetDescription("EF Core veritabanı migrasyonlarını yönetir");

            config
                .AddCommand<AddMigrationCliCommand>(name: "add")
                .WithDescription("Yeni bir EF Core migrasyonu oluşturur (proje profiline uygun katmanda izole eder)")
                .WithExample(new[] { "migration", "add", "AddProductsTable" });

            config
                .AddCommand<ApplyMigrationCliCommand>(name: "apply")
                .WithDescription("Bekleyen EF Core migrasyonlarını hedef veritabanına uygular")
                .WithExample(new[] { "migration", "apply" });
        }
    );
    #endregion

    #region Migrate Command (Roslyn AST Architecture Transition)
    config
        .AddCommand<MigrateArchCliCommand>(name: "migrate")
        .WithDescription("Mevcut projeyi Roslyn AST ile bir mimari profilden diğerine dönüştürür (flat -> layered -> clean-cqrs)")
        .WithExample(new[] { "migrate", "--to", "layered" })
        .WithExample(new[] { "migrate", "--to", "clean-cqrs" })
        .WithExample(new[] { "migrate", "--to", "layered", "--dry-run" });
    #endregion

    #region Lint Command (Roslyn AST Architecture Checker)
    config
        .AddCommand<ClrinfCS.Commands.Lint.LintCliCommand>(name: "lint")
        .WithDescription("Roslyn AST ile mimari kuralları ve katman sınırlarını deterministik olarak denetler")
        .WithExample(new[] { "lint" })
        .WithExample(new[] { "lint", "src/MyProject" })
        .WithExample(new[] { "lint", "--strict" });
    #endregion

    #region MCP Server Command (Model Context Protocol for AI Agents)
    config
        .AddCommand<ClrinfCS.Commands.Mcp.McpCliCommand>(name: "mcp")
        .WithDescription("IDE ve AI ajanları (Cursor, Antigravity, Claude Desktop) için Model Context Protocol (MCP) JSON-RPC sunucusunu başlatır")
        .WithExample(new[] { "mcp" });
    #endregion

    #region AI Branch (Rule and Test Synthesis)
    config.AddBranch(
        name: "ai",
        action: config =>
        {
            config.SetDescription("Doğal dildeki iş kurallarını ve birim testleri izole C# sınıfları olarak sentezler");

            config
                .AddCommand<ClrinfCS.Commands.Ai.AiRuleCliCommand>(name: "rule")
                .WithDescription("Mevcut hiçbir dosyayı bozmadan, izole bir IBusinessRule<T> sınıfı ve xUnit testlerini üretir")
                .WithExample(new[] { "ai", "rule", "\"İndirim oranı 50'den büyük olamaz\"", "-m", "Product", "-n", "MaxDiscountLimitRule", "-c", "CreateProductCommand" });
        }
    );
    #endregion
});

bool isSilentMode = args.Any(a => a.Equals("--json", StringComparison.OrdinalIgnoreCase) || a.Equals("mcp", StringComparison.OrdinalIgnoreCase));
if (!isSilentMode)
{
    var version = typeof(Program).Assembly.GetName().Version?.ToString(3) ?? "1.0.0";
    AnsiConsole.Write(new FigletText("clrinfcs").LeftJustified().Color(Color.Green));
    AnsiConsole.MarkupLine($"[grey]Version: {version}[/]");
    AnsiConsole.MarkupLine("[grey]Specialized .NET Roslyn AST architecture linter & migration worker[/]");
    AnsiConsole.WriteLine();
}

return app.Run(args);
