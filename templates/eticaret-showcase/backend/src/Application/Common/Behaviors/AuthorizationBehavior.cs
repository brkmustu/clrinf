using System;
using System.Linq;
using System.Net.Http;
using System.Net.Http.Json;
using System.Reflection;
using System.Security.Claims;
using System.Text.Json;
using System.Threading;
using System.Threading.Tasks;
using Microsoft.AspNetCore.Http;
using Microsoft.Extensions.Configuration;
using EticaretApp.Application.Common.Attributes;
using EticaretApp.Application.Common.Exceptions;
using EticaretApp.Application.Common.Pipeline;

namespace EticaretApp.Application.Common.Behaviors;

public sealed class AuthorizationBehavior<TRequest, TResponse> : IPipelineBehavior<TRequest, TResponse>
    where TRequest : notnull
{
    private readonly IHttpContextAccessor _httpContextAccessor;
    private readonly IConfiguration _configuration;
    private static readonly HttpClient _httpClient = new();

    public AuthorizationBehavior(IHttpContextAccessor httpContextAccessor, IConfiguration configuration)
    {
        _httpContextAccessor = httpContextAccessor;
        _configuration = configuration;
    }

    public async ValueTask<TResponse> HandleAsync(
        TRequest request,
        CancellationToken cancellationToken,
        RequestHandlerDelegate<TResponse> next)
    {
        var httpContext = _httpContextAccessor.HttpContext;
        string? authHeader = httpContext?.Request.Headers["Authorization"].FirstOrDefault();
        string? token = authHeader?.StartsWith("Bearer ", StringComparison.OrdinalIgnoreCase) == true
            ? authHeader["Bearer ".Length..].Trim()
            : null;

        // 1. Check if request implements ISecuredRequest or has RequireAuthorizationAttribute
        bool isSecured = request is ISecuredRequest || typeof(TRequest).GetCustomAttribute<RequireAuthorizationAttribute>() != null;
        bool isQuery = typeof(TRequest).Name.EndsWith("Query") || typeof(TRequest).Name.StartsWith("Get");

        if (isSecured && !isQuery)
        {
            if (string.IsNullOrEmpty(token) && httpContext?.User?.Identity?.IsAuthenticated != true)
            {
                throw new BusinessException("Kimlik doğrulaması gereklidir. Lütfen geçerli bir Bearer Token sağlayın.");
            }

            string authServiceUrl = _configuration["AUTH_SERVICE_URL"] 
                ?? _configuration["AuthService:Url"] 
                ?? Environment.GetEnvironmentVariable("AUTH_SERVICE_URL") 
                ?? "http://localhost:8081";

            // 2. Query Rust Cedar ABAC Policy Engine
            if (!string.IsNullOrEmpty(token) && !string.IsNullOrEmpty(authServiceUrl))
            {
                string resourceName = typeof(TRequest).Name.Replace("Command", "").Replace("Query", "");
                string actionName = typeof(TRequest).Name;

                try
                {
                    var authRequest = new
                    {
                        token = token,
                        action = actionName,
                        resource = resourceName
                    };

                    using var response = await _httpClient.PostAsJsonAsync($"{authServiceUrl}/auth/authorize", authRequest, cancellationToken);
                    if (response.IsSuccessStatusCode)
                    {
                        var result = await response.Content.ReadFromJsonAsync<CedarAuthResponse>(cancellationToken: cancellationToken);
                        if (result?.Allowed != true)
                        {
                            throw new BusinessException($"Erişim Reddedildi: Rust Cedar ABAC politikası bu eyleme ('{actionName}' on '{resourceName}') izin vermedi.");
                        }
                    }
                }
                catch (BusinessException)
                {
                    throw;
                }
                catch (Exception)
                {
                    // Fallback to local role check if auth-service is unreachable during offline dev
                    EvaluateLocalClaims(request);
                }
            }
            else
            {
                EvaluateLocalClaims(request);
            }
        }

        return await next();
    }

    private void EvaluateLocalClaims(TRequest request)
    {
        var user = _httpContextAccessor.HttpContext?.User;
        if (user?.Identity?.IsAuthenticated != true)
            return;

        if (request is ISecuredRequest securedRequest && securedRequest.Roles?.Length > 0)
        {
            bool isAuthorized = user.IsInRole("Admin") || user.HasClaim(ClaimTypes.Role, "Admin") ||
                securedRequest.Roles.Any(role => user.IsInRole(role) || user.HasClaim(ClaimTypes.Role, role));

            if (!isAuthorized)
            {
                throw new BusinessException("Bu işlem için yetkiniz bulunmamaktadır.");
            }
        }
    }

    private record CedarAuthResponse
    {
        public bool Allowed { get; init; }
        public string? TenantId { get; init; }
        public string? Principal { get; init; }
        public string? Resource { get; init; }
        public string? Action { get; init; }
    }
}
