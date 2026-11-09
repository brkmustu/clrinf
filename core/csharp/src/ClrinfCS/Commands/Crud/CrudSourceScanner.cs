using System;
using System.Collections.Generic;
using System.IO;
using System.Linq;
using System.Text.RegularExpressions;
using Microsoft.CodeAnalysis;
using Microsoft.CodeAnalysis.CSharp;
using Microsoft.CodeAnalysis.CSharp.Syntax;

namespace ClrinfCS.Commands.Crud;

public sealed record CrudHookMap(
    string Rule,
    string Entity,
    string Operation,
    string File);

public sealed record CrudEntityMap(
    string Name,
    string Namespace,
    string File,
    string RoutePrefix,
    string Dispatcher,
    bool Secured,
    IReadOnlyDictionary<string, string> Operations,
    IReadOnlyList<CrudHookMap> Hooks);

public sealed record CrudMapResult(IReadOnlyList<CrudEntityMap> Entities);

/// <summary>
/// Syntax-only CRUD map. Reads entities, attributes, hook classes and manual
/// overrides without loading MSBuild; generated files under obj/ are ignored.
/// </summary>
public static class CrudSourceScanner
{
    private static readonly string[] OperationNames =
        ["Create", "Update", "Delete", "GetById", "GetList"];

    private static readonly Regex DispatcherProperty = new(
        "<ClrinfCrudDispatcher>\\s*([^<]+)\\s*</ClrinfCrudDispatcher>",
        RegexOptions.IgnoreCase | RegexOptions.Compiled);

    private static readonly Regex SecuredProperty = new(
        "<ClrinfCrudSecured>\\s*([^<]+)\\s*</ClrinfCrudSecured>",
        RegexOptions.IgnoreCase | RegexOptions.Compiled);

    private static readonly Regex OperationToken = new(
        "CrudOperations\\.(\\w+)", RegexOptions.Compiled);

    public static CrudMapResult Scan(string projectPath)
    {
        var root = Path.GetFullPath(projectPath);
        var (dispatcher, secured) = ReadBuildProperties(root);
        var files = EnumerateSourceFiles(root).ToArray();

        var declaredTypes = new HashSet<string>(StringComparer.Ordinal);
        var hooks = new List<CrudHookMap>();
        var entities = new List<(string Name, string Namespace, string File, string RoutePrefix,
            string? AttributeDispatcher, IReadOnlyList<string> Operations)>();

        foreach (var file in files)
        {
            var text = File.ReadAllText(file);
            var tree = CSharpSyntaxTree.ParseText(text);
            var rootNode = tree.GetCompilationUnitRoot();

            foreach (var type in rootNode.DescendantNodes().OfType<TypeDeclarationSyntax>())
            {
                declaredTypes.Add(type.Identifier.Text);

                var hook = TryReadHook(type);
                if (hook is not null)
                {
                    hooks.Add(new CrudHookMap(
                        type.Identifier.Text,
                        hook.Value.Entity,
                        hook.Value.Operation,
                        Path.GetRelativePath(root, file)));
                }
            }

            foreach (var declaration in rootNode.DescendantNodes().OfType<ClassDeclarationSyntax>())
            {
                var attribute = declaration.AttributeLists
                    .SelectMany(static list => list.Attributes)
                    .FirstOrDefault(static a => a.Name.ToString().Contains("GenerateCrudEndpoints"));
                if (attribute is null) continue;

                var entityName = declaration.Identifier.Text;
                var entityNamespace = NamespaceOf(declaration);
                var routePrefix = ReadRoutePrefix(attribute) ?? $"/api/{Pluralize(entityName).ToLowerInvariant()}";
                var attributeDispatcher = ReadDispatcher(attribute);
                var operations = ReadOperations(attribute);
                entities.Add((entityName, entityNamespace, Path.GetRelativePath(root, file),
                    routePrefix, attributeDispatcher, operations));
            }
        }

        var maps = new List<CrudEntityMap>();
        foreach (var entity in entities.OrderBy(static e => e.Name, StringComparer.Ordinal))
        {
            var operations = new Dictionary<string, string>(StringComparer.OrdinalIgnoreCase);
            for (var i = 0; i < OperationNames.Length; i++)
            {
                var operation = OperationNames[i];
                var included = entity.Operations.Contains(operation, StringComparer.OrdinalIgnoreCase);
                var overridden = declaredTypes.Contains(operation + entity.Name + "Handler") ||
                                 declaredTypes.Contains(operation + entity.Name + "Endpoint");
                operations[operation] = overridden ? "ejected" : included ? "generated" : "disabled";
            }

            var entityHooks = hooks
                .Where(h => string.Equals(h.Entity, entity.Name, StringComparison.Ordinal))
                .ToArray();

            maps.Add(new CrudEntityMap(
                entity.Name,
                entity.Namespace,
                entity.File,
                entity.RoutePrefix,
                entity.AttributeDispatcher ?? dispatcher,
                secured,
                operations,
                entityHooks));
        }

        return new CrudMapResult(maps);
    }

    private static (string Dispatcher, bool Secured) ReadBuildProperties(string root)
    {
        var propsPath = Path.Combine(root, "Directory.Build.props");
        if (!File.Exists(propsPath))
        {
            return ("native", false);
        }

        var text = File.ReadAllText(propsPath);
        var dispatcher = DispatcherProperty.Match(text) is { Success: true } match
            ? match.Groups[1].Value.Trim().ToLowerInvariant()
            : "native";
        if (dispatcher is "mediator.net") dispatcher = "mediatornet";
        var secured = SecuredProperty.Match(text) is { Success: true } securedMatch &&
                      bool.TryParse(securedMatch.Groups[1].Value.Trim(), out var value) && value;
        return (dispatcher, secured);
    }

    private static IEnumerable<string> EnumerateSourceFiles(string root)
    {
        if (!Directory.Exists(root)) yield break;
        foreach (var file in Directory.EnumerateFiles(root, "*.cs", SearchOption.AllDirectories))
        {
            var normalized = file.Replace('\\', '/');
            if (normalized.Contains("/bin/") || normalized.Contains("/obj/") ||
                normalized.Contains(".system_generated")) continue;
            yield return file;
        }
    }

    private static (string Entity, string Operation)? TryReadHook(TypeDeclarationSyntax type)
    {
        if (type.BaseList is null) return null;
        foreach (var baseType in type.BaseList.Types)
        {
            var text = baseType.Type.ToString();
            if (!text.StartsWith("IBefore", StringComparison.Ordinal) || !text.Contains("Rule<")) continue;
            var operation = text.StartsWith("IBeforeCreateRule", StringComparison.Ordinal) ? "Create"
                : text.StartsWith("IBeforeUpdateRule", StringComparison.Ordinal) ? "Update"
                : text.StartsWith("IBeforeDeleteRule", StringComparison.Ordinal) ? "Delete"
                : null;
            if (operation is null) continue;

            var start = text.IndexOf('<') + 1;
            var end = text.LastIndexOf('>');
            if (start <= 0 || end <= start) continue;
            var entity = text.Substring(start, end - start).Trim();
            var lastDot = entity.LastIndexOf('.');
            if (lastDot >= 0) entity = entity[(lastDot + 1)..].Trim();
            return (entity, operation);
        }

        return null;
    }

    private static string NamespaceOf(SyntaxNode node)
    {
        var parts = new List<string>();
        for (var current = node.Parent; current is not null; current = current.Parent)
        {
            if (current is BaseNamespaceDeclarationSyntax ns)
            {
                parts.Insert(0, ns.Name.ToString());
            }
        }

        return string.Join(".", parts);
    }

    private static string? ReadRoutePrefix(AttributeSyntax attribute)
    {
        var argument = attribute.ArgumentList?.Arguments
            .FirstOrDefault(static a => a.NameEquals?.Name.Identifier.Text == "RoutePrefix");
        return argument?.Expression is LiteralExpressionSyntax literal &&
               literal.Token.Value is string value
            ? value
            : null;
    }

    private static string? ReadDispatcher(AttributeSyntax attribute)
    {
        var argument = attribute.ArgumentList?.Arguments
            .FirstOrDefault(static a => a.NameEquals?.Name.Identifier.Text == "Dispatcher");
        if (argument is null) return null;
        var text = argument.Expression.ToString();
        if (text.Contains("MediatorNet", StringComparison.Ordinal)) return "mediatornet";
        if (text.Contains("Native", StringComparison.Ordinal)) return "native";
        return null;
    }

    private static IReadOnlyList<string> ReadOperations(AttributeSyntax attribute)
    {
        var argument = attribute.ArgumentList?.Arguments
            .FirstOrDefault(static a => a.NameEquals?.Name.Identifier.Text == "Operations");
        if (argument is null) return OperationNames;

        var expression = argument.Expression.ToString();
        var result = new HashSet<string>(StringComparer.OrdinalIgnoreCase);
        foreach (Match match in OperationToken.Matches(expression))
        {
            var name = match.Groups[1].Value;
            var remove = match.Index > 0 && expression[..match.Index].TrimEnd().EndsWith("~", StringComparison.Ordinal);
            if (name.Equals("All", StringComparison.OrdinalIgnoreCase))
            {
                foreach (var op in OperationNames) result.Add(op);
            }
            else if (name.Equals("None", StringComparison.OrdinalIgnoreCase))
            {
                result.Clear();
            }
            else if (remove)
            {
                result.Remove(name);
            }
            else
            {
                result.Add(name);
            }
        }

        return result.Count == 0 && expression.Contains("None", StringComparison.Ordinal)
            ? Array.Empty<string>()
            : result.ToArray();
    }

    private static string Pluralize(string name)
    {
        if (name.EndsWith("s", StringComparison.Ordinal) ||
            name.EndsWith("sh", StringComparison.Ordinal) ||
            name.EndsWith("ch", StringComparison.Ordinal) ||
            name.EndsWith("x", StringComparison.Ordinal) ||
            name.EndsWith("z", StringComparison.Ordinal)) return name + "es";
        if (name.EndsWith("y", StringComparison.Ordinal) &&
            !name.EndsWith("ay", StringComparison.Ordinal) &&
            !name.EndsWith("ey", StringComparison.Ordinal) &&
            !name.EndsWith("oy", StringComparison.Ordinal) &&
            !name.EndsWith("uy", StringComparison.Ordinal))
            return name[..^1] + "ies";
        return name + "s";
    }
}
