using System.Collections.Generic;
using ClrinfCS.Migration;
using Xunit;

namespace ClrinfCS.Tests;

public class RoslynMigrationTests
{
    [Fact]
    public void ClassClassifier_ShouldClassifyEntityCorrectly()
    {
        string code = @"
namespace MyApp.Entities;

public class Product : IEntity
{
    public Guid Id { get; set; }
    public string Name { get; set; } = string.Empty;
}";
        var category = ClassClassifier.Classify("Entities/Product.cs", code);
        Assert.Equal(CodeElementCategory.Entity, category);
    }

    [Fact]
    public void ClassClassifier_ShouldClassifyDbContextCorrectly()
    {
        string code = @"
using Microsoft.EntityFrameworkCore;

namespace MyApp.Data;

public class AppDbContext : DbContext
{
    public DbSet<Product> Products { get; set; }
}";
        var category = ClassClassifier.Classify("Data/AppDbContext.cs", code);
        Assert.Equal(CodeElementCategory.DbContext, category);
    }

    [Fact]
    public void ClassClassifier_ShouldClassifyServiceAndInterfaceCorrectly()
    {
        string interfaceCode = @"
namespace MyApp.Services;

public interface IProductService : IBaseService<Product>
{
}";
        var interfaceCategory = ClassClassifier.Classify("Services/IProductService.cs", interfaceCode);
        Assert.Equal(CodeElementCategory.ServiceInterface, interfaceCategory);

        string serviceCode = @"
namespace MyApp.Services;

public class ProductService : IProductService
{
}";
        var serviceCategory = ClassClassifier.Classify("Services/ProductService.cs", serviceCode);
        Assert.Equal(CodeElementCategory.Service, serviceCategory);
    }

    [Fact]
    public void ClassClassifier_ShouldClassifyControllerCorrectly()
    {
        string code = @"
using Microsoft.AspNetCore.Mvc;

namespace MyApp.Controllers;

public class ProductsController : Controller
{
    public IActionResult Index() => View();
}";
        var category = ClassClassifier.Classify("Controllers/ProductsController.cs", code);
        Assert.Equal(CodeElementCategory.Controller, category);
    }

    [Fact]
    public void ClassClassifier_ShouldClassifyHostStartupCorrectly()
    {
        string code = @"
var builder = WebApplication.CreateBuilder(args);
var app = builder.Build();
app.Run();";
        var category = ClassClassifier.Classify("Program.cs", code);
        Assert.Equal(CodeElementCategory.HostStartup, category);
    }

    [Fact]
    public void NamespaceRewriter_ShouldRewriteFileScopedNamespace()
    {
        string source = @"namespace OldApp.Entities;

public class Customer
{
    public int Id { get; set; }
}";

        string result = NamespaceRewriter.Rewrite(source, "NewApp.Core.Entities");

        Assert.Contains("namespace NewApp.Core.Entities;", result);
        Assert.DoesNotContain("namespace OldApp.Entities;", result);
        Assert.Contains("public class Customer", result);
    }

    [Fact]
    public void NamespaceRewriter_ShouldReplaceUsingDirectivesAndAddUsings()
    {
        string source = @"using System;
using OldApp.Entities;

namespace OldApp.Controllers;

public class CustomersController
{
}";

        var replacements = new Dictionary<string, string>
        {
            { "OldApp.Entities", "NewApp.Core.Entities" }
        };

        var additional = new[] { "NewApp.Core.Services", "Microsoft.AspNetCore.Mvc" };

        string result = NamespaceRewriter.Rewrite(source, "NewApp.Web.Controllers", replacements, additional);

        Assert.Contains("namespace NewApp.Web.Controllers;", result);
        Assert.Contains("using NewApp.Core.Entities;", result);
        Assert.Contains("using NewApp.Core.Services;", result);
        Assert.Contains("using Microsoft.AspNetCore.Mvc;", result);
        Assert.DoesNotContain("using OldApp.Entities;", result);
    }
}
