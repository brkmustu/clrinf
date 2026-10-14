using System;
using System.Text;
using ClrinfCS;
using ClrinfCS.Commands.Migrate;
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
});

bool isSilentMode = args.Any(a => a.Equals("--json", StringComparison.OrdinalIgnoreCase));
if (!isSilentMode)
{
    var version = typeof(Program).Assembly.GetName().Version?.ToString(3) ?? "1.0.0";
    AnsiConsole.Write(new FigletText("clrinfcs").LeftJustified().Color(Color.Green));
    AnsiConsole.MarkupLine($"[grey]Version: {version}[/]");
    AnsiConsole.MarkupLine("[grey]Specialized .NET Roslyn AST architecture linter & migration worker[/]");
    AnsiConsole.WriteLine();
}

return app.Run(args);
