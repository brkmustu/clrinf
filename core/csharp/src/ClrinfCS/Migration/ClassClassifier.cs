using System;
using System.IO;
using System.Linq;
using Microsoft.CodeAnalysis;
using Microsoft.CodeAnalysis.CSharp;
using Microsoft.CodeAnalysis.CSharp.Syntax;

namespace ClrinfCS.Migration;

public enum CodeElementCategory
{
    Entity,
    ValueObject,
    DbContext,
    EntityConfiguration,
    Migration,
    ServiceInterface,
    Service,
    Command,
    Query,
    Handler,
    Dto,
    Controller,
    Endpoint,
    HostStartup,
    Other
}

public class ClassClassifier
{
    public static CodeElementCategory Classify(string filePath, string fileContent)
    {
        string fileName = Path.GetFileName(filePath);
        string normalizedPath = filePath.Replace('\\', '/');

        if (fileName.Equals("Program.cs", StringComparison.OrdinalIgnoreCase) ||
            fileName.StartsWith("appsettings", StringComparison.OrdinalIgnoreCase))
        {
            return CodeElementCategory.HostStartup;
        }

        if (normalizedPath.Contains("/Migrations/") || fileName.Contains("Migration"))
        {
            return CodeElementCategory.Migration;
        }

        try
        {
            SyntaxTree tree = CSharpSyntaxTree.ParseText(fileContent);
            CompilationUnitSyntax root = tree.GetCompilationUnitRoot();

            // Check for interfaces first
            var interfaceDecl = root.DescendantNodes().OfType<InterfaceDeclarationSyntax>().FirstOrDefault();
            if (interfaceDecl != null)
            {
                string iName = interfaceDecl.Identifier.Text;
                if (iName.EndsWith("Service", StringComparison.OrdinalIgnoreCase))
                    return CodeElementCategory.ServiceInterface;
                if (iName.Equals("IEntity", StringComparison.OrdinalIgnoreCase))
                    return CodeElementCategory.Entity;
            }

            // Check class or record declarations
            var typeDecls = root.DescendantNodes().OfType<TypeDeclarationSyntax>().ToList();
            foreach (var typeDecl in typeDecls)
            {
                string typeName = typeDecl.Identifier.Text;
                var baseTypes = typeDecl.BaseList?.Types
                    .Select(t => t.Type.ToString())
                    .ToList() ?? new();

                // DbContext
                if (typeName.EndsWith("DbContext", StringComparison.OrdinalIgnoreCase) ||
                    baseTypes.Any(b => b.Contains("DbContext")))
                {
                    return CodeElementCategory.DbContext;
                }

                // EntityConfiguration
                if (baseTypes.Any(b => b.Contains("IEntityTypeConfiguration")))
                {
                    return CodeElementCategory.EntityConfiguration;
                }

                // Migration
                if (baseTypes.Any(b => b.Equals("Migration", StringComparison.OrdinalIgnoreCase)) ||
                    typeDecl.AttributeLists.Any(al => al.Attributes.Any(a => a.Name.ToString().Contains("Migration"))))
                {
                    return CodeElementCategory.Migration;
                }

                // Controller
                if (typeName.EndsWith("Controller", StringComparison.OrdinalIgnoreCase) ||
                    baseTypes.Any(b => b.Contains("Controller")))
                {
                    return CodeElementCategory.Controller;
                }

                // Service
                if (typeName.EndsWith("Service", StringComparison.OrdinalIgnoreCase) ||
                    baseTypes.Any(b => b.EndsWith("Service", StringComparison.OrdinalIgnoreCase)))
                {
                    return CodeElementCategory.Service;
                }

                // CQRS Command / Query / Handler
                if (typeName.EndsWith("Command", StringComparison.OrdinalIgnoreCase) ||
                    typeName.EndsWith("CommandHandler", StringComparison.OrdinalIgnoreCase) ||
                    baseTypes.Any(b => b.Contains("ICommand") || b.Contains("IRequest")))
                {
                    return CodeElementCategory.Command;
                }

                if (typeName.EndsWith("Query", StringComparison.OrdinalIgnoreCase) ||
                    typeName.EndsWith("QueryHandler", StringComparison.OrdinalIgnoreCase) ||
                    baseTypes.Any(b => b.Contains("IQuery")))
                {
                    return CodeElementCategory.Query;
                }

                // Entity
                if (baseTypes.Any(b => b.Equals("IEntity", StringComparison.OrdinalIgnoreCase) ||
                                       b.Equals("Entity", StringComparison.OrdinalIgnoreCase) ||
                                       b.Contains("BaseEntity")) ||
                    normalizedPath.Contains("/Entities/"))
                {
                    return CodeElementCategory.Entity;
                }
            }
        }
        catch
        {
            // Fallback to path heuristics
        }

        // Path-based fallbacks
        if (normalizedPath.Contains("/Entities/")) return CodeElementCategory.Entity;
        if (normalizedPath.Contains("/Controllers/")) return CodeElementCategory.Controller;
        if (normalizedPath.Contains("/Endpoints/")) return CodeElementCategory.Endpoint;
        if (normalizedPath.Contains("/Services/")) return CodeElementCategory.Service;
        if (normalizedPath.Contains("/Data/")) return CodeElementCategory.DbContext;

        return CodeElementCategory.Other;
    }
}
