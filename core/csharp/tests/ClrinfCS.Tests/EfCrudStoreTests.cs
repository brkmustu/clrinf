using System;
using System.Collections.Generic;
using System.Threading.Tasks;
using ClrinfCS.Adapters.EfCore;
using ClrinfCS.Core;
using Microsoft.EntityFrameworkCore;
using Xunit;

namespace ClrinfCS.Tests;

public class EfCrudStoreTests
{
    private sealed class Widget : ITenantScoped
    {
        public int Id { get; set; }
        public string TenantId { get; set; } = string.Empty;
        public string Name { get; set; } = string.Empty;
    }

    private sealed class TestDb(DbContextOptions<TestDb> options) : DbContext(options)
    {
        public DbSet<Widget> Widgets => Set<Widget>();
    }

    private static readonly RequestContext TenantA = new("tenant-a", "corr-a", "cause-a");
    private static readonly RequestContext TenantB = new("tenant-b", "corr-b", "cause-b");

    private static (TestDb Db, EfCrudStore<Widget, int, TestDb> Store) Create()
    {
        var options = new DbContextOptionsBuilder<TestDb>()
            .UseInMemoryDatabase(Guid.NewGuid().ToString())
            .Options;
        var db = new TestDb(options);
        return (db, new EfCrudStore<Widget, int, TestDb>(db));
    }

    [Fact]
    public async Task Add_SetsTenantFromContext_AndGetListScopesByTenant()
    {
        var (db, store) = Create();
        await store.AddAsync(new Widget { Id = 1, Name = "A", TenantId = "spoofed" }, TenantA);
        await store.AddAsync(new Widget { Id = 2, Name = "B", TenantId = "spoofed" }, TenantB);

        var pageA = await store.GetListAsync(new CrudQuery(), TenantA);
        var pageB = await store.GetListAsync(new CrudQuery(), TenantB);

        Assert.Single(pageA.Items);
        Assert.Equal("tenant-a", pageA.Items[0].TenantId);
        Assert.Single(pageB.Items);
        Assert.Equal("tenant-b", pageB.Items[0].TenantId);
        Assert.Null(await store.GetByIdAsync(2, TenantA));
        Assert.NotNull(await store.GetByIdAsync(2, TenantB));

        await db.DisposeAsync();
    }

    [Fact]
    public async Task GetList_SortsAndPages()
    {
        var (db, store) = Create();
        await store.AddAsync(new Widget { Id = 1, Name = "Zed" }, TenantA);
        await store.AddAsync(new Widget { Id = 2, Name = "Alpha" }, TenantA);
        await store.AddAsync(new Widget { Id = 3, Name = "Mike" }, TenantA);

        var ascending = await store.GetListAsync(new CrudQuery(1, 2, "Name"), TenantA);
        var descending = await store.GetListAsync(new CrudQuery(1, 2, "-Name"), TenantA);

        Assert.Equal(new[] { "Alpha", "Mike" }, Names(ascending.Items));
        Assert.Equal(new[] { "Zed", "Mike" }, Names(descending.Items));
        Assert.Equal(3, ascending.TotalCount);
        Assert.Equal(2, ascending.TotalPages);

        await db.DisposeAsync();
    }

    [Fact]
    public async Task Update_And_Delete_RespectTenantScope()
    {
        var (db, store) = Create();
        await store.AddAsync(new Widget { Id = 1, Name = "A" }, TenantA);
        var loaded = await store.GetByIdAsync(1, TenantA);
        Assert.NotNull(loaded);
        loaded!.Name = "Changed";

        await store.UpdateAsync(loaded, TenantA);
        Assert.Equal("Changed", (await store.GetByIdAsync(1, TenantA))!.Name);

        Assert.True(await store.DeleteAsync(loaded, TenantA));
        Assert.Null(await store.GetByIdAsync(1, TenantA));

        await db.DisposeAsync();
    }

    private static string[] Names(IReadOnlyList<Widget> items)
    {
        var names = new string[items.Count];
        for (var i = 0; i < items.Count; i++) names[i] = items[i].Name;
        return names;
    }
}
