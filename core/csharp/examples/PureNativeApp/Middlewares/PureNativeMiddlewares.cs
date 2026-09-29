using System.Threading.Tasks;
using Microsoft.AspNetCore.Http;

namespace PureNativeApp.Middlewares;

public class LoggingMiddleware
{
    private readonly RequestDelegate _next;

    public LoggingMiddleware(RequestDelegate next)
    {
        _next = next;
    }

    public Task InvokeAsync(HttpContext context)
    {
        // Pipeline Logging execution
        return _next(context);
    }
}

public class ValidationMiddleware
{
    private readonly RequestDelegate _next;

    public ValidationMiddleware(RequestDelegate next)
    {
        _next = next;
    }

    public Task InvokeAsync(HttpContext context)
    {
        // Pipeline Validation execution
        return _next(context);
    }
}

public class AuthorizationMiddleware
{
    private readonly RequestDelegate _next;

    public AuthorizationMiddleware(RequestDelegate next)
    {
        _next = next;
    }

    public Task InvokeAsync(HttpContext context)
    {
        // Pipeline Authorization execution
        return _next(context);
    }
}

public class PassThroughMiddleware1
{
    private readonly RequestDelegate _next;

    public PassThroughMiddleware1(RequestDelegate next)
    {
        _next = next;
    }

    public Task InvokeAsync(HttpContext context)
    {
        return _next(context);
    }
}

public class PassThroughMiddleware2
{
    private readonly RequestDelegate _next;

    public PassThroughMiddleware2(RequestDelegate next)
    {
        _next = next;
    }

    public Task InvokeAsync(HttpContext context)
    {
        return _next(context);
    }
}
