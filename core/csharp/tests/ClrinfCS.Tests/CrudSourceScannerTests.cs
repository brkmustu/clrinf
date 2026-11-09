using System;
using System.IO;
using System.Linq;
using ClrinfCS.Commands.Crud;
using Xunit;

namespace ClrinfCS.Tests;

public class CrudSourceScannerTests
{
    private static string CreateProject()
    {
        var root = Path.Combine(Path.GetTempPath(), "clrinf-crud-map-" + Guid.NewGuid().ToString("N"));
        Directory.CreateDirectory(Path.Combine(root, "Domain"));
        Directory.CreateDirectory(Path.Combine(root, "Features", "Categories", "Rules"));
        Directory.CreateDirectory(Path.Combine(root, "Features", "Category"));

        File.WriteAllText(Path.Combine(root, "Directory.Build.props"), """
            <Project>
              <PropertyGroup>
                <ClrinfCrudDispatcher>native</ClrinfCrudDispatcher>
                <ClrinfCrudSecured>true</ClrinfCrudSecured>
              </PropertyGroup>
            </Project>
            """);

        File.WriteAllText(Path.Combine(root, "Domain", "Category.cs"), """
            using ClrinfCS.Core;
            namespace Shop.Domain;

            [GenerateCrudEndpoints(Operations = CrudOperations.All & ~CrudOperations.Delete)]
            public class Category
            {
                public int Id { get; set; }
                public string Name { get; set; } = string.Empty;
            }
            """);

        File.WriteAllText(Path.Combine(root, "Domain", "Brand.cs"), """
            using ClrinfCS.Core;
            namespace Shop.Domain;

            [GenerateCrudEndpoints(Operations = CrudOperations.Create | CrudOperations.GetList,
                Dispatcher = CrudDispatcher.MediatorNet)]
            public class Brand
            {
                public int Id { get; set; }
            }
            """);

        File.WriteAllText(Path.Combine(root, "Features", "Categories", "Rules",
            "ActiveProductsCheckRule.cs"), """
            using ClrinfCS.Core;
            namespace Shop.Domain.Features.Categories.Rules;

            public class ActiveProductsCheckRule : IBeforeDeleteRule<Category>
            {
                public ValueTask<RuleResult> EvaluateAsync(Category entity, RequestContext context,
                    CancellationToken cancellationToken = default) =>
                    ValueTask.FromResult(RuleResult.Success());
            }
            """);

        File.WriteAllText(Path.Combine(root, "Features", "Category", "UpdateCategoryHandler.cs"), """
            namespace Shop.Domain.Features.Categories;

            public sealed class UpdateCategoryHandler { }
            """);

        return root;
    }

    [Fact]
    public void Scan_ReportsOperationsHooksAndDispatcher()
    {
        var root = CreateProject();
        try
        {
            var map = CrudSourceScanner.Scan(root);
            Assert.Equal(2, map.Entities.Count);

            var category = map.Entities.Single(e => e.Name == "Category");
            Assert.Equal("native", category.Dispatcher);
            Assert.True(category.Secured);
            Assert.Equal("generated", category.Operations["Create"]);
            Assert.Equal("ejected", category.Operations["Update"]);
            Assert.Equal("disabled", category.Operations["Delete"]);
            Assert.Equal("generated", category.Operations["GetList"]);
            var hook = Assert.Single(category.Hooks);
            Assert.Equal("ActiveProductsCheckRule", hook.Rule);
            Assert.Equal("Delete", hook.Operation);

            var brand = map.Entities.Single(e => e.Name == "Brand");
            Assert.Equal("mediatornet", brand.Dispatcher);
            Assert.Equal("generated", brand.Operations["Create"]);
            Assert.Equal("disabled", brand.Operations["Update"]);
        }
        finally
        {
            Directory.Delete(root, recursive: true);
        }
    }
}
