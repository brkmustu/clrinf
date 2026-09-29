using System;
using System.Collections.Generic;
using System.Linq;
using Microsoft.CodeAnalysis;
using Microsoft.CodeAnalysis.CSharp;
using Microsoft.CodeAnalysis.CSharp.Syntax;

namespace ClrinfCS.Migration;

public class NamespaceRewriter : CSharpSyntaxRewriter
{
    private readonly string? _newNamespace;
    private readonly Dictionary<string, string> _namespaceReplacements;

    public NamespaceRewriter(string? newNamespace, Dictionary<string, string>? namespaceReplacements = null)
    {
        _newNamespace = newNamespace;
        _namespaceReplacements = namespaceReplacements ?? new Dictionary<string, string>();
    }

    public override SyntaxNode? VisitFileScopedNamespaceDeclaration(FileScopedNamespaceDeclarationSyntax node)
    {
        if (!string.IsNullOrWhiteSpace(_newNamespace))
        {
            var newName = SyntaxFactory.ParseName(_newNamespace)
                .WithLeadingTrivia(node.Name.GetLeadingTrivia())
                .WithTrailingTrivia(node.Name.GetTrailingTrivia());
            return node.WithName(newName);
        }
        return base.VisitFileScopedNamespaceDeclaration(node);
    }

    public override SyntaxNode? VisitNamespaceDeclaration(NamespaceDeclarationSyntax node)
    {
        if (!string.IsNullOrWhiteSpace(_newNamespace))
        {
            var newName = SyntaxFactory.ParseName(_newNamespace)
                .WithLeadingTrivia(node.Name.GetLeadingTrivia())
                .WithTrailingTrivia(node.Name.GetTrailingTrivia());
            return node.WithName(newName);
        }
        return base.VisitNamespaceDeclaration(node);
    }

    public override SyntaxNode? VisitUsingDirective(UsingDirectiveSyntax node)
    {
        if (node.Name != null)
        {
            string usingName = node.Name.ToString();
            foreach (var kvp in _namespaceReplacements)
            {
                if (usingName.Equals(kvp.Key, StringComparison.Ordinal) || usingName.StartsWith(kvp.Key + ".", StringComparison.Ordinal))
                {
                    string replaced = usingName.Replace(kvp.Key, kvp.Value);
                    var newName = SyntaxFactory.ParseName(replaced)
                        .WithLeadingTrivia(node.Name.GetLeadingTrivia())
                        .WithTrailingTrivia(node.Name.GetTrailingTrivia());
                    return node.WithName(newName);
                }
            }
        }
        return base.VisitUsingDirective(node);
    }

    public static string Rewrite(
        string code,
        string? newNamespace,
        Dictionary<string, string>? namespaceReplacements = null,
        IEnumerable<string>? additionalUsings = null)
    {
        SyntaxTree tree = CSharpSyntaxTree.ParseText(code);
        CompilationUnitSyntax root = tree.GetCompilationUnitRoot();

        var rewriter = new NamespaceRewriter(newNamespace, namespaceReplacements);
        var rewrittenRoot = (CompilationUnitSyntax)rewriter.Visit(root);

        if (additionalUsings != null)
        {
            var currentUsings = rewrittenRoot.Usings.Select(u => u.Name?.ToString()).Where(n => n != null).ToHashSet();
            var usingsToAdd = new List<UsingDirectiveSyntax>();

            foreach (var u in additionalUsings)
            {
                if (!string.IsNullOrWhiteSpace(u) && !currentUsings.Contains(u))
                {
                    usingsToAdd.Add(SyntaxFactory.UsingDirective(SyntaxFactory.ParseName(u)).NormalizeWhitespace());
                    currentUsings.Add(u);
                }
            }

            if (usingsToAdd.Count > 0)
            {
                rewrittenRoot = rewrittenRoot.AddUsings(usingsToAdd.ToArray());
            }
        }

        return rewrittenRoot.ToFullString();
    }
}
