using ClrinfCS.Core;

namespace EcommerceMonolith.Security;

/// <summary>
/// Interface for requests that require explicit Cedar operation claims.
/// </summary>
public interface IRequireOperationClaim
{
    OperationClaim GetRequiredClaim();
}

public sealed class CedarAuthException(string message, bool isTenantViolation) : InvalidOperationException(message)
{
    public bool IsTenantViolation { get; } = isTenantViolation;
}

public static class EcommerceCedarFactory
{
    public static MultiTenantCedarAuthorizer CreateAuthorizer()
    {
        var authorizer = new MultiTenantCedarAuthorizer(platformAdminRole: "PlatformAdmin");

        // 1. Store Admin - full access
        authorizer.AddRule(new CedarPolicyRule { PrincipalRole = "StoreAdmin", ResourceType = "Catalog", EffectPermit = true });
        authorizer.AddRule(new CedarPolicyRule { PrincipalRole = "StoreAdmin", ResourceType = "Inventory", EffectPermit = true });
        authorizer.AddRule(new CedarPolicyRule { PrincipalRole = "StoreAdmin", ResourceType = "Orders", EffectPermit = true });

        // 2. Catalog Manager - catalog & inventory management
        authorizer.AddRule(new CedarPolicyRule { PrincipalRole = "CatalogManager", ResourceType = "Catalog", EffectPermit = true });
        authorizer.AddRule(new CedarPolicyRule { PrincipalRole = "CatalogManager", Action = "inventory.read", ResourceType = "Inventory", EffectPermit = true });

        // 3. Customer - read catalog, manage own orders, inventory reservation
        authorizer.AddRule(new CedarPolicyRule { PrincipalRole = "Customer", Action = "catalog.read", ResourceType = "Catalog", EffectPermit = true });
        authorizer.AddRule(new CedarPolicyRule { PrincipalRole = "Customer", Action = "inventory.reserve", ResourceType = "Inventory", EffectPermit = true });
        authorizer.AddRule(new CedarPolicyRule { PrincipalRole = "Customer", Action = "inventory.release", ResourceType = "Inventory", EffectPermit = true });
        authorizer.AddRule(new CedarPolicyRule { PrincipalRole = "Customer", Action = "inventory.commit", ResourceType = "Inventory", EffectPermit = true });
        authorizer.AddRule(new CedarPolicyRule { PrincipalRole = "Customer", Action = "orders.create", ResourceType = "Orders", EffectPermit = true });
        authorizer.AddRule(new CedarPolicyRule { PrincipalRole = "Customer", Action = "orders.read", ResourceType = "Orders", EffectPermit = true });
        authorizer.AddRule(new CedarPolicyRule { PrincipalRole = "Customer", Action = "orders.cancel", ResourceType = "Orders", EffectPermit = true });

        return authorizer;
    }
}

public sealed class CedarAuthorizationPipelineBehavior<TRequest, TResponse>(
    MultiTenantCedarAuthorizer authorizer,
    AuthzClaims userClaims) : IPipelineBehavior<TRequest, TResponse> where TRequest : IRequest<TResponse>
{
    public async ValueTask<TResponse> HandleAsync(
        TRequest request,
        RequestContext context,
        CancellationToken cancellationToken,
        RequestHandlerDelegate<TResponse> next)
    {
        if (request is IRequireOperationClaim claimProvider)
        {
            var requiredClaim = claimProvider.GetRequiredClaim();
            var decision = await authorizer.AuthorizeAsync(userClaims, [requiredClaim], cancellationToken);
            if (!decision.IsAllowed)
            {
                throw new CedarAuthException(decision.Reason ?? "Access Denied by Cedar Policy", decision.IsTenantViolation);
            }
        }

        return await next();
    }
}
