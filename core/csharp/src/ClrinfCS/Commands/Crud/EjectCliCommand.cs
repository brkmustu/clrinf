using System;
using System.Collections.Generic;
using System.IO;
using System.Linq;
using System.Text.Json;
using System.Threading.Tasks;
using ClrinfCS.Generators;
using Microsoft.Build.Locator;
using Microsoft.CodeAnalysis;
using Microsoft.CodeAnalysis.CSharp;
using Microsoft.CodeAnalysis.CSharp.Syntax;
using Microsoft.CodeAnalysis.MSBuild;
using Spectre.Console;
using Spectre.Console.Cli;

namespace ClrinfCS.Commands.Crud;

public class EjectCliCommand : AsyncCommand<EjectCliCommand.Settings>
{
    public class Settings : CommandSettings
    {
        [CommandArgument(0, "<ENTITY>")]
        public string Entity { get; set; } = string.Empty;

        [CommandOption("-o|--operation <OPERATION>")]
        public string Operation { get; set; } = string.Empty;

        [CommandOption("-p|--project <PATH>")]
        public string? ProjectPath { get; set; }

        [CommandOption("--json")]
        public bool Json { get; set; }

        [CommandOption("--force")]
        public bool Force { get; set; }
    }

    public override async Task<int> ExecuteAsync(CommandContext context, Settings settings)
    {
        if (string.IsNullOrWhiteSpace(settings.Entity))
        {
            return Fail(settings.Json, "Entity is required.");
        }

        var operation = NormalizeOperation(settings.Operation);
        if (operation is null)
        {
            return Fail(settings.Json, "Operation must be Create, Update, Delete, GetById or GetList.");
        }

        var projectFile = FindProjectFile(settings.ProjectPath);
        if (projectFile is null)
        {
            return Fail(settings.Json, "No .csproj found. Pass --project <path>.");
        }

        try
        {
            var result = await EjectAsync(projectFile, settings.Entity, operation, settings.Force);
            if (settings.Json)
            {
                Console.WriteLine(JsonSerializer.Serialize(result, JsonOptions));
            }
            else
            {
                AnsiConsole.MarkupLine($"[green]✔ {settings.Entity}.{operation} ejected.[/]");
                AnsiConsole.MarkupLine($"  Entity attribute updated: [grey]{result.EntityFile}[/]");
                foreach (var file in result.Files) AnsiConsole.MarkupLine($"  Physical file: [grey]{file}[/]");
            }

            return 0;
        }
        catch (Exception ex)
        {
            return Fail(settings.Json, ex.Message);
        }
    }

    private sealed record EjectResult(
        string Entity,
        string Operation,
        string EntityFile,
        IReadOnlyList<string> Files);

    private static async Task<EjectResult> EjectAsync(string projectFile, string entityName,
        string operation, bool force)
    {
        RegisterMsBuild();
        using var workspace = MSBuildWorkspace.Create(new Dictionary<string, string>
        {
            ["NuGetAudit"] = "false"
        });
        var failures = new List<string>();
        workspace.RegisterWorkspaceFailedHandler(e =>
        {
            if (e.Diagnostic.Kind == WorkspaceDiagnosticKind.Failure) failures.Add(e.Diagnostic.Message);
        });

        var project = await workspace.OpenProjectAsync(projectFile);
        var compilation = await project.GetCompilationAsync();
        if (compilation is null || !compilation.SyntaxTrees.Any())
        {
            var details = string.Join(" | ", failures.Concat(workspace.Diagnostics.Select(static d => d.ToString())).Take(8));
            throw new InvalidOperationException(
                $"MSBuildWorkspace failed to compile project '{project.Name}': {details}");
        }

        // MSBuildWorkspace may already contain source-generated documents from the
        // analyzer reference. Remove them so Smart Override does not mistake the
        // generator's own output for a manual override.
        var generatedTrees = compilation.SyntaxTrees
            .Where(static t => !IsUserSource(t.FilePath))
            .ToArray();
        if (generatedTrees.Length > 0)
        {
            compilation = compilation.RemoveSyntaxTrees(generatedTrees);
        }

        var parseOptions = compilation.SyntaxTrees.FirstOrDefault()?.Options as CSharpParseOptions
            ?? project.ParseOptions as CSharpParseOptions
            ?? CSharpParseOptions.Default;
        GeneratorDriver driver = CSharpGeneratorDriver.Create(
            new[] { new CrudIncrementalGenerator().AsSourceGenerator() },
            parseOptions: parseOptions);
        driver = driver.RunGeneratorsAndUpdateCompilation(compilation, out _, out var diagnostics);
        var generatorResult = driver.GetRunResult();
        var generatorErrors = generatorResult.Diagnostics
            .Where(static d => d.Severity == DiagnosticSeverity.Error).ToArray();
        if (generatorErrors.Length > 0)
        {
            throw new InvalidOperationException(
                "The CRUD generator reported errors: " + string.Join(" | ", generatorErrors.Select(static d => d.ToString())));
        }

        var entitySymbol = compilation.GetSymbolsWithName(
                name => string.Equals(name, entityName, StringComparison.Ordinal), SymbolFilter.Type)
            .OfType<INamedTypeSymbol>()
            .FirstOrDefault(static s => s.GetAttributes().Any(static a =>
                a.AttributeClass?.ToDisplayString() == "ClrinfCS.Core.GenerateCrudEndpointsAttribute"))
            ?? throw new InvalidOperationException($"Entity '{entityName}' with [GenerateCrudEndpoints] was not found.");

        var syntaxReference = entitySymbol.DeclaringSyntaxReferences.FirstOrDefault()
            ?? throw new InvalidOperationException($"Entity '{entityName}' has no declaring syntax.");
        var tree = syntaxReference.SyntaxTree;
        var root = await tree.GetRootAsync();
        var classDeclaration = root.DescendantNodes().OfType<ClassDeclarationSyntax>()
            .FirstOrDefault(c => c.Identifier.Text == entityName)
            ?? throw new InvalidOperationException($"Entity '{entityName}' declaration was not found.");
        var attribute = classDeclaration.AttributeLists.SelectMany(static l => l.Attributes)
            .FirstOrDefault(static a => a.Name.ToString().Contains("GenerateCrudEndpoints"))
            ?? throw new InvalidOperationException($"[GenerateCrudEndpoints] was not found on {entityName}.");

        var projectDir = Path.GetDirectoryName(project.FilePath!)!;
        var outputDir = Path.Combine(projectDir, "Features", entityName);
        Directory.CreateDirectory(outputDir);

        var handlerTree = FindTree(generatorResult, entityName, operation, "Handler");
        var endpointTree = FindTree(generatorResult, entityName, operation, "Endpoint");
        var files = new[]
        {
            WriteEjected(handlerTree, Path.Combine(outputDir, $"{operation}{entityName}Handler.cs"), force),
            WriteEjected(endpointTree, Path.Combine(outputDir, $"{operation}{entityName}Endpoint.cs"), force),
        };

        // Only change the entity attribute after both physical files are on disk.
        var updatedAttribute = ExcludeOperation(attribute, operation);
        var updatedRoot = root.ReplaceNode(attribute, updatedAttribute);
        if (tree.FilePath is null)
        {
            throw new InvalidOperationException("Entity file path is unknown.");
        }

        await File.WriteAllTextAsync(tree.FilePath, updatedRoot.ToFullString());

        return new EjectResult(entityName, operation,
            Path.GetRelativePath(projectDir, tree.FilePath), files);
    }

    private static bool IsUserSource(string filePath)
    {
        var normalized = filePath.Replace('\\', '/');
        return !normalized.Contains("ClrinfCS.Generators", StringComparison.OrdinalIgnoreCase) &&
               !normalized.Contains("CrudIncrementalGenerator", StringComparison.OrdinalIgnoreCase);
    }

    private static SyntaxTree FindTree(GeneratorDriverRunResult result, string entity, string operation, string kind)
    {
        var suffix = $".{entity}.Crud.{operation}.{kind}.g.cs";
        return result.GeneratedTrees.FirstOrDefault(t =>
                   t.FilePath.EndsWith(suffix, StringComparison.Ordinal))
               ?? throw new InvalidOperationException(
                   $"Generator did not produce {kind} output for {entity}.{operation}. Generated: {string.Join(", ", result.GeneratedTrees.Select(static t => t.FilePath))}. Diagnostics: {string.Join(" | ", result.Diagnostics.Select(static d => d.ToString()))}. Results: {result.Results.Length}, Exception: {result.Results[0].Exception}");
    }

    private static string WriteEjected(SyntaxTree tree, string path, bool force)
    {
        if (File.Exists(path) && !force)
        {
            throw new InvalidOperationException($"Refusing to overwrite {path}. Use --force to replace it.");
        }

        var text = tree.GetText().ToString();
        var lines = text.Split('\n')
            .Where(static line => !line.Contains("<auto-generated", StringComparison.Ordinal));
        var content = "// Ejected by clrinfcs eject. Smart Override prevents the generator from re-emitting this operation.\n" +
                      string.Join('\n', lines);
        File.WriteAllText(path, content);
        return path;
    }

    private static AttributeSyntax ExcludeOperation(AttributeSyntax attribute, string operation)
    {
        const string allExpression = "CrudOperations.All & ~CrudOperations.";
        if (attribute.ArgumentList is null)
        {
            return attribute.WithArgumentList(SyntaxFactory.AttributeArgumentList(
                SyntaxFactory.SingletonSeparatedList(
                    OperationsArgument($"{allExpression}{operation}"))));
        }

        var existing = attribute.ArgumentList.Arguments
            .FirstOrDefault(static a => a.NameEquals?.Name.Identifier.Text == "Operations");
        if (existing is not null)
        {
            var expression = existing.Expression.ToString();
            if (expression.Contains($"~CrudOperations.{operation}", StringComparison.Ordinal))
            {
                return attribute;
            }

            var argument = existing.WithExpression(
                SyntaxFactory.ParseExpression($"({expression}) & ~CrudOperations.{operation}"));
            return attribute.WithArgumentList(
                attribute.ArgumentList.WithArguments(attribute.ArgumentList.Arguments.Replace(existing, argument)));
        }

        var added = OperationsArgument($"{allExpression}{operation}");
        return attribute.WithArgumentList(
            attribute.ArgumentList.WithArguments(attribute.ArgumentList.Arguments.Add(added)));
    }

    private static AttributeArgumentSyntax OperationsArgument(string expression) =>
        SyntaxFactory.AttributeArgument(SyntaxFactory.ParseExpression(expression))
            .WithNameEquals(SyntaxFactory.NameEquals("Operations"));

    private static void RegisterMsBuild()
    {
        if (!MSBuildLocator.IsRegistered)
        {
            MSBuildLocator.RegisterDefaults();
        }
    }

    private static string? NormalizeOperation(string value) => value.Trim().ToLowerInvariant() switch
    {
        "create" => "Create",
        "update" => "Update",
        "delete" => "Delete",
        "getbyid" or "get-by-id" => "GetById",
        "getlist" or "get-list" => "GetList",
        _ => null,
    };

    private static string? FindProjectFile(string? path)
    {
        var target = string.IsNullOrWhiteSpace(path) ? Environment.CurrentDirectory : Path.GetFullPath(path);
        if (File.Exists(target) && target.EndsWith(".csproj", StringComparison.OrdinalIgnoreCase))
        {
            return target;
        }

        if (!Directory.Exists(target)) return null;
        return Directory.EnumerateFiles(target, "*.csproj", SearchOption.AllDirectories)
            .Where(static f => !f.Replace('\\', '/').Contains("/bin/") &&
                               !f.Replace('\\', '/').Contains("/obj/") &&
                               !f.Replace('\\', '/').Contains("/tests/"))
            .OrderBy(static f => f.Length)
            .FirstOrDefault();
    }

    private static int Fail(bool json, string message)
    {
        if (json)
        {
            Console.WriteLine(JsonSerializer.Serialize(new { error = message }, JsonOptions));
        }
        else
        {
            AnsiConsole.MarkupLine($"[red]✖ {message.EscapeMarkup()}[/]");
        }

        return 1;
    }

    private static readonly JsonSerializerOptions JsonOptions = new()
    {
        WriteIndented = true,
        PropertyNamingPolicy = JsonNamingPolicy.CamelCase,
    };
}
