using System.Threading;
using Microsoft.AspNetCore.Builder;
using Microsoft.Extensions.DependencyInjection;
using Microsoft.Extensions.Logging;
using PureNativeApp.Common;
using PureNativeApp.Features.Benchmark;
using PureNativeApp.Middlewares;

// Calibrated MinThreads to 100 matching max concurrency
ThreadPool.SetMinThreads(100, 100);

var builder = WebApplication.CreateBuilder(args);

// Enable Console logging with Filter to capture error stack traces if 500/exceptions occur
builder.Logging.ClearProviders();
builder.Logging.AddConsole();
builder.Logging.SetMinimumLevel(LogLevel.Error);

builder.Services.AddControllers();
builder.Services.AddEndpointsApiExplorer();

// Direct CQRS Handlers registration (No Dispatcher / No Mediator)
builder.Services.AddScoped<ICommandHandler<BenchmarkCommand, string>, BenchmarkCommandHandler>();
builder.Services.AddScoped<IQueryHandler<BenchmarkQuery, string>, BenchmarkQueryHandler>();

var app = builder.Build();

// Cross-cutting concerns handled at ASP.NET Core Middleware Pipeline level
app.UseMiddleware<LoggingMiddleware>();
app.UseMiddleware<ValidationMiddleware>();
app.UseMiddleware<AuthorizationMiddleware>();

// PassThrough Middlewares for 1-to-1 parity with NativeApp & MediatRApp behaviors
app.UseMiddleware<PassThroughMiddleware1>();
app.UseMiddleware<PassThroughMiddleware2>();

app.MapControllers();

app.Run();
