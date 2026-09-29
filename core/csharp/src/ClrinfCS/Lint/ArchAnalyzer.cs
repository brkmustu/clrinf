using System;
using System.Collections.Generic;
using System.IO;
using System.Linq;
using System.Threading.Tasks;
using Microsoft.CodeAnalysis;
using Microsoft.CodeAnalysis.CSharp;
using Microsoft.CodeAnalysis.CSharp.Syntax;

namespace ClrinfCS.Lint;

public record ArchViolation(
    string RuleId,
    string Severity, // "Error", "Warning", "Info"
    string Message,
    string FilePath,
    int LineNumber,
    string Snippet = ""
);

public class ArchAnalyzer
{
    private static readonly HashSet<string> ForbiddenDomainUsings = new(StringComparer.OrdinalIgnoreCase)
    {
        "Microsoft.EntityFrameworkCore",
        "Microsoft.AspNetCore",
        "Microsoft.AspNetCore.Mvc",
        "Microsoft.AspNetCore.Http"
    };

    public static async Task<IReadOnlyList<ArchViolation>> AnalyzeDirectoryAsync(string directoryPath)
    {
        var violations = new List<ArchViolation>();
        if (!Directory.Exists(directoryPath))
            return violations;

        var csFiles = Directory.GetFiles(directoryPath, "*.cs", SearchOption.AllDirectories)
            .Where(f => !f.Contains(Path.Combine("bin", "")) &&
                        !f.Contains(Path.Combine("obj", "")) &&
                        !f.Contains(".system_generated"))
            .ToList();

        foreach (var file in csFiles)
        {
            var content = await File.ReadAllTextAsync(file);
            AnalyzeFile(file, content, violations);
        }

        return violations;
    }

    public static void AnalyzeFile(string filePath, string fileContent, List<ArchViolation> violations)
    {
        var normalizedPath = filePath.Replace('\\', '/');

        SyntaxTree tree;
        try
        {
            tree = CSharpSyntaxTree.ParseText(fileContent);
        }
        catch
        {
            return;
        }

        var root = tree.GetCompilationUnitRoot();
        var isDomainLayer = normalizedPath.Contains("/Domain/") ||
                            normalizedPath.Contains(".Domain/") ||
                            normalizedPath.Contains("/Entities/");

        // ARCH001: Domain Layer Purity (No EF, ASP.NET, Persistence, Infrastructure)
        if (isDomainLayer)
        {
            foreach (var usingDirective in root.Usings)
            {
                var name = usingDirective.Name?.ToString() ?? string.Empty;
                var isForbidden = ForbiddenDomainUsings.Any(f => name.StartsWith(f, StringComparison.OrdinalIgnoreCase)) ||
                                  name.Contains(".Persistence") ||
                                  name.Contains(".Infrastructure") ||
                                  name.Contains(".Application");

                if (isForbidden)
                {
                    var lineSpan = tree.GetLineSpan(usingDirective.Span);
                    violations.Add(new ArchViolation(
                        "ARCH001",
                        "Error",
                        $"Domain katmanı harici katman veya framework bağımlılığı içeremez: '{name}'",
                        filePath,
                        lineSpan.StartLinePosition.Line + 1,
                        usingDirective.ToString()
                    ));
                }
            }
        }

        // Check TypeDeclarations
        foreach (var typeDecl in root.DescendantNodes().OfType<TypeDeclarationSyntax>())
        {
            var typeName = typeDecl.Identifier.Text;
            var lineSpan = tree.GetLineSpan(typeDecl.Identifier.Span);
            var lineNo = lineSpan.StartLinePosition.Line + 1;

            // ARCH002: Rule Convention
            if (normalizedPath.Contains("/Rules/") || typeName.EndsWith("Rule", StringComparison.OrdinalIgnoreCase))
            {
                if (!typeName.Equals("BaseBusinessRules", StringComparison.OrdinalIgnoreCase) &&
                    !typeName.EndsWith("Rules", StringComparison.OrdinalIgnoreCase))
                {
                    var implementsRule = typeDecl.BaseList?.Types.Any(t =>
                    {
                        var tText = t.Type.ToString();
                        return tText.StartsWith("IBusinessRule") || tText.Equals("BaseBusinessRules");
                    }) ?? false;

                    if (!implementsRule)
                    {
                        violations.Add(new ArchViolation(
                            "ARCH002",
                            "Warning",
                            $"'{typeName}' kural sınıfı IBusinessRule<TContext> arayüzünü uygulamalı veya BaseBusinessRules'tan türemelidir.",
                            filePath,
                            lineNo,
                            typeName
                        ));
                    }
                }
            }

            // ARCH003: Direct DbContext in Controller / API Host
            var isHostOrController = normalizedPath.Contains("/Controllers/") ||
                                     normalizedPath.Contains("/Endpoints/") ||
                                     typeName.EndsWith("Controller", StringComparison.OrdinalIgnoreCase);

            if (isHostOrController && typeDecl is ClassDeclarationSyntax classDecl)
            {
                foreach (var ctor in classDecl.Members.OfType<ConstructorDeclarationSyntax>())
                {
                    foreach (var param in ctor.ParameterList.Parameters)
                    {
                        var paramType = param.Type?.ToString() ?? string.Empty;
                        if (paramType.EndsWith("DbContext", StringComparison.OrdinalIgnoreCase) ||
                            paramType.Equals("DbContext", StringComparison.OrdinalIgnoreCase))
                        {
                            var pSpan = tree.GetLineSpan(param.Span);
                            violations.Add(new ArchViolation(
                                "ARCH003",
                                "Warning",
                                $"'{typeName}' denetleyicisi doğrudan DbContext ('{paramType}') enjekte etmemelidir. IDispatcher veya servis katmanı kullanılmalıdır.",
                                filePath,
                                pSpan.StartLinePosition.Line + 1,
                                param.ToString()
                            ));
                        }
                    }
                }
            }

            // ARCH004: CQRS Request Contract
            if ((typeName.EndsWith("Command", StringComparison.OrdinalIgnoreCase) ||
                 typeName.EndsWith("Query", StringComparison.OrdinalIgnoreCase)) &&
                !typeName.EndsWith("Handler", StringComparison.OrdinalIgnoreCase))
            {
                var implementsRequest = typeDecl.BaseList?.Types.Any(t =>
                {
                    var tText = t.Type.ToString();
                    return tText.StartsWith("IRequest") || tText.StartsWith("ICommand") || tText.StartsWith("IQuery");
                }) ?? false;

                if (!implementsRequest)
                {
                    violations.Add(new ArchViolation(
                        "ARCH004",
                        "Info",
                        $"'{typeName}' CQRS mesajı IRequest<TResponse> arayüzünü uygulamalıdır.",
                        filePath,
                        lineNo,
                        typeName
                    ));
                }
            }
        }
    }
}
