using System;
using System.Collections.Generic;
using System.Diagnostics;
using System.Linq;
using System.Net.Http;
using System.Net.Http.Json;
using System.Text;
using System.Threading.Tasks;
using LoadSurge.Models;
using LoadSurge.Runner;

namespace HttpApiBenchmark;

public class Program
{
    public static async Task Main(string[] args)
    {
        Console.OutputEncoding = Encoding.UTF8;
        Console.WriteLine("==========================================================================");
        Console.WriteLine(" LoadSurge RIGOROUS UNTHROTTLED (FINE-GRAINED STEPPING 75->85->90->95->100)");
        Console.WriteLine(" NativeApp vs MediatRApp vs PureNativeApp (Dispatcher-less)");
        Console.WriteLine(" NativeApp (Native Dispatcher) http://localhost:5005");
        Console.WriteLine(" MediatRApp (MediatR v12.4.1) http://localhost:5020");
        Console.WriteLine(" PureNativeApp (Dispatcher-less) http://localhost:5030");
        Console.WriteLine("==========================================================================");
        Console.WriteLine();

        var handler = new SocketsHttpHandler
        {
            PooledConnectionLifetime = TimeSpan.FromMinutes(15),
            PooledConnectionIdleTimeout = TimeSpan.FromMinutes(2),
            MaxConnectionsPerServer = 500, // Calibrated for 500 max connection pool
            EnableMultipleHttp2Connections = true
        };

        using var httpClient = new HttpClient(handler);

        // Ensure all Web API servers are alive
        await WarmupEndpointsAsync(httpClient);

        // -------------------------------------------------------------
        // FAZ 0: ÇİFT YÖNLÜ İSTEMCİ & SUNUCU KALİBRASYONU (Interval = 1ms)
        // -------------------------------------------------------------
        Console.WriteLine("\n==========================================================================");
        Console.WriteLine(" FAZ 0: NATIVEAPP, MEDIATRAPP VE PURENATIVEAPP KALİBRASYON (Interval = 1ms)");
        Console.WriteLine("==========================================================================");

        var calibrationResults = await RunDualCalibrationPhaseAsync(httpClient);
        PrintSteppedSummaryTable("FAZ 0: KALİBRASYON (NATIVE vs MEDIATR vs PURE NATIVE - INTERVAL = 1ms)", calibrationResults);

        // -------------------------------------------------------------
        // FAZ 1: 1-to-1 UNTHROTTLED FAIR COMPARISON (50 Concurrency, 5 Repeats Median)
        // -------------------------------------------------------------
        const int repeats = 5;
        Console.WriteLine("\n==========================================================================");
        Console.WriteLine($" FAZ 1: UNTHROTTLED 1-to-1 FAİR COMPARISON (50 Concurrency, {repeats} Tekrar Medyan)");
        Console.WriteLine("==========================================================================");

        var sustainedScenarios = new List<ScenarioDefinition>
        {
            new ScenarioDefinition(
                Name: "NativeApp HTTP Command (Native)",
                ProcessName: "NativeApp",
                CreatePlan: () => new LoadExecutionPlan
                {
                    Name = "NativeApp_Unthrottled_Command",
                    Settings = new LoadSettings
                    {
                        Concurrency = 50,
                        Duration = TimeSpan.FromSeconds(3),
                        Interval = TimeSpan.FromMilliseconds(1),
                        RequestTimeout = TimeSpan.FromSeconds(3)
                    },
                    ActionWithCancellation = async token =>
                    {
                        var response = await httpClient.PostAsJsonAsync("http://localhost:5005/api/Benchmark/command", new { Message = "UnthrottledTest" }, token);
                        return response.IsSuccessStatusCode;
                    }
                }
            ),

            new ScenarioDefinition(
                Name: "MediatRApp HTTP Command (MediatR v12.4.1)",
                ProcessName: "MediatRApp",
                CreatePlan: () => new LoadExecutionPlan
                {
                    Name = "MediatRApp_Unthrottled_Command",
                    Settings = new LoadSettings
                    {
                        Concurrency = 50,
                        Duration = TimeSpan.FromSeconds(3),
                        Interval = TimeSpan.FromMilliseconds(1),
                        RequestTimeout = TimeSpan.FromSeconds(3)
                    },
                    ActionWithCancellation = async token =>
                    {
                        var response = await httpClient.PostAsJsonAsync("http://localhost:5020/api/Benchmark/command", new { Message = "UnthrottledTest" }, token);
                        return response.IsSuccessStatusCode;
                    }
                }
            ),

            new ScenarioDefinition(
                Name: "PureNativeApp HTTP Command (Dispatcher-less)",
                ProcessName: "PureNativeApp",
                CreatePlan: () => new LoadExecutionPlan
                {
                    Name = "PureNativeApp_Unthrottled_Command",
                    Settings = new LoadSettings
                    {
                        Concurrency = 50,
                        Duration = TimeSpan.FromSeconds(3),
                        Interval = TimeSpan.FromMilliseconds(1),
                        RequestTimeout = TimeSpan.FromSeconds(3)
                    },
                    ActionWithCancellation = async token =>
                    {
                        var response = await httpClient.PostAsJsonAsync("http://localhost:5030/api/Benchmark/command", new { Message = "UnthrottledTest" }, token);
                        return response.IsSuccessStatusCode;
                    }
                }
            ),

            new ScenarioDefinition(
                Name: "NativeApp HTTP Query (Native)",
                ProcessName: "NativeApp",
                CreatePlan: () => new LoadExecutionPlan
                {
                    Name = "NativeApp_Unthrottled_Query",
                    Settings = new LoadSettings
                    {
                        Concurrency = 50,
                        Duration = TimeSpan.FromSeconds(3),
                        Interval = TimeSpan.FromMilliseconds(1),
                        RequestTimeout = TimeSpan.FromSeconds(3)
                    },
                    ActionWithCancellation = async token =>
                    {
                        var response = await httpClient.GetAsync("http://localhost:5005/api/Benchmark/query?id=999", token);
                        return response.IsSuccessStatusCode;
                    }
                }
            ),

            new ScenarioDefinition(
                Name: "MediatRApp HTTP Query (MediatR v12.4.1)",
                ProcessName: "MediatRApp",
                CreatePlan: () => new LoadExecutionPlan
                {
                    Name = "MediatRApp_Unthrottled_Query",
                    Settings = new LoadSettings
                    {
                        Concurrency = 50,
                        Duration = TimeSpan.FromSeconds(3),
                        Interval = TimeSpan.FromMilliseconds(1),
                        RequestTimeout = TimeSpan.FromSeconds(3)
                    },
                    ActionWithCancellation = async token =>
                    {
                        var response = await httpClient.GetAsync("http://localhost:5020/api/Benchmark/query?id=999", token);
                        return response.IsSuccessStatusCode;
                    }
                }
            ),

            new ScenarioDefinition(
                Name: "PureNativeApp HTTP Query (Dispatcher-less)",
                ProcessName: "PureNativeApp",
                CreatePlan: () => new LoadExecutionPlan
                {
                    Name = "PureNativeApp_Unthrottled_Query",
                    Settings = new LoadSettings
                    {
                        Concurrency = 50,
                        Duration = TimeSpan.FromSeconds(3),
                        Interval = TimeSpan.FromMilliseconds(1),
                        RequestTimeout = TimeSpan.FromSeconds(3)
                    },
                    ActionWithCancellation = async token =>
                    {
                        var response = await httpClient.GetAsync("http://localhost:5030/api/Benchmark/query?id=999", token);
                        return response.IsSuccessStatusCode;
                    }
                }
            )
        };

        var faz1Results = await RunBenchmarkPhaseAsync(sustainedScenarios, repeats);
        PrintSummaryTable("FAZ 1: UNTHROTTLED COMPARISON (INTERVAL = 1ms, 50 CONCURRENCY - 5 TEKRAR MEDYAN)", faz1Results);

        // -------------------------------------------------------------
        // FAZ 2: HASSAS ADIMLI STEPPED STRESS (75 -> 85 -> 90 -> 95 -> 100)
        // -------------------------------------------------------------
        Console.WriteLine("\n==========================================================================");
        Console.WriteLine($" FAZ 2: HASSAS ADIMLI STEPPED STRESS TEST ({repeats} Tekrar Medyan + StdDev)");
        Console.WriteLine("==========================================================================");

        var breakingPointResults = await RunSteppedPhaseWithRepeatsAsync(httpClient, repeats);
        PrintSteppedSummaryTable("FAZ 2: HASSAS ADIMLI STEPPED STRESS SONUÇLARI (5 TEKRAR MEDYAN)", breakingPointResults);
    }

    private static async Task<List<SteppedScenarioResult>> RunDualCalibrationPhaseAsync(HttpClient httpClient)
    {
        var concurrencies = new[] { 10, 25, 50, 75, 100 };
        var targets = new[]
        {
            (Name: "NativeApp Kalibrasyon", AppName: "NativeApp", Url: "http://localhost:5005/api/Benchmark/query?id=999"),
            (Name: "MediatRApp Kalibrasyon", AppName: "MediatRApp", Url: "http://localhost:5020/api/Benchmark/query?id=999"),
            (Name: "PureNativeApp Kalibrasyon", AppName: "PureNativeApp", Url: "http://localhost:5030/api/Benchmark/query?id=999")
        };

        var list = new List<SteppedScenarioResult>();

        foreach (var target in targets)
        {
            Console.WriteLine($"\n  [KALİBRASYON] {target.Name}");
            foreach (var c in concurrencies)
            {
                GC.Collect();
                GC.WaitForPendingFinalizers();
                GC.Collect();
                await Task.Delay(500);

                var plan = new LoadExecutionPlan
                {
                    Name = $"{target.Name}_C{c}",
                    Settings = new LoadSettings
                    {
                        Concurrency = c,
                        Duration = TimeSpan.FromSeconds(3),
                        Interval = TimeSpan.FromMilliseconds(1),
                        RequestTimeout = TimeSpan.FromSeconds(3)
                    },
                    ActionWithCancellation = async token =>
                    {
                        try
                        {
                            var resp = await httpClient.GetAsync(target.Url, token);
                            return resp.IsSuccessStatusCode;
                        }
                        catch
                        {
                            return false;
                        }
                    }
                };

                LoadResult res;
                try
                {
                    res = await LoadRunner.Run(plan);
                }
                catch (ArgumentException)
                {
                    await Task.Delay(200);
                    res = await LoadRunner.Run(plan);
                }

                var proc = Process.GetProcessesByName(target.AppName).FirstOrDefault();
                double ramMB = proc != null ? proc.WorkingSet64 / (1024.0 * 1024.0) : 0;

                list.Add(new SteppedScenarioResult(target.Name, c, res.RequestsPerSecond, 0, res.AverageLatency, res.Percentile95Latency, res.Percentile99Latency, res.Failure, ramMB));
                Console.WriteLine($"    Concurrency {c,3}: RPS = {res.RequestsPerSecond,7:N0}, Mean = {res.AverageLatency,6:F2}ms, P95 = {res.Percentile95Latency,6:F2}ms, Fail = {res.Failure}, RAM = {ramMB:F1}MB");
            }
        }

        return list;
    }

    private static async Task<List<ScenarioSummary>> RunBenchmarkPhaseAsync(
        List<ScenarioDefinition> scenarios,
        int repeats)
    {
        var aggregatedResults = new List<ScenarioSummary>();

        foreach (var sc in scenarios)
        {
            Console.WriteLine($"\n  [İZOLASYON] {sc.Name} ({repeats} Tekrar)");

            var rpsList = new List<double>();
            var meanMsList = new List<double>();
            var p95MsList = new List<double>();
            var p99MsList = new List<double>();
            var failCountList = new List<long>();
            var memoryMBList = new List<double>();

            for (int r = 1; r <= repeats; r++)
            {
                GC.Collect();
                GC.WaitForPendingFinalizers();
                GC.Collect();
                await Task.Delay(500);

                var procBefore = Process.GetProcessesByName(sc.ProcessName).FirstOrDefault();
                double ramBeforeMB = procBefore != null ? procBefore.WorkingSet64 / (1024.0 * 1024.0) : 0;

                var plan = sc.CreatePlan();
                LoadResult result;
                try
                {
                    result = await LoadRunner.Run(plan);
                }
                catch (ArgumentException)
                {
                    await Task.Delay(200);
                    result = await LoadRunner.Run(plan);
                }

                var procAfter = Process.GetProcessesByName(sc.ProcessName).FirstOrDefault();
                double ramAfterMB = procAfter != null ? procAfter.WorkingSet64 / (1024.0 * 1024.0) : ramBeforeMB;

                rpsList.Add(result.RequestsPerSecond);
                meanMsList.Add(result.AverageLatency);
                p95MsList.Add(result.Percentile95Latency);
                p99MsList.Add(result.Percentile99Latency);
                failCountList.Add(result.Failure);
                memoryMBList.Add(ramAfterMB);

                Console.WriteLine($"    Tekrar {r}/{repeats}: RPS = {result.RequestsPerSecond:N0}, Mean = {result.AverageLatency:F2}ms, P95 = {result.Percentile95Latency:F2}ms, Fail = {result.Failure}, RAM = {ramAfterMB:F1} MB");
            }

            // Compute Medians
            rpsList.Sort();
            meanMsList.Sort();
            p95MsList.Sort();
            p99MsList.Sort();
            memoryMBList.Sort();

            double medianRps = rpsList[repeats / 2];
            double medianMean = meanMsList[repeats / 2];
            double medianP95 = p95MsList[repeats / 2];
            double medianP99 = p99MsList[repeats / 2];
            double medianRamMB = memoryMBList[repeats / 2];

            double avgRps = rpsList.Average();
            double stdDevRps = Math.Sqrt(rpsList.Select(x => Math.Pow(x - avgRps, 2)).Sum() / repeats);

            aggregatedResults.Add(new ScenarioSummary(
                sc.Name,
                medianRps,
                stdDevRps,
                medianMean,
                medianP95,
                medianP99,
                failCountList.Max(),
                medianRamMB
            ));
        }

        return aggregatedResults;
    }

    private static async Task<List<SteppedScenarioResult>> RunSteppedPhaseWithRepeatsAsync(HttpClient httpClient, int repeats)
    {
        // Refined Stepping: 10 -> 25 -> 50 -> 75 -> 85 -> 90 -> 95 -> 100
        var steppedConfigs = new[]
        {
            (Concurrency: 10, DurationSec: 3),
            (Concurrency: 25, DurationSec: 3),
            (Concurrency: 50, DurationSec: 3),
            (Concurrency: 75, DurationSec: 3),
            (Concurrency: 85, DurationSec: 3),
            (Concurrency: 90, DurationSec: 3),
            (Concurrency: 95, DurationSec: 3),
            (Concurrency: 100, DurationSec: 3)
        };

        var targets = new[]
        {
            (Name: "NativeApp Command", AppName: "NativeApp", Url: "http://localhost:5005/api/Benchmark/command", IsPost: true),
            (Name: "MediatRApp Command", AppName: "MediatRApp", Url: "http://localhost:5020/api/Benchmark/command", IsPost: true),
            (Name: "PureNativeApp Command", AppName: "PureNativeApp", Url: "http://localhost:5030/api/Benchmark/command", IsPost: true),
            (Name: "NativeApp Query", AppName: "NativeApp", Url: "http://localhost:5005/api/Benchmark/query?id=999", IsPost: false),
            (Name: "MediatRApp Query", AppName: "MediatRApp", Url: "http://localhost:5020/api/Benchmark/query?id=999", IsPost: false),
            (Name: "PureNativeApp Query", AppName: "PureNativeApp", Url: "http://localhost:5030/api/Benchmark/query?id=999", IsPost: false)
        };

        var list = new List<SteppedScenarioResult>();

        foreach (var target in targets)
        {
            Console.WriteLine($"\n  [STEPPED RIGOROUS] {target.Name} ({repeats} Tekrar Medyan)");
            foreach (var cfg in steppedConfigs)
            {
                var rpsList = new List<double>();
                var meanMsList = new List<double>();
                var p95MsList = new List<double>();
                var p99MsList = new List<double>();
                var failCountList = new List<long>();
                var memoryMBList = new List<double>();

                for (int r = 1; r <= repeats; r++)
                {
                    GC.Collect();
                    GC.WaitForPendingFinalizers();
                    GC.Collect();
                    await Task.Delay(500);

                    var procBefore = Process.GetProcessesByName(target.AppName).FirstOrDefault();
                    double ramBeforeMB = procBefore != null ? procBefore.WorkingSet64 / (1024.0 * 1024.0) : 0;

                    var plan = new LoadExecutionPlan
                    {
                        Name = $"{target.Name}_C{cfg.Concurrency}_R{r}",
                        Settings = new LoadSettings
                        {
                            Concurrency = cfg.Concurrency,
                            Duration = TimeSpan.FromSeconds(cfg.DurationSec),
                            Interval = TimeSpan.FromMilliseconds(1),
                            RequestTimeout = TimeSpan.FromSeconds(3)
                        },
                        ActionWithCancellation = async token =>
                        {
                            try
                            {
                                HttpResponseMessage resp;
                                if (target.IsPost)
                                    resp = await httpClient.PostAsJsonAsync(target.Url, new { Message = "SteppedTest" }, token);
                                else
                                    resp = await httpClient.GetAsync(target.Url, token);
                                return resp.IsSuccessStatusCode;
                            }
                            catch (Exception)
                            {
                                return false;
                            }
                        }
                    };

                    LoadResult res;
                    try
                    {
                        res = await LoadRunner.Run(plan);
                    }
                    catch (ArgumentException)
                    {
                        await Task.Delay(200);
                        res = await LoadRunner.Run(plan);
                    }

                    var procAfter = Process.GetProcessesByName(target.AppName).FirstOrDefault();
                    double ramAfterMB = procAfter != null ? procAfter.WorkingSet64 / (1024.0 * 1024.0) : ramBeforeMB;

                    rpsList.Add(res.RequestsPerSecond);
                    meanMsList.Add(res.AverageLatency);
                    p95MsList.Add(res.Percentile95Latency);
                    p99MsList.Add(res.Percentile99Latency);
                    failCountList.Add(res.Failure);
                    memoryMBList.Add(ramAfterMB);
                }

                rpsList.Sort();
                meanMsList.Sort();
                p95MsList.Sort();
                p99MsList.Sort();
                memoryMBList.Sort();

                double medianRps = rpsList[repeats / 2];
                double avgRps = rpsList.Average();
                double stdDevRps = Math.Sqrt(rpsList.Select(x => Math.Pow(x - avgRps, 2)).Sum() / repeats);
                double medianMean = meanMsList[repeats / 2];
                double medianP95 = p95MsList[repeats / 2];
                double medianP99 = p99MsList[repeats / 2];
                double medianRamMB = memoryMBList[repeats / 2];

                list.Add(new SteppedScenarioResult(target.Name, cfg.Concurrency, medianRps, stdDevRps, medianMean, medianP95, medianP99, failCountList.Max(), medianRamMB));
                Console.WriteLine($"    Concurrency {cfg.Concurrency,4}: RPS = {medianRps,7:N0} (±{stdDevRps:F0}), Mean = {medianMean,6:F2}ms, P95 = {medianP95,6:F2}ms, Fail = {failCountList.Max()}, RAM = {medianRamMB:F1}MB");
            }
        }

        return list;
    }

    private static async Task WarmupEndpointsAsync(HttpClient client)
    {
        Console.WriteLine("[ISINMA] Web API uç noktalarına ön ısıtma isteği gönderiliyor...");
        try
        {
            await client.PostAsJsonAsync("http://localhost:5005/api/Benchmark/command", new { Message = "warmup" });
            await client.GetAsync("http://localhost:5005/api/Benchmark/query?id=1");
            await client.PostAsJsonAsync("http://localhost:5020/api/Benchmark/command", new { Message = "warmup" });
            await client.GetAsync("http://localhost:5020/api/Benchmark/query?id=1");
            await client.PostAsJsonAsync("http://localhost:5030/api/Benchmark/command", new { Message = "warmup" });
            await client.GetAsync("http://localhost:5030/api/Benchmark/query?id=1");
            Console.WriteLine("[ISINMA] Ön ısıtma başarıyla tamamlandı.\n");
        }
        catch (Exception ex)
        {
            Console.WriteLine($"[UYARI] Isınma isteği sırasında hata: {ex.Message}");
        }
    }

    private static void PrintSummaryTable(string title, List<ScenarioSummary> results)
    {
        Console.WriteLine($"\n===================================================================================================================================");
        Console.WriteLine($" {title}");
        Console.WriteLine($"===================================================================================================================================");
        Console.WriteLine($"| {"Senaryo / Mimari",-44} | {"Medyan RPS",12} | {"StdDev RPS",12} | {"Mean (ms)",10} | {"P95 (ms)",9} | {"P99 (ms)",9} | {"Hata (Count)",12} | {"Sunucu RAM (MB)",16} |");
        Console.WriteLine($"|{new string('-', 46)}|{new string('-', 14)}|{new string('-', 14)}|{new string('-', 12)}|{new string('-', 11)}|{new string('-', 11)}|{new string('-', 14)}|{new string('-', 18)}|");

        foreach (var r in results)
        {
            Console.WriteLine($"| {r.Name,-44} | {r.MedianRps,12:N0} | {r.StdDevRps,12:F2} | {r.MedianMeanMs,10:F2} | {r.MedianP95Ms,9:F2} | {r.MedianP99Ms,9:F2} | {r.MaxFailCount,12} | {r.MedianRamMB,16:F1} |");
        }
        Console.WriteLine($"===================================================================================================================================\n");
    }

    private static void PrintSteppedSummaryTable(string title, List<SteppedScenarioResult> results)
    {
        Console.WriteLine($"\n===================================================================================================================================");
        Console.WriteLine($" {title}");
        Console.WriteLine($"===================================================================================================================================");
        Console.WriteLine($"| {"Senaryo",-24} | {"Concurrency",12} | {"Medyan RPS",12} | {"StdDev RPS",10} | {"Mean (ms)",10} | {"P95 (ms)",9} | {"P99 (ms)",9} | {"Hata",8} | {"RAM (MB)",10} |");
        Console.WriteLine($"|{new string('-', 26)}|{new string('-', 14)}|{new string('-', 14)}|{new string('-', 12)}|{new string('-', 12)}|{new string('-', 11)}|{new string('-', 11)}|{new string('-', 10)}|{new string('-', 12)}|");

        foreach (var r in results)
        {
            Console.WriteLine($"| {r.ScenarioName,-24} | {r.Concurrency,12} | {r.Rps,12:N0} | {r.StdDevRps,10:F0} | {r.MeanMs,10:F2} | {r.P95Ms,9:F2} | {r.P99Ms,9:F2} | {r.Failures,8} | {r.RamMB,10:F1} |");
        }
        Console.WriteLine($"===================================================================================================================================\n");
    }

    private record ScenarioDefinition(string Name, string ProcessName, Func<LoadExecutionPlan> CreatePlan);

    private record ScenarioSummary(
        string Name,
        double MedianRps,
        double StdDevRps,
        double MedianMeanMs,
        double MedianP95Ms,
        double MedianP99Ms,
        long MaxFailCount,
        double MedianRamMB
    );

    private record SteppedScenarioResult(
        string ScenarioName,
        int Concurrency,
        double Rps,
        double StdDevRps,
        double MeanMs,
        double P95Ms,
        double P99Ms,
        long Failures,
        double RamMB
    );
}
