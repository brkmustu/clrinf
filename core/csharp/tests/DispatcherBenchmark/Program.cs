using System;
using System.Collections.Concurrent;
using System.Collections.Generic;
using System.Diagnostics;
using System.Linq;
using System.Threading;
using System.Threading.Tasks;
using BenchmarkDotNet.Attributes;
using BenchmarkDotNet.Engines;
using BenchmarkDotNet.Jobs;
using BenchmarkDotNet.Running;
using MediatR;
using Microsoft.Extensions.DependencyInjection;

namespace DispatcherBenchmark;

public class Program
{
    public static async Task Main(string[] args)
    {
        Console.OutputEncoding = System.Text.Encoding.UTF8;
        Console.WriteLine("==========================================================================");
        Console.WriteLine(" CQRS DISPATCHER PERFORMANCE & STRESS TEST SUITE");
        Console.WriteLine(" SampleApp (Native .NET 10 Pipeline) vs MediatR v12.4.1 vs Direct Typed");
        Console.WriteLine("==========================================================================");
        Console.WriteLine();

        if (args.Length > 0 && args[0].Equals("--benchmark", StringComparison.OrdinalIgnoreCase))
        {
            Console.WriteLine("[BİLGİ] BenchmarkDotNet Mikro-Benchmark çalıştırılıyor...");
            BenchmarkRunner.Run<DispatcherComparisonBenchmark>();
            return;
        }

        if (args.Length > 0 && args[0].Equals("--benchmark-batch", StringComparison.OrdinalIgnoreCase))
        {
            Console.WriteLine("[BİLGİ] BenchmarkDotNet [Params] Toplu Stres Benchmark'ı (1K - 1M) çalıştırılıyor...");
            BenchmarkRunner.Run<DispatcherBatchBenchmark>();
            return;
        }

        if (args.Length > 0 && args[0].Equals("--benchmark-scoped", StringComparison.OrdinalIgnoreCase))
        {
            Console.WriteLine("[BİLGİ] BenchmarkDotNet ASP.NET Core Scoped HTTP Request Benchmark'ı çalıştırılıyor...");
            BenchmarkRunner.Run<DispatcherScopedBenchmark>();
            return;
        }

        // Run multi-repeated randomized stress runner (5 repeats per count, Median + StdDev)
        int[] iterationsList = new[] { 1_000, 10_000, 100_000, 1_000_000 };
        var runner = new StressTestRunner();

        await runner.WarmupAsync();

        foreach (var count in iterationsList)
        {
            await runner.RunStressTestRepeatedAsync(count, repeats: 5);
        }

        Console.WriteLine("\n[BİLGİ] BenchmarkDotNet parametreleri:");
        Console.WriteLine("  --benchmark        : Nanoseniye seviyesinde mikro-benchmark");
        Console.WriteLine("  --benchmark-batch  : 1K - 1M arası BenchmarkDotNet [Params] toplu stres testi");
        Console.WriteLine("  --benchmark-scoped : Gerçek dünya ASP.NET Core IServiceScope HTTP istek benchmark'ı");
    }
}

// -------------------------------------------------------------
// SampleApp CQRS Abstractions & Handlers
// -------------------------------------------------------------
public record SampleAppCommand(string Name) : NativeApp.Common.ICommand<string>;
public record SampleAppQuery(int Id) : NativeApp.Common.IQuery<string>;

public class SampleAppCommandHandler : NativeApp.Common.ICommandHandler<SampleAppCommand, string>
{
    public ValueTask<string> HandleAsync(SampleAppCommand command, CancellationToken cancellationToken = default)
    {
        return ValueTask.FromResult($"Handled SampleApp Command: {command.Name}");
    }
}

public class SampleAppQueryHandler : NativeApp.Common.IQueryHandler<SampleAppQuery, string>
{
    public ValueTask<string> HandleAsync(SampleAppQuery query, CancellationToken cancellationToken = default)
    {
        return ValueTask.FromResult($"Handled SampleApp Query: {query.Id}");
    }
}

public class PassThroughBehavior1<TRequest, TResponse> : NativeApp.Common.IPipelineBehavior<TRequest, TResponse> where TRequest : class
{
    public ValueTask<TResponse> HandleAsync(TRequest request, CancellationToken cancellationToken, NativeApp.Common.RequestHandlerDelegate<TResponse> next)
    {
        return next();
    }
}

public class PassThroughBehavior2<TRequest, TResponse> : NativeApp.Common.IPipelineBehavior<TRequest, TResponse> where TRequest : class
{
    public ValueTask<TResponse> HandleAsync(TRequest request, CancellationToken cancellationToken, NativeApp.Common.RequestHandlerDelegate<TResponse> next)
    {
        return next();
    }
}

// -------------------------------------------------------------
// MediatR Abstractions, Behaviors & Handlers
// -------------------------------------------------------------
public record MediatRCommand(string Name) : MediatR.IRequest<string>;
public record MediatRQuery(int Id) : MediatR.IRequest<string>;

public class MediatRCommandHandler : MediatR.IRequestHandler<MediatRCommand, string>
{
    public Task<string> Handle(MediatRCommand request, CancellationToken cancellationToken)
    {
        return Task.FromResult($"Handled MediatR Command: {request.Name}");
    }
}

public class MediatRQueryHandler : MediatR.IRequestHandler<MediatRQuery, string>
{
    public Task<string> Handle(MediatRQuery request, CancellationToken cancellationToken)
    {
        return Task.FromResult($"Handled MediatR Query: {request.Id}");
    }
}

public class MediatRPassThroughBehavior1<TRequest, TResponse> : MediatR.IPipelineBehavior<TRequest, TResponse> where TRequest : notnull
{
    public Task<TResponse> Handle(TRequest request, RequestHandlerDelegate<TResponse> next, CancellationToken cancellationToken)
    {
        return next();
    }
}

public class MediatRPassThroughBehavior2<TRequest, TResponse> : MediatR.IPipelineBehavior<TRequest, TResponse> where TRequest : notnull
{
    public Task<TResponse> Handle(TRequest request, RequestHandlerDelegate<TResponse> next, CancellationToken cancellationToken)
    {
        return next();
    }
}

// -------------------------------------------------------------
// Direct Typed Zero-Reflection Baseline
// -------------------------------------------------------------
public interface IDirectCommand<TResponse> { }
public interface IDirectQuery<TResponse> { }

public interface IDirectCommandHandler<TCommand, TResponse> where TCommand : IDirectCommand<TResponse>
{
    Task<TResponse> HandleAsync(TCommand command, CancellationToken cancellationToken = default);
}

public interface IDirectQueryHandler<TQuery, TResponse> where TQuery : IDirectQuery<TResponse>
{
    Task<TResponse> HandleAsync(TQuery query, CancellationToken cancellationToken = default);
}

public interface IDirectDispatcher
{
    Task<TResponse> SendAsync<TCommand, TResponse>(TCommand command, CancellationToken cancellationToken = default) where TCommand : IDirectCommand<TResponse>;
    Task<TResponse> QueryAsync<TQuery, TResponse>(TQuery query, CancellationToken cancellationToken = default) where TQuery : IDirectQuery<TResponse>;
}

public class DirectTypedDispatcher : IDirectDispatcher
{
    private readonly IServiceProvider _serviceProvider;

    public DirectTypedDispatcher(IServiceProvider serviceProvider) => _serviceProvider = serviceProvider;

    public Task<TResponse> SendAsync<TCommand, TResponse>(TCommand command, CancellationToken cancellationToken = default) where TCommand : IDirectCommand<TResponse>
    {
        var handler = _serviceProvider.GetRequiredService<IDirectCommandHandler<TCommand, TResponse>>();
        return handler.HandleAsync(command, cancellationToken);
    }

    public Task<TResponse> QueryAsync<TQuery, TResponse>(TQuery query, CancellationToken cancellationToken = default) where TQuery : IDirectQuery<TResponse>
    {
        var handler = _serviceProvider.GetRequiredService<IDirectQueryHandler<TQuery, TResponse>>();
        return handler.HandleAsync(query, cancellationToken);
    }
}

public record DirectCommand(string Name) : IDirectCommand<string>;
public record DirectQuery(int Id) : IDirectQuery<string>;

public class DirectCommandHandler : IDirectCommandHandler<DirectCommand, string>
{
    public Task<string> HandleAsync(DirectCommand command, CancellationToken cancellationToken = default)
    {
        return Task.FromResult($"Handled Direct Command: {command.Name}");
    }
}

public class DirectQueryHandler : IDirectQueryHandler<DirectQuery, string>
{
    public Task<string> HandleAsync(DirectQuery query, CancellationToken cancellationToken = default)
    {
        return Task.FromResult($"Handled Direct Query: {query.Id}");
    }
}

// -------------------------------------------------------------
// Multi-Repeated Order-Bias Free Stress Test Runner
// -------------------------------------------------------------
public class StressTestRunner
{
    private readonly NativeApp.Common.IDispatcher _sampleAppNoBehaviors;
    private readonly NativeApp.Common.IDispatcher _sampleAppWithBehaviors;
    private readonly MediatR.IMediator _mediatRNoBehaviors;
    private readonly MediatR.IMediator _mediatRWithBehaviors;
    private readonly IDirectDispatcher _directDispatcher;

    private readonly SampleAppCommand _sampleAppCommand = new("StressTestCommand");
    private readonly SampleAppQuery _sampleAppQuery = new(42);
    private readonly MediatRCommand _mediatRCommand = new("StressTestCommand");
    private readonly MediatRQuery _mediatRQuery = new(42);
    private readonly DirectCommand _directCommand = new("StressTestCommand");
    private readonly DirectQuery _directQuery = new(42);

    public StressTestRunner()
    {
        var s1 = new ServiceCollection();
        s1.AddScoped<NativeApp.Common.IDispatcher, NativeApp.Common.Dispatcher>();
        s1.AddTransient<NativeApp.Common.ICommandHandler<SampleAppCommand, string>, SampleAppCommandHandler>();
        s1.AddTransient<NativeApp.Common.IQueryHandler<SampleAppQuery, string>, SampleAppQueryHandler>();
        _sampleAppNoBehaviors = s1.BuildServiceProvider().GetRequiredService<NativeApp.Common.IDispatcher>();

        var s2 = new ServiceCollection();
        s2.AddScoped<NativeApp.Common.IDispatcher, NativeApp.Common.Dispatcher>();
        s2.AddTransient<NativeApp.Common.ICommandHandler<SampleAppCommand, string>, SampleAppCommandHandler>();
        s2.AddTransient<NativeApp.Common.IQueryHandler<SampleAppQuery, string>, SampleAppQueryHandler>();
        s2.AddTransient(typeof(NativeApp.Common.IPipelineBehavior<,>), typeof(PassThroughBehavior1<,>));
        s2.AddTransient(typeof(NativeApp.Common.IPipelineBehavior<,>), typeof(PassThroughBehavior2<,>));
        _sampleAppWithBehaviors = s2.BuildServiceProvider().GetRequiredService<NativeApp.Common.IDispatcher>();

        var s3 = new ServiceCollection();
        s3.AddMediatR(cfg => cfg.RegisterServicesFromAssembly(typeof(Program).Assembly));
        _mediatRNoBehaviors = s3.BuildServiceProvider().GetRequiredService<MediatR.IMediator>();

        var s4 = new ServiceCollection();
        s4.AddMediatR(cfg =>
        {
            cfg.RegisterServicesFromAssembly(typeof(Program).Assembly);
            cfg.AddBehavior(typeof(MediatR.IPipelineBehavior<,>), typeof(MediatRPassThroughBehavior1<,>));
            cfg.AddBehavior(typeof(MediatR.IPipelineBehavior<,>), typeof(MediatRPassThroughBehavior2<,>));
        });
        _mediatRWithBehaviors = s4.BuildServiceProvider().GetRequiredService<MediatR.IMediator>();

        var s5 = new ServiceCollection();
        s5.AddScoped<IDirectDispatcher, DirectTypedDispatcher>();
        s5.AddTransient<IDirectCommandHandler<DirectCommand, string>, DirectCommandHandler>();
        s5.AddTransient<IDirectQueryHandler<DirectQuery, string>, DirectQueryHandler>();
        _directDispatcher = s5.BuildServiceProvider().GetRequiredService<IDirectDispatcher>();
    }

    public async Task WarmupAsync()
    {
        Console.WriteLine("[ISINMA] Warmup çalıştırılıyor...");
        for (int i = 0; i < 500; i++)
        {
            await _sampleAppNoBehaviors.SendAsync(_sampleAppCommand);
            await _sampleAppNoBehaviors.QueryAsync(_sampleAppQuery);
            await _sampleAppWithBehaviors.SendAsync(_sampleAppCommand);
            await _sampleAppWithBehaviors.QueryAsync(_sampleAppQuery);
            await _mediatRNoBehaviors.Send(_mediatRCommand);
            await _mediatRNoBehaviors.Send(_mediatRQuery);
            await _mediatRWithBehaviors.Send(_mediatRCommand);
            await _mediatRWithBehaviors.Send(_mediatRQuery);
            await _directDispatcher.SendAsync<DirectCommand, string>(_directCommand);
            await _directDispatcher.QueryAsync<DirectQuery, string>(_directQuery);
        }
        Console.WriteLine("[ISINMA] Warmup tamamlandı.\n");
    }

    public async Task RunStressTestRepeatedAsync(int iterations, int repeats = 5)
    {
        Console.WriteLine($"==========================================================================");
        Console.WriteLine($" STRES TESTİ: {iterations:N0} İSTEK ({repeats} TEKRAR MEDYAN & STDDEV - ORDER BIAS FREE)");
        Console.WriteLine($"==========================================================================");

        var tests = new List<(string Name, Func<Task> Action)>
        {
            ("SampleApp Command Dispatcher (Standart / Fast-Path)", async () =>
            {
                for (int i = 0; i < iterations; i++)
                    await _sampleAppNoBehaviors.SendAsync(_sampleAppCommand);
            }),
            ("SampleApp Command Dispatcher (+ 2 Pipeline Behavior)", async () =>
            {
                for (int i = 0; i < iterations; i++)
                    await _sampleAppWithBehaviors.SendAsync(_sampleAppCommand);
            }),
            ("SampleApp Query Dispatcher (+ 2 Pipeline Behavior)", async () =>
            {
                for (int i = 0; i < iterations; i++)
                    await _sampleAppWithBehaviors.QueryAsync(_sampleAppQuery);
            }),
            ("MediatR v12.4.1 Command (Standart Pipeline)", async () =>
            {
                for (int i = 0; i < iterations; i++)
                    await _mediatRNoBehaviors.Send(_mediatRCommand);
            }),
            ("MediatR v12.4.1 Command (+ 2 Pipeline Behavior)", async () =>
            {
                for (int i = 0; i < iterations; i++)
                    await _mediatRWithBehaviors.Send(_mediatRCommand);
            }),
            ("Direct Typed Baseline (Ref-Free Direct DI)", async () =>
            {
                for (int i = 0; i < iterations; i++)
                    await _directDispatcher.SendAsync<DirectCommand, string>(_directCommand);
            })
        };

        var runsList = new Dictionary<string, List<TestResult>>();
        foreach (var t in tests) runsList[t.Name] = new List<TestResult>();

        for (int r = 0; r < repeats; r++)
        {
            var shuffled = tests.OrderBy(_ => Random.Shared.Next()).ToList();
            foreach (var t in shuffled)
            {
                var res = await MeasureAsync(t.Name, t.Action, iterations);
                runsList[t.Name].Add(res);
            }
        }

        // Calculate Medians and StdDev across the repeats (strictly using Median for both Time and Allocated Memory)
        var finalResults = new List<AggregatedResult>();
        foreach (var t in tests)
        {
            var runs = runsList[t.Name];
            var sortedTimes = runs.Select(x => x.ElapsedMs).OrderBy(x => x).ToList();
            var sortedAllocs = runs.Select(x => x.AllocatedMB).OrderBy(x => x).ToList();

            double medianMs = sortedTimes[sortedTimes.Count / 2];
            double medianAllocatedMB = sortedAllocs[sortedAllocs.Count / 2];
            double avgMs = sortedTimes.Average();
            double stdDev = Math.Sqrt(sortedTimes.Select(x => Math.Pow(x - avgMs, 2)).Sum() / sortedTimes.Count);
            double opsPerSec = iterations / (medianMs / 1000.0);

            finalResults.Add(new AggregatedResult(t.Name, medianMs, stdDev, opsPerSec, medianAllocatedMB));
        }

        printResultsTable(iterations, finalResults);
    }

    private async Task<TestResult> MeasureAsync(string name, Func<Task> action, int iterations)
    {
        GC.Collect();
        GC.WaitForPendingFinalizers();
        GC.Collect();

        long startBytes = GC.GetTotalAllocatedBytes(true);
        var sw = Stopwatch.StartNew();

        await action();

        sw.Stop();
        long endBytes = GC.GetTotalAllocatedBytes(true);

        double elapsedMs = sw.Elapsed.TotalMilliseconds;
        double opsPerSec = iterations / (sw.Elapsed.TotalSeconds == 0 ? 0.00001 : sw.Elapsed.TotalSeconds);
        long allocatedBytes = Math.Max(0, endBytes - startBytes);
        double allocatedMB = allocatedBytes / (1024.0 * 1024.0);

        return new TestResult(name, elapsedMs, opsPerSec, allocatedBytes, allocatedMB);
    }

    private void printResultsTable(int iterations, List<AggregatedResult> results)
    {
        double baseTime = results[0].MedianMs;

        Console.WriteLine($"\n| {"Mimari / Yöntem",-50} | {"Medyan (ms)",11} | {"StdDev (ms)",11} | {"Ops / Saniye",14} | {"Tahsis (MB)",12} | {"Hız Oranı (vs SA)",18} |");
        Console.WriteLine($"|{new string('-', 52)}|{new string('-', 13)}|{new string('-', 13)}|{new string('-', 16)}|{new string('-', 14)}|{new string('-', 20)}|");

        foreach (var r in results)
        {
            double ratio = r.MedianMs / baseTime;
            string speedLabel = ratio <= 1.05 ? "1.00x (Baseline)" : $"{ratio:F2}x Daha Yavaş";
            Console.WriteLine($"| {r.Name,-50} | {r.MedianMs,11:F2} | {r.StdDev,11:F2} | {r.OpsPerSec,14:N0} | {r.AllocatedMB,12:F2} | {speedLabel,18} |");
        }
        Console.WriteLine();
    }

    private record TestResult(string Name, double ElapsedMs, double OpsPerSec, long AllocatedBytes, double AllocatedMB);
    private record AggregatedResult(string Name, double MedianMs, double StdDev, double OpsPerSec, double AllocatedMB);
}

// -------------------------------------------------------------
// BenchmarkDotNet Micro-Benchmark Suite
// -------------------------------------------------------------
[MemoryDiagnoser]
[RankColumn]
public class DispatcherComparisonBenchmark
{
    private IServiceProvider _spSampleAppNoBehaviors = null!;
    private IServiceProvider _spSampleAppWithBehaviors = null!;
    private IServiceProvider _spMediatRNoBehaviors = null!;
    private IServiceProvider _spMediatRWithBehaviors = null!;
    private IServiceProvider _spDirect = null!;

    private NativeApp.Common.IDispatcher _sampleAppDispatcherNoBehaviors = null!;
    private NativeApp.Common.IDispatcher _sampleAppDispatcherWithBehaviors = null!;
    private MediatR.IMediator _mediatRNoBehaviors = null!;
    private MediatR.IMediator _mediatRWithBehaviors = null!;
    private IDirectDispatcher _directDispatcher = null!;

    private SampleAppCommand _sampleAppCommand = null!;
    private SampleAppQuery _sampleAppQuery = null!;
    private MediatRCommand _mediatRCommand = null!;
    private MediatRQuery _mediatRQuery = null!;
    private DirectCommand _directCommand = null!;
    private DirectQuery _directQuery = null!;

    [GlobalSetup]
    public void Setup()
    {
        var services1 = new ServiceCollection();
        services1.AddScoped<NativeApp.Common.IDispatcher, NativeApp.Common.Dispatcher>();
        services1.AddTransient<NativeApp.Common.ICommandHandler<SampleAppCommand, string>, SampleAppCommandHandler>();
        services1.AddTransient<NativeApp.Common.IQueryHandler<SampleAppQuery, string>, SampleAppQueryHandler>();
        _spSampleAppNoBehaviors = services1.BuildServiceProvider();
        _sampleAppDispatcherNoBehaviors = _spSampleAppNoBehaviors.GetRequiredService<NativeApp.Common.IDispatcher>();

        var services2 = new ServiceCollection();
        services2.AddScoped<NativeApp.Common.IDispatcher, NativeApp.Common.Dispatcher>();
        services2.AddTransient<NativeApp.Common.ICommandHandler<SampleAppCommand, string>, SampleAppCommandHandler>();
        services2.AddTransient<NativeApp.Common.IQueryHandler<SampleAppQuery, string>, SampleAppQueryHandler>();
        services2.AddTransient(typeof(NativeApp.Common.IPipelineBehavior<,>), typeof(PassThroughBehavior1<,>));
        services2.AddTransient(typeof(NativeApp.Common.IPipelineBehavior<,>), typeof(PassThroughBehavior2<,>));
        _spSampleAppWithBehaviors = services2.BuildServiceProvider();
        _sampleAppDispatcherWithBehaviors = _spSampleAppWithBehaviors.GetRequiredService<NativeApp.Common.IDispatcher>();

        var services3 = new ServiceCollection();
        services3.AddMediatR(cfg => cfg.RegisterServicesFromAssembly(typeof(Program).Assembly));
        _spMediatRNoBehaviors = services3.BuildServiceProvider();
        _mediatRNoBehaviors = _spMediatRNoBehaviors.GetRequiredService<MediatR.IMediator>();

        var services4 = new ServiceCollection();
        services4.AddMediatR(cfg =>
        {
            cfg.RegisterServicesFromAssembly(typeof(Program).Assembly);
            cfg.AddBehavior(typeof(MediatR.IPipelineBehavior<,>), typeof(MediatRPassThroughBehavior1<,>));
            cfg.AddBehavior(typeof(MediatR.IPipelineBehavior<,>), typeof(MediatRPassThroughBehavior2<,>));
        });
        _spMediatRWithBehaviors = services4.BuildServiceProvider();
        _mediatRWithBehaviors = _spMediatRWithBehaviors.GetRequiredService<MediatR.IMediator>();

        var services5 = new ServiceCollection();
        services5.AddScoped<IDirectDispatcher, DirectTypedDispatcher>();
        services5.AddTransient<IDirectCommandHandler<DirectCommand, string>, DirectCommandHandler>();
        services5.AddTransient<IDirectQueryHandler<DirectQuery, string>, DirectQueryHandler>();
        _spDirect = services5.BuildServiceProvider();
        _directDispatcher = _spDirect.GetRequiredService<IDirectDispatcher>();

        _sampleAppCommand = new SampleAppCommand("Benchmark");
        _sampleAppQuery = new SampleAppQuery(100);
        _mediatRCommand = new MediatRCommand("Benchmark");
        _mediatRQuery = new MediatRQuery(100);
        _directCommand = new DirectCommand("Benchmark");
        _directQuery = new DirectQuery(100);
    }

    [Benchmark(Baseline = true)]
    public async Task<string> MediatR_Command()
    {
        return await _mediatRNoBehaviors.Send(_mediatRCommand);
    }

    [Benchmark]
    public async Task<string> MediatR_Command_With2Behaviors()
    {
        return await _mediatRWithBehaviors.Send(_mediatRCommand);
    }

    [Benchmark]
    public async Task<string> SampleApp_Dispatcher_Command()
    {
        return await _sampleAppDispatcherNoBehaviors.SendAsync(_sampleAppCommand);
    }

    [Benchmark]
    public async Task<string> SampleApp_Dispatcher_Command_With2Behaviors()
    {
        return await _sampleAppDispatcherWithBehaviors.SendAsync(_sampleAppCommand);
    }

    [Benchmark]
    public async Task<string> SampleApp_Dispatcher_Query_With2Behaviors()
    {
        return await _sampleAppDispatcherWithBehaviors.QueryAsync(_sampleAppQuery);
    }

    [Benchmark]
    public async Task<string> DirectTyped_Command_Baseline()
    {
        return await _directDispatcher.SendAsync<DirectCommand, string>(_directCommand);
    }
}

// -------------------------------------------------------------
// BenchmarkDotNet Batch Stress Benchmark (1K - 1M)
// -------------------------------------------------------------
[SimpleJob(launchCount: 1, warmupCount: 2, iterationCount: 5)]
[MemoryDiagnoser]
[RankColumn]
public class DispatcherBatchBenchmark
{
    [Params(1_000, 10_000, 100_000, 1_000_000)]
    public int N;

    private IServiceProvider _spSampleAppNoBehaviors = null!;
    private IServiceProvider _spSampleAppWithBehaviors = null!;
    private IServiceProvider _spMediatRNoBehaviors = null!;
    private IServiceProvider _spMediatRWithBehaviors = null!;
    private IServiceProvider _spDirect = null!;

    private NativeApp.Common.IDispatcher _sampleAppDispatcherNoBehaviors = null!;
    private NativeApp.Common.IDispatcher _sampleAppDispatcherWithBehaviors = null!;
    private MediatR.IMediator _mediatRNoBehaviors = null!;
    private MediatR.IMediator _mediatRWithBehaviors = null!;
    private IDirectDispatcher _directDispatcher = null!;

    private SampleAppCommand _sampleAppCommand = null!;
    private SampleAppQuery _sampleAppQuery = null!;
    private MediatRCommand _mediatRCommand = null!;
    private DirectCommand _directCommand = null!;

    [GlobalSetup]
    public void Setup()
    {
        var services1 = new ServiceCollection();
        services1.AddScoped<NativeApp.Common.IDispatcher, NativeApp.Common.Dispatcher>();
        services1.AddTransient<NativeApp.Common.ICommandHandler<SampleAppCommand, string>, SampleAppCommandHandler>();
        services1.AddTransient<NativeApp.Common.IQueryHandler<SampleAppQuery, string>, SampleAppQueryHandler>();
        _spSampleAppNoBehaviors = services1.BuildServiceProvider();
        _sampleAppDispatcherNoBehaviors = _spSampleAppNoBehaviors.GetRequiredService<NativeApp.Common.IDispatcher>();

        var services2 = new ServiceCollection();
        services2.AddScoped<NativeApp.Common.IDispatcher, NativeApp.Common.Dispatcher>();
        services2.AddTransient<NativeApp.Common.ICommandHandler<SampleAppCommand, string>, SampleAppCommandHandler>();
        services2.AddTransient<NativeApp.Common.IQueryHandler<SampleAppQuery, string>, SampleAppQueryHandler>();
        services2.AddTransient(typeof(NativeApp.Common.IPipelineBehavior<,>), typeof(PassThroughBehavior1<,>));
        services2.AddTransient(typeof(NativeApp.Common.IPipelineBehavior<,>), typeof(PassThroughBehavior2<,>));
        _spSampleAppWithBehaviors = services2.BuildServiceProvider();
        _sampleAppDispatcherWithBehaviors = _spSampleAppWithBehaviors.GetRequiredService<NativeApp.Common.IDispatcher>();

        var services3 = new ServiceCollection();
        services3.AddMediatR(cfg => cfg.RegisterServicesFromAssembly(typeof(Program).Assembly));
        _spMediatRNoBehaviors = services3.BuildServiceProvider();
        _mediatRNoBehaviors = _spMediatRNoBehaviors.GetRequiredService<MediatR.IMediator>();

        var services4 = new ServiceCollection();
        services4.AddMediatR(cfg =>
        {
            cfg.RegisterServicesFromAssembly(typeof(Program).Assembly);
            cfg.AddBehavior(typeof(MediatR.IPipelineBehavior<,>), typeof(MediatRPassThroughBehavior1<,>));
            cfg.AddBehavior(typeof(MediatR.IPipelineBehavior<,>), typeof(MediatRPassThroughBehavior2<,>));
        });
        _spMediatRWithBehaviors = services4.BuildServiceProvider();
        _mediatRWithBehaviors = _spMediatRWithBehaviors.GetRequiredService<MediatR.IMediator>();

        var services5 = new ServiceCollection();
        services5.AddScoped<IDirectDispatcher, DirectTypedDispatcher>();
        services5.AddTransient<IDirectCommandHandler<DirectCommand, string>, DirectCommandHandler>();
        _spDirect = services5.BuildServiceProvider();
        _directDispatcher = _spDirect.GetRequiredService<IDirectDispatcher>();

        _sampleAppCommand = new SampleAppCommand("BatchTest");
        _sampleAppQuery = new SampleAppQuery(777);
        _mediatRCommand = new MediatRCommand("BatchTest");
        _directCommand = new DirectCommand("BatchTest");
    }

    [Benchmark(Baseline = true)]
    public async Task MediatR_Batch_Standart()
    {
        for (int i = 0; i < N; i++)
            await _mediatRNoBehaviors.Send(_mediatRCommand);
    }

    [Benchmark]
    public async Task MediatR_Batch_With2Behaviors()
    {
        for (int i = 0; i < N; i++)
            await _mediatRWithBehaviors.Send(_mediatRCommand);
    }

    [Benchmark]
    public async Task SampleApp_Batch_Standart()
    {
        for (int i = 0; i < N; i++)
            await _sampleAppDispatcherNoBehaviors.SendAsync(_sampleAppCommand);
    }

    [Benchmark]
    public async Task SampleApp_Batch_With2Behaviors_Command()
    {
        for (int i = 0; i < N; i++)
            await _sampleAppDispatcherWithBehaviors.SendAsync(_sampleAppCommand);
    }

    [Benchmark]
    public async Task SampleApp_Batch_With2Behaviors_Query()
    {
        for (int i = 0; i < N; i++)
            await _sampleAppDispatcherWithBehaviors.QueryAsync(_sampleAppQuery);
    }

    [Benchmark]
    public async Task DirectTyped_Batch_Baseline()
    {
        for (int i = 0; i < N; i++)
            await _directDispatcher.SendAsync<DirectCommand, string>(_directCommand);
    }
}

// -------------------------------------------------------------
// Real-World ASP.NET Core HTTP Request Scoped Benchmark Matrix
// -------------------------------------------------------------
[MemoryDiagnoser]
[RankColumn]
public class DispatcherScopedBenchmark
{
    private IServiceProvider _spSampleAppNoBehaviors = null!;
    private IServiceProvider _spSampleAppWithBehaviors = null!;
    private IServiceProvider _spMediatRNoBehaviors = null!;
    private IServiceProvider _spMediatRWithBehaviors = null!;
    private IServiceProvider _spDirect = null!;

    private SampleAppCommand _sampleAppCommand = null!;
    private SampleAppQuery _sampleAppQuery = null!;
    private MediatRCommand _mediatRCommand = null!;
    private DirectCommand _directCommand = null!;

    [GlobalSetup]
    public void Setup()
    {
        var services1 = new ServiceCollection();
        services1.AddScoped<NativeApp.Common.IDispatcher, NativeApp.Common.Dispatcher>();
        services1.AddTransient<NativeApp.Common.ICommandHandler<SampleAppCommand, string>, SampleAppCommandHandler>();
        services1.AddTransient<NativeApp.Common.IQueryHandler<SampleAppQuery, string>, SampleAppQueryHandler>();
        _spSampleAppNoBehaviors = services1.BuildServiceProvider();

        var services2 = new ServiceCollection();
        services2.AddScoped<NativeApp.Common.IDispatcher, NativeApp.Common.Dispatcher>();
        services2.AddTransient<NativeApp.Common.ICommandHandler<SampleAppCommand, string>, SampleAppCommandHandler>();
        services2.AddTransient<NativeApp.Common.IQueryHandler<SampleAppQuery, string>, SampleAppQueryHandler>();
        services2.AddTransient(typeof(NativeApp.Common.IPipelineBehavior<,>), typeof(PassThroughBehavior1<,>));
        services2.AddTransient(typeof(NativeApp.Common.IPipelineBehavior<,>), typeof(PassThroughBehavior2<,>));
        _spSampleAppWithBehaviors = services2.BuildServiceProvider();

        var services3 = new ServiceCollection();
        services3.AddMediatR(cfg => cfg.RegisterServicesFromAssembly(typeof(Program).Assembly));
        _spMediatRNoBehaviors = services3.BuildServiceProvider();

        var services4 = new ServiceCollection();
        services4.AddMediatR(cfg =>
        {
            cfg.RegisterServicesFromAssembly(typeof(Program).Assembly);
            cfg.AddBehavior(typeof(MediatR.IPipelineBehavior<,>), typeof(MediatRPassThroughBehavior1<,>));
            cfg.AddBehavior(typeof(MediatR.IPipelineBehavior<,>), typeof(MediatRPassThroughBehavior2<,>));
        });
        _spMediatRWithBehaviors = services4.BuildServiceProvider();

        var services5 = new ServiceCollection();
        services5.AddScoped<IDirectDispatcher, DirectTypedDispatcher>();
        services5.AddTransient<IDirectCommandHandler<DirectCommand, string>, DirectCommandHandler>();
        _spDirect = services5.BuildServiceProvider();

        _sampleAppCommand = new SampleAppCommand("ScopedHttpRequest");
        _sampleAppQuery = new SampleAppQuery(999);
        _mediatRCommand = new MediatRCommand("ScopedHttpRequest");
        _directCommand = new DirectCommand("ScopedHttpRequest");
    }

    [Benchmark(Baseline = true)]
    public async Task<string> MediatR_Scoped_HttpRequest_Standart()
    {
        using var scope = _spMediatRNoBehaviors.CreateScope();
        var mediator = scope.ServiceProvider.GetRequiredService<MediatR.IMediator>();
        return await mediator.Send(_mediatRCommand);
    }

    [Benchmark]
    public async Task<string> MediatR_Scoped_HttpRequest_With2Behaviors()
    {
        using var scope = _spMediatRWithBehaviors.CreateScope();
        var mediator = scope.ServiceProvider.GetRequiredService<MediatR.IMediator>();
        return await mediator.Send(_mediatRCommand);
    }

    [Benchmark]
    public async Task<string> SampleApp_Scoped_HttpRequest_Standart()
    {
        using var scope = _spSampleAppNoBehaviors.CreateScope();
        var dispatcher = scope.ServiceProvider.GetRequiredService<NativeApp.Common.IDispatcher>();
        return await dispatcher.SendAsync(_sampleAppCommand);
    }

    [Benchmark]
    public async Task<string> SampleApp_Scoped_HttpRequest_With2Behaviors_Command()
    {
        using var scope = _spSampleAppWithBehaviors.CreateScope();
        var dispatcher = scope.ServiceProvider.GetRequiredService<NativeApp.Common.IDispatcher>();
        return await dispatcher.SendAsync(_sampleAppCommand);
    }

    [Benchmark]
    public async Task<string> SampleApp_Scoped_HttpRequest_With2Behaviors_Query()
    {
        using var scope = _spSampleAppWithBehaviors.CreateScope();
        var dispatcher = scope.ServiceProvider.GetRequiredService<NativeApp.Common.IDispatcher>();
        return await dispatcher.QueryAsync(_sampleAppQuery);
    }

    [Benchmark]
    public async Task<string> DirectTyped_Scoped_HttpRequest_Baseline()
    {
        using var scope = _spDirect.CreateScope();
        var dispatcher = scope.ServiceProvider.GetRequiredService<IDirectDispatcher>();
        return await dispatcher.SendAsync<DirectCommand, string>(_directCommand);
    }
}
