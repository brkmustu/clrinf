using System.Text;
using Microsoft.AspNetCore.Authentication.JwtBearer;
using Microsoft.EntityFrameworkCore;
using Microsoft.IdentityModel.Tokens;
using CrmMonolith.Data;
using CrmMonolith.Domain.Entities;
using CrmMonolith.Common.Security;
using CrmMonolith.Common.Functional;
using CrmMonolith.Domain.Common.Exceptions;

// Module namespaces
using DealMod = CrmMonolith.Modules.Deal;
using ContactMod = CrmMonolith.Modules.Contact;
using ActivityMod = CrmMonolith.Modules.Activity;

var builder = WebApplication.CreateBuilder(args);

// 1. Database Configuration (InMemory or SQLite)
builder.Services.AddDbContext<CrmDbContext>(options =>
{
    var connStr = builder.Configuration.GetConnectionString("DefaultConnection");
    if (!string.IsNullOrEmpty(connStr))
    {
        options.UseSqlite(connStr);
    }
    else
    {
        options.UseInMemoryDatabase("CrmMonolithDb");
    }
});

// 2. JWT Authentication Configuration
var jwtSection = builder.Configuration.GetSection("TokenOptions");
var tokenOptions = new TokenOptions(
    Issuer: jwtSection.GetValue<string>("Issuer") ?? "CrmMonolith",
    Audience: jwtSection.GetValue<string>("Audience") ?? "CrmMonolithClient",
    SecurityKey: jwtSection.GetValue<string>("SecurityKey") ?? "SuperSecretKeyForCrmMonolithModularApp2026!",
    ExpirationMinutes: jwtSection.GetValue<int?>("ExpirationMinutes") ?? 60
);
builder.Services.AddSingleton(tokenOptions);

builder.Services.AddAuthentication(JwtBearerDefaults.AuthenticationScheme)
    .AddJwtBearer(options =>
    {
        options.TokenValidationParameters = new TokenValidationParameters
        {
            ValidateIssuer = true,
            ValidateAudience = true,
            ValidateLifetime = true,
            ValidateIssuerSigningKey = true,
            ValidIssuer = tokenOptions.Issuer,
            ValidAudience = tokenOptions.Audience,
            IssuerSigningKey = new SymmetricSecurityKey(Encoding.UTF8.GetBytes(tokenOptions.SecurityKey))
        };
    });

builder.Services.AddAuthorization();

// 3. Register Handlers in DI
// Deal Handlers
builder.Services.AddScoped<DealMod.Create.Handler>();
builder.Services.AddScoped<DealMod.Update.Handler>();
builder.Services.AddScoped<DealMod.Delete.Handler>();
builder.Services.AddScoped<DealMod.GetById.Handler>();
builder.Services.AddScoped<DealMod.GetList.Handler>();

// Contact Handlers
builder.Services.AddScoped<ContactMod.Create.Handler>();
builder.Services.AddScoped<ContactMod.Update.Handler>();
builder.Services.AddScoped<ContactMod.Delete.Handler>();
builder.Services.AddScoped<ContactMod.GetById.Handler>();
builder.Services.AddScoped<ContactMod.GetList.Handler>();

// Activity Handlers
builder.Services.AddScoped<ActivityMod.Create.Handler>();
builder.Services.AddScoped<ActivityMod.Update.Handler>();
builder.Services.AddScoped<ActivityMod.Delete.Handler>();
builder.Services.AddScoped<ActivityMod.GetById.Handler>();
builder.Services.AddScoped<ActivityMod.GetList.Handler>();

var app = builder.Build();

// 4. Ensure DB Created & Seed Default Admin
using (var scope = app.Services.CreateScope())
{
    var db = scope.ServiceProvider.GetRequiredService<CrmDbContext>();
    db.Database.EnsureCreated();

    if (!db.Users.Any())
    {
        HashingHelper.CreatePasswordHash("Admin123!", out var hash, out var salt);
        var admin = new User
        {
            FirstName = "System",
            LastName = "Admin",
            Email = "admin@admin.com",
            PasswordHash = hash,
            PasswordSalt = salt,
            Role = "Admin",
            Status = true,
            TenantId = "tenant-crm-demo"
        };
        db.Users.Add(admin);
        db.SaveChanges();
    }
}

app.UseAuthentication();
app.UseAuthorization();

// 5. Minimal API Endpoints

// Auth Endpoints
app.MapPost("/api/auth/login", async (LoginRequest request, CrmDbContext db, TokenOptions opts) =>
{
    var user = await db.Users.FirstOrDefaultAsync(u => u.Email == request.Email);
    if (user is null || !HashingHelper.VerifyPasswordHash(request.Password, user.PasswordHash, user.PasswordSalt))
    {
        return Results.Unauthorized();
    }

    var token = JwtHelper.CreateToken(user.Id, user.Email, user.TenantId, new[] { user.Role }, opts);
    return Results.Ok(token);
});

app.MapPost("/api/auth/register", async (RegisterRequest request, CrmDbContext db, TokenOptions opts) =>
{
    if (await db.Users.AnyAsync(u => u.Email == request.Email))
    {
        return Results.BadRequest(new { error = "User.AlreadyExists", message = "Email is already registered." });
    }

    HashingHelper.CreatePasswordHash(request.Password, out var hash, out var salt);
    var user = new User
    {
        FirstName = request.FirstName,
        LastName = request.LastName,
        Email = request.Email,
        PasswordHash = hash,
        PasswordSalt = salt,
        Role = "User",
        Status = true,
        TenantId = request.TenantId ?? "default"
    };

    db.Users.Add(user);
    await db.SaveChangesAsync();

    var token = JwtHelper.CreateToken(user.Id, user.Email, user.TenantId, new[] { user.Role }, opts);
    return Results.Ok(token);
});

// Deal Endpoints
app.MapPost("/api/deals", async (DealMod.Create.Command cmd, DealMod.Create.Handler handler, CancellationToken ct) =>
{
    var res = await handler.HandleAsync(cmd, ct);
    return res.Match(
        success => Results.Created($"/api/deals/{success.Id}", success),
        error => Results.BadRequest(error)
    );
});

app.MapPut("/api/deals", async (DealMod.Update.Command cmd, DealMod.Update.Handler handler, CancellationToken ct) =>
{
    var res = await handler.HandleAsync(cmd, ct);
    return res.Match(Results.Ok, error => Results.BadRequest(error));
});

app.MapDelete("/api/deals/{id}", async (string id, DealMod.Delete.Handler handler, CancellationToken ct) =>
{
    var res = await handler.HandleAsync(new DealMod.Delete.Command(id), ct);
    return res.Match(Results.Ok, error => Results.NotFound(error));
});

app.MapGet("/api/deals/{id}", async (string id, DealMod.GetById.Handler handler, CancellationToken ct) =>
{
    var res = await handler.HandleAsync(new DealMod.GetById.Query(id), ct);
    return res.Match(Results.Ok, error => Results.NotFound(error));
});

app.MapGet("/api/deals", async (int? page, int? pageSize, DealMod.GetList.Handler handler, CancellationToken ct) =>
{
    var query = new DealMod.GetList.Query(page ?? 1, pageSize ?? 20);
    var res = await handler.HandleAsync(query, ct);
    return res.Match(Results.Ok, error => Results.BadRequest(error));
});

// Contact Endpoints
app.MapPost("/api/contacts", async (ContactMod.Create.Command cmd, ContactMod.Create.Handler handler, CancellationToken ct) =>
{
    var res = await handler.HandleAsync(cmd, ct);
    return res.Match(
        success => Results.Created($"/api/contacts/{success.Id}", success),
        error => Results.BadRequest(error)
    );
});

app.MapPut("/api/contacts", async (ContactMod.Update.Command cmd, ContactMod.Update.Handler handler, CancellationToken ct) =>
{
    var res = await handler.HandleAsync(cmd, ct);
    return res.Match(Results.Ok, error => Results.BadRequest(error));
});

app.MapDelete("/api/contacts/{id}", async (string id, ContactMod.Delete.Handler handler, CancellationToken ct) =>
{
    var res = await handler.HandleAsync(new ContactMod.Delete.Command(id), ct);
    return res.Match(Results.Ok, error => Results.NotFound(error));
});

app.MapGet("/api/contacts/{id}", async (string id, ContactMod.GetById.Handler handler, CancellationToken ct) =>
{
    var res = await handler.HandleAsync(new ContactMod.GetById.Query(id), ct);
    return res.Match(Results.Ok, error => Results.NotFound(error));
});

app.MapGet("/api/contacts", async (int? page, int? pageSize, ContactMod.GetList.Handler handler, CancellationToken ct) =>
{
    var query = new ContactMod.GetList.Query(page ?? 1, pageSize ?? 20);
    var res = await handler.HandleAsync(query, ct);
    return res.Match(Results.Ok, error => Results.BadRequest(error));
});

// Activity Endpoints
app.MapPost("/api/activities", async (ActivityMod.Create.Command cmd, ActivityMod.Create.Handler handler, CancellationToken ct) =>
{
    var res = await handler.HandleAsync(cmd, ct);
    return res.Match(
        success => Results.Created($"/api/activities/{success.Id}", success),
        error => Results.BadRequest(error)
    );
});

app.MapPut("/api/activities", async (ActivityMod.Update.Command cmd, ActivityMod.Update.Handler handler, CancellationToken ct) =>
{
    var res = await handler.HandleAsync(cmd, ct);
    return res.Match(Results.Ok, error => Results.BadRequest(error));
});

app.MapDelete("/api/activities/{id}", async (string id, ActivityMod.Delete.Handler handler, CancellationToken ct) =>
{
    var res = await handler.HandleAsync(new ActivityMod.Delete.Command(id), ct);
    return res.Match(Results.Ok, error => Results.NotFound(error));
});

app.MapGet("/api/activities/{id}", async (string id, ActivityMod.GetById.Handler handler, CancellationToken ct) =>
{
    var res = await handler.HandleAsync(new ActivityMod.GetById.Query(id), ct);
    return res.Match(Results.Ok, error => Results.NotFound(error));
});

app.MapGet("/api/activities", async (int? page, int? pageSize, ActivityMod.GetList.Handler handler, CancellationToken ct) =>
{
    var query = new ActivityMod.GetList.Query(page ?? 1, pageSize ?? 20);
    var res = await handler.HandleAsync(query, ct);
    return res.Match(Results.Ok, error => Results.BadRequest(error));
});

app.Run();

public record LoginRequest(string Email, string Password);
public record RegisterRequest(string FirstName, string LastName, string Email, string Password, string? TenantId = null);
