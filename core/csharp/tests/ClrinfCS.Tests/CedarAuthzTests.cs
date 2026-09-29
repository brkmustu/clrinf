using ClrinfCS.Core;
using Xunit;

namespace ClrinfCS.Tests;

public class CedarAuthzTests
{
    [Fact]
    public async Task PermitsOperation_WhenPrincipalRoleAndTenantMatch()
    {
        var authorizer = new MultiTenantCedarAuthorizer();
        authorizer.AddRule(new CedarPolicyRule
        {
            EffectPermit = true,
            PrincipalRole = "OrdersManager",
            Action = "orders.create",
            ResourceType = "orders"
        });

        var claims = new AuthzClaims("user_1", new[] { "OrdersManager" }, "tenant_a");
        var required = new[] { OperationClaim.WithTenant("orders", "orders.create", "tenant_a") };

        var decision = await authorizer.AuthorizeAsync(claims, required);

        Assert.True(decision.IsAllowed);
        Assert.False(decision.IsTenantViolation);
    }

    [Fact]
    public async Task DeniesOperation_WhenCrossTenantViolationOccurs()
    {
        var authorizer = new MultiTenantCedarAuthorizer();
        authorizer.AddRule(new CedarPolicyRule
        {
            EffectPermit = true,
            PrincipalRole = "OrdersManager",
            Action = "orders.create",
            ResourceType = "orders"
        });

        // user_1 belongs to tenant_a, attempting to access resource in tenant_b
        var claims = new AuthzClaims("user_1", new[] { "OrdersManager" }, "tenant_a");
        var required = new[] { OperationClaim.WithTenant("orders", "orders.create", "tenant_b") };

        var decision = await authorizer.AuthorizeAsync(claims, required);

        Assert.False(decision.IsAllowed);
        Assert.True(decision.IsTenantViolation);
        Assert.Contains("Tenant isolation violation", decision.Reason);
    }

    [Fact]
    public async Task AllowsPlatformAdmin_ToBypassTenantIsolation()
    {
        var authorizer = new MultiTenantCedarAuthorizer();
        // PlatformAdmin should bypass tenant boundary and permit rules
        var claims = new AuthzClaims("admin_global", new[] { "PlatformAdmin" }, "system_tenant");
        var required = new[] { OperationClaim.WithTenant("orders", "orders.delete", "tenant_b") };

        var decision = await authorizer.AuthorizeAsync(claims, required);

        Assert.True(decision.IsAllowed);
        Assert.False(decision.IsTenantViolation);
    }

    [Fact]
    public async Task EnforcesExplicitCedarForbidOverride_EvenIfPermittedByRole()
    {
        var authorizer = new MultiTenantCedarAuthorizer();
        // Permit rule
        authorizer.AddRule(new CedarPolicyRule
        {
            EffectPermit = true,
            PrincipalRole = "OrdersManager",
            Action = "orders.delete",
            ResourceType = "orders"
        });
        // Forbid rule overriding delete for all users with BlockedUser role
        authorizer.AddRule(new CedarPolicyRule
        {
            EffectPermit = false,
            PrincipalRole = "BlockedUser",
            Action = "orders.delete",
            ResourceType = "orders"
        });

        // User has both OrdersManager (permit) and BlockedUser (forbid)
        var claims = new AuthzClaims("user_bad", new[] { "OrdersManager", "BlockedUser" }, "tenant_a");
        var required = new[] { OperationClaim.WithTenant("orders", "orders.delete", "tenant_a") };

        var decision = await authorizer.AuthorizeAsync(claims, required);

        Assert.False(decision.IsAllowed);
        Assert.False(decision.IsTenantViolation);
        Assert.Contains("FORBID", decision.Reason);
    }

    [Fact]
    public async Task AllowsTenantAdmin_FullAccessWithinOwnTenant()
    {
        var authorizer = new MultiTenantCedarAuthorizer();
        // No explicit rules needed for TenantAdmin
        var claims = new AuthzClaims("admin_local", new[] { "TenantAdmin" }, "tenant_a");
        var required = new[] { OperationClaim.WithTenant("orders", "orders.custom_action", "tenant_a") };

        var decision = await authorizer.AuthorizeAsync(claims, required);

        Assert.True(decision.IsAllowed);
    }
}
