using System;
using System.Collections.Generic;
using System.Threading;
using System.Threading.Tasks;
using ClrinfCS.Core;
using Xunit;

namespace ClrinfCS.Tests;

public class CrudRuleRunnerTests
{
    private sealed class Item
    {
        public int Id { get; set; }
        public string Name { get; set; } = string.Empty;
    }

    private static readonly RequestContext Context = new("tenant-a", "corr-1", "cause-1");

    private sealed class RecordingRule(string name, int priority, bool success, List<string> log)
        : IBeforeDeleteRule<Item>
    {
        public int Priority => priority;

        public ValueTask<RuleResult> EvaluateAsync(Item entity, RequestContext requestContext,
            CancellationToken cancellationToken = default)
        {
            log.Add(name);
            return ValueTask.FromResult(success
                ? RuleResult.Success()
                : RuleResult.Failed("ACTIVE_PRODUCTS", "Kategori silinemez."));
        }
    }

    [Fact]
    public async Task DeleteRules_RunInPriorityOrder()
    {
        var log = new List<string>();
        var rules = new IBeforeDeleteRule<Item>[]
        {
            new RecordingRule("second", 20, true, log),
            new RecordingRule("first", 10, true, log)
        };

        await CrudRuleRunner.RunBeforeDeleteAsync(rules, new Item { Id = 1 }, Context);

        Assert.Equal(new[] { "first", "second" }, log);
    }

    [Fact]
    public async Task DeleteRules_ShortCircuitOnFirstFailure()
    {
        var log = new List<string>();
        var rules = new IBeforeDeleteRule<Item>[]
        {
            new RecordingRule("first", 10, false, log),
            new RecordingRule("never", 20, true, log)
        };

        var ex = await Assert.ThrowsAsync<BusinessRuleViolationException>(async () =>
            await CrudRuleRunner.RunBeforeDeleteAsync(rules, new Item { Id = 1 }, Context));

        Assert.Equal("ACTIVE_PRODUCTS", ex.ErrorCode);
        Assert.Equal("Kategori silinemez.", ex.Message);
        Assert.Equal(new[] { "first" }, log);

        var envelope = ex.ToErrorEnvelope(Context);
        Assert.Equal("tenant-a", envelope.TenantId);
        Assert.Equal("corr-1", envelope.CorrelationId);
    }

    [Fact]
    public async Task CreateRules_UseSharedFailureTranslation()
    {
        var rule = new CreateRule(false);
        var ex = await Assert.ThrowsAsync<BusinessRuleViolationException>(async () =>
            await CrudRuleRunner.RunBeforeCreateAsync(new[] { rule }, new Item(), Context));

        Assert.Equal("NAME_REQUIRED", ex.ErrorCode);
    }

    private sealed class CreateRule(bool success) : IBeforeCreateRule<Item>
    {
        public ValueTask<RuleResult> EvaluateAsync(Item entity, RequestContext requestContext,
            CancellationToken cancellationToken = default) =>
            ValueTask.FromResult(success
                ? RuleResult.Success()
                : RuleResult.Failed("NAME_REQUIRED", "Ad zorunludur."));
    }

    [Fact]
    public void CrudQuery_ClampsPageAndSize()
    {
        var query = new CrudQuery(0, 0, "  ");
        Assert.Equal(1, query.Page);
        Assert.Equal(CrudQuery.DefaultPageSize, query.PageSize);
        Assert.Null(query.Sort);

        var huge = new CrudQuery(3, 10_000);
        Assert.Equal(CrudQuery.MaxPageSize, huge.PageSize);
        Assert.Equal((3 - 1) * CrudQuery.MaxPageSize, huge.Skip);
    }

    [Fact]
    public void PagedResult_ComputesTotalPages()
    {
        var page = new PagedResult<int>(new[] { 1, 2 }, 1, 2, 5);
        Assert.Equal(3, page.TotalPages);
    }
}
