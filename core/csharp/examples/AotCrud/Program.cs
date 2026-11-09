using AotCrud.Domain;
using AotCrud.Domain.Rules;
using AotCrud.Infrastructure;
using ClrinfCS.Core;
using ClrinfCS.Generated;
using ClrinfCS.OpenApi;

var builder = WebApplication.CreateBuilder(args);

// Demo services. Replace the stores with ClrinfCS.Adapters.EfCore.EfCrudStore.
builder.Services.AddSingleton<IProductCatalogService, DemoProductCatalog>();
builder.Services.AddSingleton<IBeforeDeleteRule<Category>, ActiveProductsCheckRule>();

builder.Services.AddSingleton<ICrudStore<Category, int>>(
    new InMemoryCrudStore<Category, int>(
        static c => c.Id,
        static (c, tenant) => c.TenantId = tenant,
        static c => c.TenantId));
builder.Services.AddSingleton<ICrudStore<Brand, int>>(
    new InMemoryCrudStore<Brand, int>(
        static b => b.Id,
        static (b, tenant) => b.TenantId = tenant,
        static b => b.TenantId));

builder.Services.AddGeneratedCrud();
builder.Services.AddClrinfOpenApi(options =>
{
    options.Title = "AotCrud API";
    options.Version = "1.0.0";
    options.Description = "Compile-time generated AOT CRUD endpoints.";
});

var app = builder.Build();

app.MapGet("/health", (RequestDelegate)(static http =>
{
    http.Response.ContentType = "application/json; charset=utf-8";
    return http.Response.WriteAsync("{\"status\":\"ok\"}", http.RequestAborted);
}));

// Seed one category so the delete hook is easy to demonstrate (id=1 has active products).
var seedContext = new RequestContext("demo", "seed", "seed");
await app.Services.GetRequiredService<ICrudStore<Category, int>>()
    .AddAsync(new Category { Id = 1, Name = "Electronics", Slug = "electronics" }, seedContext);

// Demo only: derive the tenant from a trusted authentication boundary in production.
app.MapGeneratedCrudEndpoints(http =>
{
    var tenant = http.Request.Headers.TryGetValue("X-Tenant-Id", out var value) &&
                 !string.IsNullOrWhiteSpace(value)
        ? value.ToString()
        : "demo";
    return new RequestContext(tenant, http.TraceIdentifier, "http-request");
});
app.MapClrinfOpenApi();

app.Run();
