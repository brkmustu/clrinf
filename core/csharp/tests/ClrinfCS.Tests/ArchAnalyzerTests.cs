using System.Collections.Generic;
using ClrinfCS.Lint;
using Xunit;

namespace ClrinfCS.Tests;

public class ArchAnalyzerTests
{
    [Fact]
    public void DomainLayer_WithForbiddenEfUsing_ReportsARCH001()
    {
        string code = @"
using System;
using Microsoft.EntityFrameworkCore;

namespace MyProject.Domain.Entities;

public class Product
{
    public Guid Id { get; set; }
}
";
        var violations = new List<ArchViolation>();
        ArchAnalyzer.AnalyzeFile("/repo/src/MyProject.Domain/Entities/Product.cs", code, violations);

        Assert.Contains(violations, v => v.RuleId == "ARCH001" && v.Severity == "Error");
    }

    [Fact]
    public void CleanDomainLayer_ReportsNoViolations()
    {
        string code = @"
using System;

namespace MyProject.Domain.Entities;

public class Product
{
    public Guid Id { get; set; }
    public string Name { get; set; } = string.Empty;
}
";
        var violations = new List<ArchViolation>();
        ArchAnalyzer.AnalyzeFile("/repo/src/MyProject.Domain/Entities/Product.cs", code, violations);

        Assert.Empty(violations);
    }

    [Fact]
    public void RuleClass_NotImplementingContract_ReportsARCH002()
    {
        string code = @"
namespace MyProject.Application.Features.Products.Rules;

public class DiscountLimitRule
{
    public void Check() {}
}
";
        var violations = new List<ArchViolation>();
        ArchAnalyzer.AnalyzeFile("/repo/src/MyProject.Application/Features/Products/Rules/DiscountLimitRule.cs", code, violations);

        Assert.Contains(violations, v => v.RuleId == "ARCH002" && v.Severity == "Warning");
    }

    [Fact]
    public void RuleClass_ImplementingIBusinessRule_ReportsNoViolation()
    {
        string code = @"
using ClrinfCS.Core;

namespace MyProject.Application.Features.Products.Rules;

public class DiscountLimitRule : IBusinessRule<object>
{
    public ValueTask<RuleResult> EvaluateAsync(object context, RequestContext requestContext, CancellationToken cancellationToken = default)
    {
        return ValueTask.FromResult(RuleResult.Success());
    }
}
";
        var violations = new List<ArchViolation>();
        ArchAnalyzer.AnalyzeFile("/repo/src/MyProject.Application/Features/Products/Rules/DiscountLimitRule.cs", code, violations);

        Assert.DoesNotContain(violations, v => v.RuleId == "ARCH002");
    }

    [Fact]
    public void Controller_InjectingDbContextDirectly_ReportsARCH003()
    {
        string code = @"
using Microsoft.AspNetCore.Mvc;

namespace MyProject.API.Controllers;

public class ProductsController : ControllerBase
{
    public ProductsController(AppDbContext context)
    {
    }
}
";
        var violations = new List<ArchViolation>();
        ArchAnalyzer.AnalyzeFile("/repo/src/MyProject.API/Controllers/ProductsController.cs", code, violations);

        Assert.Contains(violations, v => v.RuleId == "ARCH003" && v.Severity == "Warning");
    }
}
