using System.Threading;
using MediatR;
using MediatRApp.Features.Benchmark;
using Microsoft.AspNetCore.Builder;
using Microsoft.Extensions.DependencyInjection;
using Microsoft.Extensions.Logging;

// Calibrated MinThreads to 100 matching max concurrency
ThreadPool.SetMinThreads(100, 100);

var builder = WebApplication.CreateBuilder(args);

// Enable Console logging with Filter to capture error stack traces if 500/exceptions occur
builder.Logging.ClearProviders();
builder.Logging.AddConsole();
builder.Logging.SetMinimumLevel(LogLevel.Error);

builder.Services.AddControllers();
builder.Services.AddEndpointsApiExplorer();

// Register MediatR & Behaviors
builder.Services.AddMediatR(cfg =>
{
    cfg.RegisterServicesFromAssembly(typeof(Program).Assembly);
    cfg.AddBehavior(typeof(IPipelineBehavior<,>), typeof(PassThroughBehavior1<,>));
    cfg.AddBehavior(typeof(IPipelineBehavior<,>), typeof(PassThroughBehavior2<,>));
});

var app = builder.Build();

app.UseAuthorization();
app.MapControllers();

app.Run();
