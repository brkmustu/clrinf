namespace ClrinfCS.Core;

/// <summary>
/// Defines an isolated, strongly-typed business rule contract evaluated before handling an operation.
/// Rules are deterministic, step-debuggable C# classes.
/// </summary>
public interface IBusinessRule<in TContext>
{
    /// <summary>Execution order priority (lower numbers execute first).</summary>
    int Priority => 0;

    /// <summary>Evaluates the rule against the given context and request metadata.</summary>
    ValueTask<RuleResult> EvaluateAsync(TContext context, RequestContext requestContext, CancellationToken cancellationToken = default);
}

/// <summary>
/// Represents the outcome of evaluating a business rule.
/// </summary>
public readonly record struct RuleResult(bool IsSuccess, string? ErrorCode = null, string? Message = null)
{
    public static RuleResult Success() => new(true);
    public static RuleResult Failed(string errorCode, string message) => new(false, errorCode, message);
}

/// <summary>
/// Exception thrown when one or more business rules fail.
/// </summary>
public sealed class BusinessRuleViolationException : InvalidOperationException
{
    public string ErrorCode { get; }

    public BusinessRuleViolationException(string errorCode, string message) : base(message)
    {
        ErrorCode = ContractGuard.Text(errorCode, nameof(errorCode));
    }

    public ErrorEnvelope ToErrorEnvelope(RequestContext context, bool retryable = false) =>
        ErrorEnvelope.From(context, ErrorCode, Message, retryable);
}

/// <summary>
/// Dispatcher pipeline behavior that executes all registered IBusinessRule implementations in priority order.
/// Halts execution immediately on the first failed rule (short-circuit).
/// </summary>
public sealed class BusinessRulePipelineBehavior<TRequest, TResponse> : IPipelineBehavior<TRequest, TResponse>
    where TRequest : IRequest<TResponse>
{
    private readonly IBusinessRule<TRequest>[] rules;

    public BusinessRulePipelineBehavior(IEnumerable<IBusinessRule<TRequest>> rules)
    {
        ArgumentNullException.ThrowIfNull(rules);
        this.rules = rules.OrderBy(r => r.Priority).ToArray();
    }

    public async ValueTask<TResponse> HandleAsync(TRequest request, RequestContext context,
        CancellationToken cancellationToken, RequestHandlerDelegate<TResponse> next)
    {
        cancellationToken.ThrowIfCancellationRequested();

        for (var i = 0; i < rules.Length; i++)
        {
            var rule = rules[i];
            var result = await rule.EvaluateAsync(request, context, cancellationToken);
            if (!result.IsSuccess)
            {
                throw new BusinessRuleViolationException(
                    result.ErrorCode ?? "RULE_VIOLATION",
                    result.Message ?? "Business rule violation.");
            }
        }

        return await next();
    }
}
