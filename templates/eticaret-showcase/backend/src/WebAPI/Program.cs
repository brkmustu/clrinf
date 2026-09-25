using System.Text;
using Microsoft.AspNetCore.Authentication.JwtBearer;
using Microsoft.AspNetCore.Builder;
using Microsoft.EntityFrameworkCore;
using Microsoft.Extensions.Configuration;
using Microsoft.Extensions.DependencyInjection;
using Microsoft.Extensions.Logging;
using Microsoft.IdentityModel.Tokens;
using EticaretApp.Application;
using EticaretApp.Infrastructure;
using EticaretApp.Infrastructure.Adapters.Graylog;
using EticaretApp.Infrastructure.Security.JWT;
using EticaretApp.Persistence;
using EticaretApp.Persistence.Contexts;
using clrinf.OpenApi;

var builder = WebApplication.CreateBuilder(args);

// Native Graylog UDP Logging (Zero-Dependency)
var graylogHost = builder.Configuration.GetValue<string>("Graylog:Host") ?? "127.0.0.1";
var graylogPort = builder.Configuration.GetValue<int?>("Graylog:Port") ?? 12201;

builder.Logging.ClearProviders();
builder.Logging.AddConsole();
builder.Logging.AddGraylog(options =>
{
    options.Host = graylogHost;
    options.Port = graylogPort;
    options.ApplicationName = builder.Environment.ApplicationName;
});

// Add services to the container.
builder.Services.AddControllers();
builder.Services.AddClrinfOpenApi();

builder.Services.AddCors(options =>
{
    options.AddDefaultPolicy(policy =>
    {
        policy.AllowAnyOrigin()
              .AllowAnyHeader()
              .AllowAnyMethod();
    });
});

builder.Services.AddApplicationServices();
builder.Services.AddPersistenceServices(builder.Configuration);
builder.Services.AddInfrastructureServices();

// JWT Authentication Service Registration
var tokenOptions = builder.Configuration.GetSection("TokenOptions").Get<TokenOptions>();
builder.Services.AddAuthentication(options =>
{
    options.DefaultAuthenticateScheme = JwtBearerDefaults.AuthenticationScheme;
    options.DefaultChallengeScheme = JwtBearerDefaults.AuthenticationScheme;
})
.AddJwtBearer(options =>
{
    options.TokenValidationParameters = new TokenValidationParameters
    {
        ValidateIssuer = true,
        ValidateAudience = true,
        ValidateLifetime = true,
        ValidIssuer = tokenOptions?.Issuer,
        ValidAudience = tokenOptions?.Audience,
        ValidateIssuerSigningKey = true,
        IssuerSigningKey = new SymmetricSecurityKey(Encoding.UTF8.GetBytes(tokenOptions?.SecurityKey ?? "clrinfcs_default_jwt_super_secret_security_key_2026_must_be_at_least_512_bits_long!"))
    };
});

var app = builder.Build();

// Enable Database Schema Check (Migrations & Seeding managed by MigratorApp)
using (var scope = app.Services.CreateScope())
{
    try
    {
        var dbContext = scope.ServiceProvider.GetRequiredService<BaseDbContext>();
        dbContext.Database.EnsureCreated();
    }
    catch (Exception ex)
    {
        Console.WriteLine($"[INFO] Database status check: {ex.Message}");
    }
}

// Enable Static Files & OpenAPI Specification
app.UseDefaultFiles();
app.UseStaticFiles();
app.UseCors();
app.MapClrinfOpenApi();

app.UseAuthentication();
app.UseAuthorization();
app.MapControllers();

app.Run();
