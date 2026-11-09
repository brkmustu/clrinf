using System;
using System.Collections.Generic;
using System.Collections.Immutable;
using System.IO;
using System.Linq;
using ClrinfCS.Generators;
using Microsoft.CodeAnalysis;
using Microsoft.CodeAnalysis.CSharp;
using Microsoft.CodeAnalysis.Diagnostics;

namespace ClrinfCS.Generators.Tests;

internal static class GeneratorTestHarness
{
    private static readonly ImmutableArray<MetadataReference> References = BuildReferences();

    public static (Compilation Input, Compilation Output, GeneratorDriverRunResult Result, ImmutableArray<Diagnostic> GeneratorDiagnostics, GeneratorDriver Driver)
        Run(string source, string? dispatcherProperty = null, string? securedProperty = null,
            string? additionalSources = null, bool trackSteps = false, params string[] extraTrees)
    {
        var trees = new List<SyntaxTree>
        {
            CSharpSyntaxTree.ParseText(source, new CSharpParseOptions(LanguageVersion.Preview))
        };
        if (!string.IsNullOrEmpty(additionalSources))
        {
            trees.Add(CSharpSyntaxTree.ParseText(additionalSources, new CSharpParseOptions(LanguageVersion.Preview)));
        }
        foreach (var extra in extraTrees)
        {
            trees.Add(CSharpSyntaxTree.ParseText(extra, new CSharpParseOptions(LanguageVersion.Preview)));
        }

        var compilation = CSharpCompilation.Create(
            "GeneratorTestAssembly",
            trees,
            References,
            new CSharpCompilationOptions(OutputKind.DynamicallyLinkedLibrary,
                nullableContextOptions: NullableContextOptions.Enable));

        var optionsProvider = new TestOptionsProvider(dispatcherProperty, securedProperty);
        GeneratorDriver driver = CSharpGeneratorDriver.Create(
            new[] { new CrudIncrementalGenerator().AsSourceGenerator() },
            parseOptions: new CSharpParseOptions(LanguageVersion.Preview),
            optionsProvider: optionsProvider,
            driverOptions: new GeneratorDriverOptions(
                IncrementalGeneratorOutputKind.None, trackIncrementalGeneratorSteps: trackSteps));

        driver = driver.RunGeneratorsAndUpdateCompilation(compilation, out var output, out var diagnostics);
        var result = driver.GetRunResult();
        return (compilation, output, result, diagnostics, driver);
    }

    public static GeneratorDriver RunIncrementally(GeneratorDriver driver, Compilation compilation) =>
        driver.RunGenerators(compilation);

    private static ImmutableArray<MetadataReference> BuildReferences()
    {
        var paths = new HashSet<string>(StringComparer.OrdinalIgnoreCase);
        foreach (var path in ((string)AppContext.GetData("TRUSTED_PLATFORM_ASSEMBLIES")!).Split(Path.PathSeparator))
        {
            paths.Add(path);
        }

        foreach (var assembly in AppDomain.CurrentDomain.GetAssemblies())
        {
            if (!assembly.IsDynamic && !string.IsNullOrEmpty(assembly.Location))
            {
                paths.Add(assembly.Location);
            }
        }

        paths.Add(typeof(ClrinfCS.Core.RequestContext).Assembly.Location);
        paths.Add(typeof(Microsoft.AspNetCore.Routing.IEndpointRouteBuilder).Assembly.Location);
        paths.Add(typeof(Microsoft.AspNetCore.Http.HttpContext).Assembly.Location);
        paths.Add(typeof(Microsoft.Extensions.DependencyInjection.IServiceCollection).Assembly.Location);

        return paths
            .Where(static p => p.EndsWith(".dll", StringComparison.OrdinalIgnoreCase) && File.Exists(p))
            .Select(static p => (MetadataReference)MetadataReference.CreateFromFile(p))
            .ToImmutableArray();
    }

    private sealed class TestOptionsProvider : AnalyzerConfigOptionsProvider
    {
        private readonly AnalyzerConfigOptions _global;
        private readonly AnalyzerConfigOptions _empty = new TestOptions(new Dictionary<string, string>());

        public TestOptionsProvider(string? dispatcher, string? secured)
        {
            var values = new Dictionary<string, string>(StringComparer.OrdinalIgnoreCase);
            if (dispatcher is not null) values["build_property.ClrinfCrudDispatcher"] = dispatcher;
            if (secured is not null) values["build_property.ClrinfCrudSecured"] = secured;
            _global = new TestOptions(values);
        }

        public override AnalyzerConfigOptions GlobalOptions => _global;
        public override AnalyzerConfigOptions GetOptions(SyntaxTree tree) => _empty;
        public override AnalyzerConfigOptions GetOptions(AdditionalText textFile) => _empty;
    }

    private sealed class TestOptions(Dictionary<string, string> values) : AnalyzerConfigOptions
    {
        public override bool TryGetValue(string key, out string value) => values.TryGetValue(key, out value!);
    }
}
