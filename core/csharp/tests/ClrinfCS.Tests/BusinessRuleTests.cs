using ClrinfCS.Core;
using Xunit;

namespace ClrinfCS.Tests;

public class BusinessRuleTests
{
    private static readonly RequestContext Context = new("tenant-enterprise", "corr-123", "cause-456");

    private sealed record CreateItem(string Name, decimal Price) : IRequest<string>;

    private sealed class CreateItemHandler : IRequestHandler<CreateItem, string>
    {
        public ValueTask<string> HandleAsync(CreateItem request, RequestContext context, CancellationToken cancellationToken)
        {
            return ValueTask.FromResult($"Created:{request.Name}");
        }
    }

    private sealed class NameMustNotBeEmptyRule : IBusinessRule<CreateItem>
    {
        public int Priority => 1;

        public ValueTask<RuleResult> EvaluateAsync(CreateItem context, RequestContext requestContext, CancellationToken cancellationToken = default)
        {
            if (string.IsNullOrWhiteSpace(context.Name))
                return ValueTask.FromResult(RuleResult.Failed("EMPTY_NAME", "Item name cannot be empty."));

            return ValueTask.FromResult(RuleResult.Success());
        }
    }

    private sealed class PriceMustBePositiveRule : IBusinessRule<CreateItem>
    {
        public int Priority => 2;

        public ValueTask<RuleResult> EvaluateAsync(CreateItem context, RequestContext requestContext, CancellationToken cancellationToken = default)
        {
            if (context.Price <= 0)
                return ValueTask.FromResult(RuleResult.Failed("INVALID_PRICE", "Item price must be greater than zero."));

            return ValueTask.FromResult(RuleResult.Success());
        }
    }

    private sealed class EnterpriseTenantMaxPriceRule : IBusinessRule<CreateItem>
    {
        public int Priority => 3;

        public ValueTask<RuleResult> EvaluateAsync(CreateItem context, RequestContext requestContext, CancellationToken cancellationToken = default)
        {
            if (requestContext.TenantId == "tenant-enterprise" && context.Price > 10_000)
                return ValueTask.FromResult(RuleResult.Failed("PRICE_LIMIT_EXCEEDED", "Enterprise limit for item is 10,000."));

            return ValueTask.FromResult(RuleResult.Success());
        }
    }

    private sealed class OrderTrackingRule(int order, List<int> executionLog) : IBusinessRule<CreateItem>
    {
        public int Priority => order;

        public ValueTask<RuleResult> EvaluateAsync(CreateItem context, RequestContext requestContext, CancellationToken cancellationToken = default)
        {
            executionLog.Add(order);
            return ValueTask.FromResult(RuleResult.Success());
        }
    }

    [Fact]
    public async Task WhenAllRulesPass_HandlerExecutesSuccessfully()
    {
        var rules = new IBusinessRule<CreateItem>[]
        {
            new NameMustNotBeEmptyRule(),
            new PriceMustBePositiveRule(),
            new EnterpriseTenantMaxPriceRule()
        };

        var dispatcher = new Dispatcher.Builder()
            .Register(new CreateItemHandler(), rules)
            .Build();

        var result = await dispatcher.SendAsync(new CreateItem("Laptop", 1500m), Context);

        Assert.Equal("Created:Laptop", result);
    }

    [Fact]
    public async Task WhenRuleFails_ThrowsBusinessRuleViolationException_WithErrorCodeAndMessage()
    {
        var rules = new IBusinessRule<CreateItem>[]
        {
            new NameMustNotBeEmptyRule(),
            new PriceMustBePositiveRule()
        };

        var dispatcher = new Dispatcher.Builder()
            .Register(new CreateItemHandler(), rules)
            .Build();

        var ex = await Assert.ThrowsAsync<BusinessRuleViolationException>(() =>
            dispatcher.SendAsync(new CreateItem("", 100m), Context).AsTask());

        Assert.Equal("EMPTY_NAME", ex.ErrorCode);
        Assert.Equal("Item name cannot be empty.", ex.Message);

        var envelope = ex.ToErrorEnvelope(Context);
        Assert.Equal("EMPTY_NAME", envelope.ErrorCode);
        Assert.Equal("tenant-enterprise", envelope.TenantId);
        Assert.Equal("corr-123", envelope.CorrelationId);
    }

    [Fact]
    public async Task RulesExecuteInAscendingPriorityOrder_AndShortCircuitOnFirstFailure()
    {
        var executionLog = new List<int>();
        var rules = new IBusinessRule<CreateItem>[]
        {
            new OrderTrackingRule(20, executionLog),
            new OrderTrackingRule(5, executionLog),
            new NameMustNotBeEmptyRule(), // Priority 1 -> will fail if Name is empty
            new OrderTrackingRule(50, executionLog)
        };

        var dispatcher = new Dispatcher.Builder()
            .Register(new CreateItemHandler(), rules)
            .Build();

        // Empty name will fail on NameMustNotBeEmptyRule (Priority 1)
        // Priority 5, 20, 50 should NOT execute!
        await Assert.ThrowsAsync<BusinessRuleViolationException>(() =>
            dispatcher.SendAsync(new CreateItem("", 100m), Context).AsTask());

        Assert.Empty(executionLog);
    }

    [Fact]
    public async Task MultiTenantRule_FailsForSpecificTenant_PassesForOtherTenant()
    {
        var rules = new IBusinessRule<CreateItem>[]
        {
            new EnterpriseTenantMaxPriceRule()
        };

        var dispatcher = new Dispatcher.Builder()
            .Register(new CreateItemHandler(), rules)
            .Build();

        // Fails for tenant-enterprise
        await Assert.ThrowsAsync<BusinessRuleViolationException>(() =>
            dispatcher.SendAsync(new CreateItem("SuperServer", 25000m), Context).AsTask());

        // Passes for tenant-standard
        var standardContext = new RequestContext("tenant-standard", "corr", "cause");
        var result = await dispatcher.SendAsync(new CreateItem("SuperServer", 25000m), standardContext);
        Assert.Equal("Created:SuperServer", result);
    }
}
