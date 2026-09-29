using Microsoft.EntityFrameworkCore;
using CrmMonolith.Data;
using CrmMonolith.Domain.Entities;
using CrmMonolith.Common.Security;
using DealMod = CrmMonolith.Modules.Deal;
using ContactMod = CrmMonolith.Modules.Contact;
using ActivityMod = CrmMonolith.Modules.Activity;
using Xunit;

namespace CrmMonolith.Tests;

public class CrmMonolithTests
{
    private static CrmDbContext CreateDbContext()
    {
        var options = new DbContextOptionsBuilder<CrmDbContext>()
            .UseInMemoryDatabase(databaseName: Guid.NewGuid().ToString())
            .Options;
        return new CrmDbContext(options);
    }

    [Fact]
    public async Task Deal_Create_And_GetById_Should_Succeed()
    {
        using var db = CreateDbContext();
        var createHandler = new DealMod.Create.Handler(db);
        var getByIdHandler = new DealMod.GetById.Handler(db);

        // 1. Create Deal
        var createCmd = new DealMod.Create.Command(
            Title: "Enterprise Software License",
            TenantId: "tenant-100",
            ContactId: "contact-1",
            Amount: 150000m,
            Stage: "Proposal",
            Probability: 75
        );

        var createResult = await createHandler.HandleAsync(createCmd, default);
        Assert.True(createResult.IsSuccess);
        Assert.NotNull(createResult.Value.Id);

        // 2. Query Deal by ID
        var getResult = await getByIdHandler.HandleAsync(new DealMod.GetById.Query(createResult.Value.Id), default);
        Assert.True(getResult.IsSuccess);
        Assert.Equal("Enterprise Software License", getResult.Value.Title);
        Assert.Equal(150000m, getResult.Value.Amount);
        Assert.Equal("Proposal", getResult.Value.Stage);
    }

    [Fact]
    public async Task Deal_Update_And_Delete_Should_Succeed()
    {
        using var db = CreateDbContext();
        var createHandler = new DealMod.Create.Handler(db);
        var updateHandler = new DealMod.Update.Handler(db);
        var deleteHandler = new DealMod.Delete.Handler(db);
        var getByIdHandler = new DealMod.GetById.Handler(db);

        var createResult = await createHandler.HandleAsync(new DealMod.Create.Command(
            Title: "Initial Deal",
            TenantId: "tenant-100",
            ContactId: "contact-1",
            Amount: 50000m,
            Stage: "Lead",
            Probability: 20
        ), default);
        var dealId = createResult.Value.Id;

        // Update
        var updateResult = await updateHandler.HandleAsync(new DealMod.Update.Command(
            Id: dealId,
            Title: "Won Deal",
            TenantId: "tenant-100",
            ContactId: "contact-1",
            Amount: 60000m,
            Stage: "ClosedWon",
            Probability: 100
        ), default);
        Assert.True(updateResult.IsSuccess);

        // Delete
        var deleteResult = await deleteHandler.HandleAsync(new DealMod.Delete.Command(dealId), default);
        Assert.True(deleteResult.IsSuccess);

        // Query after delete should return DomainError.NotFound
        var getResult = await getByIdHandler.HandleAsync(new DealMod.GetById.Query(dealId), default);
        Assert.True(getResult.IsFailure);
        Assert.Equal("Deal.NotFound", getResult.Error.Code);
    }

    [Fact]
    public async Task Contact_And_Activity_Flow_Should_Succeed()
    {
        using var db = CreateDbContext();
        var contactHandler = new ContactMod.Create.Handler(db);
        var activityHandler = new ActivityMod.Create.Handler(db);

        // Create Contact
        var contactRes = await contactHandler.HandleAsync(new ContactMod.Create.Command(
            TenantId: "tenant-100",
            FullName: "Jane Doe",
            Email: "jane.doe@example.com",
            Phone: "+905551234567",
            Company: "Acme Corp"
        ), default);
        Assert.True(contactRes.IsSuccess);

        // Create Activity
        var activityRes = await activityHandler.HandleAsync(new ActivityMod.Create.Command(
            TenantId: "tenant-100",
            Title: "Discovery Call",
            Type: "Call",
            DueDate: DateTime.UtcNow.AddDays(1),
            Notes: "Discuss requirements and pricing",
            DealId: null,
            ContactId: contactRes.Value.Id
        ), default);
        Assert.True(activityRes.IsSuccess);
    }

    [Fact]
    public void Security_PasswordHashing_And_JwtToken_Should_Succeed()
    {
        const string password = "TestUserPassword123!";
        HashingHelper.CreatePasswordHash(password, out var hash, out var salt);
        Assert.True(HashingHelper.VerifyPasswordHash(password, hash, salt));
        Assert.False(HashingHelper.VerifyPasswordHash("WrongPassword", hash, salt));

        var opts = new TokenOptions("Issuer", "Audience", "SuperSecretKeyForTestingPurposesOnly32Chars!", 60);
        var token = JwtHelper.CreateToken("user-1", "user@test.com", "tenant-1", new[] { "Admin", "User" }, opts);
        Assert.NotNull(token.Token);
        Assert.True(token.Expiration > DateTime.UtcNow);
    }
}
