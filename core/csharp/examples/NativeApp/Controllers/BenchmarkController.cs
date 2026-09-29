using System;
using System.Threading;
using System.Threading.Tasks;
using Microsoft.AspNetCore.Mvc;
using Microsoft.Extensions.Logging;
using NativeApp.Common;
using NativeApp.Features.Benchmark;

namespace NativeApp.Controllers;

[ApiController]
[Route("api/[controller]")]
public class BenchmarkController : ControllerBase
{
    private readonly IDispatcher _dispatcher;
    private readonly ILogger<BenchmarkController> _logger;

    public BenchmarkController(IDispatcher dispatcher, ILogger<BenchmarkController> logger)
    {
        _dispatcher = dispatcher;
        _logger = logger;
    }

    [HttpPost("command")]
    public async ValueTask<IActionResult> Command([FromBody] BenchmarkCommand command, CancellationToken cancellationToken)
    {
        try
        {
            var result = await _dispatcher.SendAsync(command, cancellationToken);
            return Ok(result);
        }
        catch (Exception ex)
        {
            _logger.LogError(ex, "[NATIVE COMMAND ERROR] {ErrorType}: {Message}", ex.GetType().Name, ex.Message);
            return StatusCode(500, ex.Message);
        }
    }

    [HttpGet("query")]
    public async ValueTask<IActionResult> Query([FromQuery] int id, CancellationToken cancellationToken)
    {
        try
        {
            var result = await _dispatcher.QueryAsync(new BenchmarkQuery(id), cancellationToken);
            return Ok(result);
        }
        catch (Exception ex)
        {
            _logger.LogError(ex, "[NATIVE QUERY ERROR] {ErrorType}: {Message}", ex.GetType().Name, ex.Message);
            return StatusCode(500, ex.Message);
        }
    }

    [HttpGet("gc-stats")]
    public IActionResult GetGcStats()
    {
        int gen0 = GC.CollectionCount(0);
        int gen1 = GC.CollectionCount(1);
        int gen2 = GC.CollectionCount(2);
        long allocatedBytes = GC.GetTotalAllocatedBytes(precise: false);
        var proc = System.Diagnostics.Process.GetCurrentProcess();

        return Ok(new
        {
            totalAllocatedBytes = allocatedBytes,
            gen0 = gen0,
            gen1 = gen1,
            gen2 = gen2,
            workingSetMb = proc.WorkingSet64 / (1024.0 * 1024.0)
        });
    }

    [HttpPost("gc-collect")]
    public IActionResult ForceGcCollect()
    {
        GC.Collect();
        GC.WaitForPendingFinalizers();
        GC.Collect();
        return Ok(new { status = "GC Collected" });
    }
}
