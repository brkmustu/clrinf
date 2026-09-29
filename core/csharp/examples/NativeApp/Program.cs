using System.Threading;
using Microsoft.AspNetCore.Builder;
using Microsoft.Extensions.DependencyInjection;
using Microsoft.Extensions.Logging;
using NativeApp.Common;
using NativeApp.Features.Benchmark;

// Calibrated MinThreads to 100 matching max concurrency
ThreadPool.SetMinThreads(100, 100);

var builder = WebApplication.CreateBuilder(args);

// Enable Console logging with Filter to capture error stack traces if 500/exceptions occur
builder.Logging.ClearProviders();
builder.Logging.AddConsole();
builder.Logging.SetMinimumLevel(LogLevel.Error);

builder.Services.AddControllers();
builder.Services.AddEndpointsApiExplorer();

// Register Dispatcher as Scoped (matching MediatR Scoped lifetime in ASP.NET Core)
builder.Services.AddScoped<IDispatcher, Dispatcher>();
builder.Services.AddScoped<ICommandHandler<BenchmarkCommand, string>, BenchmarkCommandHandler>();
builder.Services.AddScoped<IQueryHandler<BenchmarkQuery, string>, BenchmarkQueryHandler>();

// Register 2 PassThrough Behaviors for 1-to-1 parity with MediatRApp
builder.Services.AddScoped<IPipelineBehavior<BenchmarkCommand, string>, PassThroughBehavior1<BenchmarkCommand, string>>();
builder.Services.AddScoped<IPipelineBehavior<BenchmarkCommand, string>, PassThroughBehavior2<BenchmarkCommand, string>>();

builder.Services.AddScoped<IPipelineBehavior<BenchmarkQuery, string>, PassThroughBehavior1<BenchmarkQuery, string>>();
builder.Services.AddScoped<IPipelineBehavior<BenchmarkQuery, string>, PassThroughBehavior2<BenchmarkQuery, string>>();

var app = builder.Build();

app.MapControllers();

app.Run();
