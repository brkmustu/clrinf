// @clrinf:generated — Multi-Tenant Authorization abstraction for the clrinf C# runtime.
//
// Provides policy-based access control with default multi-tenant isolation,
// Cedar-compatible ABAC/RBAC rule evaluation, and explicit forbid guarantees.

namespace ClrinfCS.Core;

/// <summary>
/// Authenticated principal identity claims for policy evaluation.
/// </summary>
public sealed record AuthzClaims
{
    public string Subject { get; init; }
    public IReadOnlyList<string> Roles { get; init; }
    public string TenantId { get; init; }

    public AuthzClaims(string subject, IReadOnlyList<string> roles, string tenantId)
    {
        Subject = subject ?? throw new ArgumentNullException(nameof(subject));
        Roles = roles ?? Array.Empty<string>();
        TenantId = tenantId ?? throw new ArgumentNullException(nameof(tenantId));
    }
}

/// <summary>
/// Authorization decision with detailed auditing context.
/// </summary>
public sealed record AuthzDecision
{
    public bool IsAllowed { get; init; }
    public bool IsTenantViolation { get; init; }
    public string? Reason { get; init; }

    public static AuthzDecision Allow(string? reason = null) =>
        new() { IsAllowed = true, IsTenantViolation = false, Reason = reason };

    public static AuthzDecision Deny(string reason) =>
        new() { IsAllowed = false, IsTenantViolation = false, Reason = reason };

    public static AuthzDecision TenantViolation(string principalTenant, string resourceTenant) =>
        new()
        {
            IsAllowed = false,
            IsTenantViolation = true,
            Reason = $"Tenant isolation violation: principal tenant '{principalTenant}' cannot access resource tenant '{resourceTenant}'"
        };
}

/// <summary>
/// An operation claim required for a specific action (e.g., "orders.create"),
/// carrying an optional resource tenant for cross-tenant isolation enforcement.
/// </summary>
public sealed record OperationClaim
{
    public string Resource { get; init; }
    public string Action { get; init; }
    public string? ResourceTenantId { get; init; }

    public OperationClaim(string resource, string action, string? resourceTenantId = null)
    {
        Resource = resource ?? throw new ArgumentNullException(nameof(resource));
        Action = action ?? throw new ArgumentNullException(nameof(action));
        ResourceTenantId = resourceTenantId;
    }

    public static OperationClaim WithTenant(string resource, string action, string tenantId) =>
        new(resource, action, tenantId);
}

/// <summary>
/// Cedar-compatible policy rule representation.
/// </summary>
public sealed record CedarPolicyRule
{
    public bool EffectPermit { get; init; } = true;
    public string? PrincipalRole { get; init; }
    public string? Action { get; init; }
    public string? ResourceType { get; init; }
    public bool RequireTenantMatch { get; init; } = true;

    public bool Matches(IReadOnlyList<string> roles, string action, string resource)
    {
        var roleMatch = PrincipalRole is null || roles.Contains(PrincipalRole);
        var actionMatch = Action is null || Action == "*" || Action.Equals(action, StringComparison.OrdinalIgnoreCase);
        var resMatch = ResourceType is null || ResourceType == "*" || ResourceType.Equals(resource, StringComparison.OrdinalIgnoreCase);

        return roleMatch && actionMatch && resMatch;
    }
}

/// <summary>
/// Authorization service contract for evaluating access policies.
/// </summary>
public interface IAuthorizationService
{
    ValueTask<AuthzDecision> AuthorizeAsync(
        AuthzClaims claims,
        IReadOnlyList<OperationClaim> required,
        CancellationToken cancellationToken = default);
}

/// <summary>
/// Default Multi-Tenant Cedar Policy Engine for C# / .NET.
/// Guarantees:
/// 1. Tenant Isolation by default: Cross-tenant access is strictly denied (Forbid),
///    unless the principal has the PlatformAdmin role.
/// 2. Role-based permit checks aligned with Cedar policies.
/// 3. Cedar Forbid rules always override permit rules.
/// </summary>
public sealed class MultiTenantCedarAuthorizer : IAuthorizationService
{
    private readonly string _platformAdminRole;
    private readonly List<CedarPolicyRule> _rules = new();

    public MultiTenantCedarAuthorizer(string platformAdminRole = "PlatformAdmin")
    {
        _platformAdminRole = platformAdminRole;
    }

    public MultiTenantCedarAuthorizer AddRule(CedarPolicyRule rule)
    {
        ArgumentNullException.ThrowIfNull(rule);
        _rules.Add(rule);
        return this;
    }

    public ValueTask<AuthzDecision> AuthorizeAsync(
        AuthzClaims claims,
        IReadOnlyList<OperationClaim> required,
        CancellationToken cancellationToken = default)
    {
        ArgumentNullException.ThrowIfNull(claims);
        ArgumentNullException.ThrowIfNull(required);

        bool isPlatformAdmin = claims.Roles.Contains(_platformAdminRole);

        foreach (var op in required)
        {
            // 1. Multi-Tenant İzolasyon Denetimi (Default Tenant Guard)
            if (!string.IsNullOrEmpty(op.ResourceTenantId) &&
                !op.ResourceTenantId.Equals(claims.TenantId, StringComparison.Ordinal) &&
                !isPlatformAdmin)
            {
                return ValueTask.FromResult(AuthzDecision.TenantViolation(claims.TenantId, op.ResourceTenantId));
            }

            // PlatformAdmin tüm izin kurallarını bypass eder
            if (isPlatformAdmin)
            {
                continue;
            }

            // 2. Cedar Forbid Kuralları Kontrolü (Forbid her zaman Permit'i ezer)
            bool forbidTriggered = _rules
                .Where(r => !r.EffectPermit)
                .Any(r => r.Matches(claims.Roles, op.Action, op.Resource));

            if (forbidTriggered)
            {
                return ValueTask.FromResult(AuthzDecision.Deny(
                    $"Explicit Cedar FORBID triggered for {op.Resource}:{op.Action}"));
            }

            // 3. İzin Kontrolü (TenantAdmin, spesifik rol veya Cedar Permit kuralı)
            bool isTenantAdmin = claims.Roles.Contains("TenantAdmin") || claims.Roles.Contains("Admin");
            string expectedPermission = $"{op.Resource}.{op.Action}";
            bool hasRolePermission = claims.Roles.Contains(expectedPermission) || claims.Roles.Contains(op.Resource);

            if (!isTenantAdmin && !hasRolePermission)
            {
                bool permittedByRule = _rules
                    .Where(r => r.EffectPermit)
                    .Any(r => r.Matches(claims.Roles, op.Action, op.Resource));

                if (!permittedByRule)
                {
                    return ValueTask.FromResult(AuthzDecision.Deny(
                        $"Insufficient permissions for operation: {expectedPermission}"));
                }
            }
        }

        return ValueTask.FromResult(AuthzDecision.Allow());
    }
}
