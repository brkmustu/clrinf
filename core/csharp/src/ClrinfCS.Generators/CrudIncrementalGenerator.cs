using System;
using System.Collections.Generic;
using System.Collections.Immutable;
using System.Linq;
using System.Threading;
using Microsoft.CodeAnalysis;
using Microsoft.CodeAnalysis.CSharp.Syntax;

namespace ClrinfCS.Generators;

/// <summary>
/// Generates CRUD handlers and Minimal API endpoints for entities marked with
/// <c>[GenerateCrudEndpoints]</c>. Only text is produced; the generator does not
/// reference ASP.NET Core or Mediator.Net at build time.
/// </summary>
[Generator(LanguageNames.CSharp)]
public sealed class CrudIncrementalGenerator : IIncrementalGenerator
{
    private const string AttributeMetadataName = "ClrinfCS.Core.GenerateCrudEndpointsAttribute";

    private static readonly SymbolDisplayFormat TypeFormat = SymbolDisplayFormat.FullyQualifiedFormat
        .AddMiscellaneousOptions(SymbolDisplayMiscellaneousOptions.IncludeNullableReferenceTypeModifier |
                                 SymbolDisplayMiscellaneousOptions.EscapeKeywordIdentifiers);

    private static readonly SymbolDisplayFormat NonNullableTypeFormat = SymbolDisplayFormat.FullyQualifiedFormat
        .AddMiscellaneousOptions(SymbolDisplayMiscellaneousOptions.EscapeKeywordIdentifiers);

    public void Initialize(IncrementalGeneratorInitializationContext context)
    {
        var entities = context.SyntaxProvider
            .ForAttributeWithMetadataName(
                AttributeMetadataName,
                static (node, _) => node is ClassDeclarationSyntax,
                static (ctx, ct) => GetEntityModel(ctx, ct))
            .Where(static m => m is not null)
            .Collect();

        // Smart Override detection deliberately uses a syntax pipeline instead of
        // compilation.GetTypeByMetadataName, so the attribute transform stays incremental.
        var declaredTypes = context.SyntaxProvider
            .CreateSyntaxProvider(
                static (node, _) => node is TypeDeclarationSyntax,
                static (ctx, ct) => GetDeclaredTypeName(ctx, ct))
            .Where(static name => !string.IsNullOrEmpty(name))
            .Collect();

        var options = context.AnalyzerConfigOptionsProvider.Select(static (provider, _) =>
        {
            provider.GlobalOptions.TryGetValue("build_property.ClrinfCrudDispatcher", out var dispatcher);
            provider.GlobalOptions.TryGetValue("build_property.ClrinfCrudSecured", out var secured);
            return new BuildOptions(dispatcher, string.Equals(secured, "true", StringComparison.OrdinalIgnoreCase));
        });

        var environment = context.CompilationProvider.Select(static (compilation, _) =>
            new BuildEnvironment(
                hasAspNetCore: compilation.GetTypeByMetadataName(
                    "Microsoft.AspNetCore.Routing.IEndpointRouteBuilder") is not null,
                hasMediatorNet: compilation.GetTypeByMetadataName("Mediator.Net.Contracts.IRequest`1") is not null &&
                                compilation.GetTypeByMetadataName("Mediator.Net.Contracts.IRequestHandler`2") is not null &&
                                compilation.GetTypeByMetadataName("Mediator.Net.Contracts.IMediator") is not null));

        var input = entities.Combine(declaredTypes).Combine(options).Combine(environment);

        context.RegisterSourceOutput(input, static (spc, data) =>
        {
            var entityModels = data.Left.Left.Left;
            var declared = data.Left.Left.Right;
            var buildOptions = data.Left.Right;
            var buildEnvironment = data.Right;
            Emit(spc, entityModels, declared, buildOptions, buildEnvironment);
        });
    }

    private static string GetDeclaredTypeName(GeneratorSyntaxContext context, CancellationToken cancellationToken)
    {
        var symbol = context.SemanticModel.GetDeclaredSymbol((TypeDeclarationSyntax)context.Node, cancellationToken);
        return symbol?.Name ?? string.Empty;
    }

    private static RawEntityModel? GetEntityModel(GeneratorAttributeSyntaxContext context,
        CancellationToken cancellationToken)
    {
        if (context.TargetSymbol is not INamedTypeSymbol entitySymbol)
        {
            return null;
        }

        var attribute = context.Attributes[0];
        var operations = CrudOperationFlags.All;
        string? routePrefix = null;
        var dispatcher = DispatcherKind.Default;

        foreach (var argument in attribute.NamedArguments)
        {
            switch (argument.Key)
            {
                case "Operations" when argument.Value.Value is not null:
                    operations = (CrudOperationFlags)Convert.ToInt32(argument.Value.Value);
                    break;
                case "RoutePrefix":
                    routePrefix = argument.Value.Value as string;
                    break;
                case "Dispatcher" when argument.Value.Value is not null:
                    dispatcher = (DispatcherKind)Convert.ToInt32(argument.Value.Value);
                    break;
            }
        }

        var properties = entitySymbol.GetMembers()
            .OfType<IPropertySymbol>()
            .Where(static p => p.DeclaredAccessibility == Accessibility.Public &&
                               !p.IsStatic && !p.IsIndexer && p.GetMethod is not null)
            .Where(static p => !p.GetAttributes().Any(static a =>
                a.AttributeClass?.ToDisplayString() == "ClrinfCS.Core.CrudIgnoreAttribute"))
            .Select(static p =>
            {
                var (jsonKind, underlyingType) = ClassifyJsonKind(p.Type);
                return new CrudPropertyModel(
                    p.Name,
                    p.Type.ToDisplayString(TypeFormat),
                    p.SetMethod?.DeclaredAccessibility == Accessibility.Public,
                    jsonKind,
                    underlyingType);
            })
            .ToImmutableArray();

        var idProperty = properties.FirstOrDefault(static p =>
            string.Equals(p.Name, "Id", StringComparison.OrdinalIgnoreCase));

        return new RawEntityModel(
            entitySymbol.Name,
            entitySymbol.ContainingNamespace.IsGlobalNamespace
                ? string.Empty
                : entitySymbol.ContainingNamespace.ToDisplayString(),
            routePrefix,
            dispatcher,
            operations,
            idProperty.Name ?? string.Empty,
            idProperty.Type ?? string.Empty,
            properties);
    }

    private static void Emit(SourceProductionContext context,
        ImmutableArray<RawEntityModel?> rawEntities,
        ImmutableArray<string> declaredTypeNames,
        BuildOptions options,
        BuildEnvironment environment)
    {
        var declared = new HashSet<string>(declaredTypeNames, StringComparer.Ordinal);
        var plans = new List<CrudEntityPlan>();
        var anyEndpoint = false;

        foreach (var raw in rawEntities.Where(static e => e is not null).Select(static e => e!))
        {
            if (string.IsNullOrEmpty(raw.IdPropertyName))
            {
                context.ReportDiagnostic(Diagnostic.Create(
                    CrudDiagnostics.MissingId, Location.None, raw.Name));
                continue;
            }

            var dispatcher = ResolveDispatcher(raw.AttributeDispatcher, options.Dispatcher);
            if (dispatcher == DispatcherKind.MediatorNet && !environment.HasMediatorNet)
            {
                context.ReportDiagnostic(Diagnostic.Create(
                    CrudDiagnostics.MediatorNetUnavailable, Location.None, raw.Name));
                continue;
            }

            var plural = Pluralize(raw.Name);
            var featureNamespace = string.IsNullOrEmpty(raw.EntityNamespace)
                ? "Features." + plural
                : raw.EntityNamespace + ".Features." + plural;
            var routePrefix = string.IsNullOrWhiteSpace(raw.RoutePrefix)
                ? "/api/" + plural.ToLowerInvariant()
                : raw.RoutePrefix!;

            var entity = new CrudEntityModel(
                raw.Name,
                raw.EntityNamespace,
                featureNamespace,
                plural,
                raw.IdPropertyName,
                raw.IdType,
                routePrefix,
                options.Secured,
                environment.HasAspNetCore,
                environment.HasMediatorNet,
                dispatcher,
                raw.Operations,
                raw.Properties);

            var generated = new bool[CrudEmitter.Operations.Length];
            var handlerDeclared = new bool[CrudEmitter.Operations.Length];
            var endpointDeclared = new bool[CrudEmitter.Operations.Length];

            for (var i = 0; i < CrudEmitter.Operations.Length; i++)
            {
                var operation = CrudEmitter.Operations[i];
                var handlerName = entity.HandlerName(operation);
                var endpointName = entity.EndpointName(operation);
                var requestName = entity.RequestName(operation);

                handlerDeclared[i] = declared.Contains(handlerName);
                endpointDeclared[i] = declared.Contains(endpointName);
                var overridden = handlerDeclared[i] || endpointDeclared[i] || declared.Contains(requestName);

                generated[i] = entity.Includes(operation) && !overridden;

                if (!entity.Includes(operation) && !handlerDeclared[i] && !endpointDeclared[i])
                {
                    context.ReportDiagnostic(Diagnostic.Create(
                        CrudDiagnostics.ExcludedOperationWithoutOverride,
                        Location.None,
                        entity.Name,
                        operation.ToString(),
                        handlerName));
                }
            }

            context.AddSource($"{featureNamespace}.{entity.Name}.Crud.Models.g.cs",
                SourceTextFactory.Create(CrudEmitter.EmitModels(entity)));

            for (var i = 0; i < CrudEmitter.Operations.Length; i++)
            {
                if (!generated[i]) continue;
                var operation = CrudEmitter.Operations[i];
                context.AddSource($"{featureNamespace}.{entity.Name}.Crud.{operation}.Handler.g.cs",
                    SourceTextFactory.Create(CrudEmitter.EmitHandler(entity, operation)));

                if (environment.HasAspNetCore)
                {
                    context.AddSource($"{featureNamespace}.{entity.Name}.Crud.{operation}.Endpoint.g.cs",
                        SourceTextFactory.Create(CrudEmitter.EmitEndpoint(entity, operation)));
                    anyEndpoint = true;
                }
            }

            if (environment.HasAspNetCore)
            {
                for (var i = 0; i < CrudEmitter.Operations.Length; i++)
                {
                    if (endpointDeclared[i]) anyEndpoint = true;
                }
            }

            plans.Add(new CrudEntityPlan(entity, generated, handlerDeclared, endpointDeclared));
        }

        if (plans.Count == 0) return;

        context.AddSource("GeneratedCrudJsonSupport.g.cs",
            SourceTextFactory.Create(CrudEmitter.EmitJsonSupport()));

        if (environment.HasAspNetCore)
        {
            context.AddSource("GeneratedCrudExtensions.g.cs",
                SourceTextFactory.Create(CrudEmitter.EmitCentral(plans)));
            if (anyEndpoint)
            {
                context.AddSource("GeneratedCrudSupport.g.cs",
                    SourceTextFactory.Create(CrudEmitter.EmitSupport()));
            }
        }
        else
        {
            context.AddSource("GeneratedCrudDi.g.cs",
                SourceTextFactory.Create(CrudEmitter.EmitDi(plans)));
        }
    }

    private static (CrudPropertyJsonKind Kind, string UnderlyingType) ClassifyJsonKind(ITypeSymbol type)
    {
        var isNullableValue = type is INamedTypeSymbol
        {
            OriginalDefinition.SpecialType: SpecialType.System_Nullable_T
        };
        var underlying = isNullableValue ? ((INamedTypeSymbol)type).TypeArguments[0] : type;
        var isNullableRef = !isNullableValue && type.NullableAnnotation == NullableAnnotation.Annotated;
        var isNullable = isNullableValue || isNullableRef;
        var underlyingName = underlying.ToDisplayString(NonNullableTypeFormat);

        if (underlying.TypeKind == TypeKind.Enum)
        {
            return (isNullableValue ? CrudPropertyJsonKind.NullableEnum : CrudPropertyJsonKind.Enum, underlyingName);
        }

        switch (underlying.SpecialType)
        {
            case SpecialType.System_String:
                return (isNullable ? CrudPropertyJsonKind.NullableString : CrudPropertyJsonKind.String, underlyingName);
            case SpecialType.System_Boolean:
                return (isNullable ? CrudPropertyJsonKind.NullableBoolean : CrudPropertyJsonKind.Boolean, underlyingName);
            case SpecialType.System_Int32:
                return (isNullable ? CrudPropertyJsonKind.NullableInt32 : CrudPropertyJsonKind.Int32, underlyingName);
            case SpecialType.System_Int64:
                return (isNullable ? CrudPropertyJsonKind.NullableInt64 : CrudPropertyJsonKind.Int64, underlyingName);
            case SpecialType.System_Int16:
                return (isNullable ? CrudPropertyJsonKind.NullableInt16 : CrudPropertyJsonKind.Int16, underlyingName);
            case SpecialType.System_Byte:
                return (isNullable ? CrudPropertyJsonKind.NullableByte : CrudPropertyJsonKind.Byte, underlyingName);
            case SpecialType.System_UInt32:
                return (isNullable ? CrudPropertyJsonKind.NullableUInt32 : CrudPropertyJsonKind.UInt32, underlyingName);
            case SpecialType.System_UInt64:
                return (isNullable ? CrudPropertyJsonKind.NullableUInt64 : CrudPropertyJsonKind.UInt64, underlyingName);
            case SpecialType.System_Decimal:
                return (isNullable ? CrudPropertyJsonKind.NullableDecimal : CrudPropertyJsonKind.Decimal, underlyingName);
            case SpecialType.System_Double:
                return (isNullable ? CrudPropertyJsonKind.NullableDouble : CrudPropertyJsonKind.Double, underlyingName);
            case SpecialType.System_Single:
                return (isNullable ? CrudPropertyJsonKind.NullableSingle : CrudPropertyJsonKind.Single, underlyingName);
            case SpecialType.System_DateTime:
                return (isNullable ? CrudPropertyJsonKind.NullableDateTime : CrudPropertyJsonKind.DateTime, underlyingName);
        }

        var fullName = underlying.ToDisplayString();
        return fullName switch
        {
            "System.Guid" => (isNullable ? CrudPropertyJsonKind.NullableGuid : CrudPropertyJsonKind.Guid, underlyingName),
            "System.DateTimeOffset" => (isNullable ? CrudPropertyJsonKind.NullableDateTimeOffset : CrudPropertyJsonKind.DateTimeOffset, underlyingName),
            "System.DateOnly" => (isNullable ? CrudPropertyJsonKind.NullableDateOnly : CrudPropertyJsonKind.DateOnly, underlyingName),
            "System.TimeOnly" => (isNullable ? CrudPropertyJsonKind.NullableTimeOnly : CrudPropertyJsonKind.TimeOnly, underlyingName),
            _ => (isNullable ? CrudPropertyJsonKind.NullableParsable : CrudPropertyJsonKind.Parsable, underlyingName)
        };
    }

    private static DispatcherKind ResolveDispatcher(DispatcherKind attributeValue, string? buildProperty)
    {
        if (attributeValue != DispatcherKind.Default)
        {
            return attributeValue;
        }

        if (string.Equals(buildProperty, "mediatornet", StringComparison.OrdinalIgnoreCase) ||
            string.Equals(buildProperty, "mediator.net", StringComparison.OrdinalIgnoreCase))
        {
            return DispatcherKind.MediatorNet;
        }

        return string.Equals(buildProperty, "native", StringComparison.OrdinalIgnoreCase)
            ? DispatcherKind.Native
            : DispatcherKind.Native;
    }

    private static string Pluralize(string name)
    {
        if (name.EndsWith("s", StringComparison.Ordinal) ||
            name.EndsWith("sh", StringComparison.Ordinal) ||
            name.EndsWith("ch", StringComparison.Ordinal) ||
            name.EndsWith("x", StringComparison.Ordinal) ||
            name.EndsWith("z", StringComparison.Ordinal))
        {
            return name + "es";
        }

        if (name.EndsWith("y", StringComparison.Ordinal) &&
            !name.EndsWith("ay", StringComparison.Ordinal) &&
            !name.EndsWith("ey", StringComparison.Ordinal) &&
            !name.EndsWith("oy", StringComparison.Ordinal) &&
            !name.EndsWith("uy", StringComparison.Ordinal))
        {
            return name.Substring(0, name.Length - 1) + "ies";
        }

        return name + "s";
    }
}
