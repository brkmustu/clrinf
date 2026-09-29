using System;
using System.Threading;
using System.Threading.Tasks;
using Microsoft.AspNetCore.Mvc;
using Microsoft.Extensions.Logging;
using PureNativeApp.Common;
using PureNativeApp.Features.Benchmark;

namespace PureNativeApp.Controllers;

[ApiController]
[Route("api/[controller]")]
public class BenchmarkController : ControllerBase
{
    private readonly ICommandHandler<BenchmarkCommand, string> _commandHandler;
    private readonly IQueryHandler<BenchmarkQuery, string> _queryHandler;
    private readonly ILogger<BenchmarkController> _logger;

    public BenchmarkController(
        ICommandHandler<BenchmarkCommand, string> commandHandler,
        IQueryHandler<BenchmarkQuery, string> queryHandler,
        ILogger<BenchmarkController> logger)
    {
        _commandHandler = commandHandler;
        _queryHandler = queryHandler;
        _logger = logger;
    }

    [HttpPost("command")]
    public async ValueTask<IActionResult> Command([FromBody] BenchmarkCommand command, CancellationToken cancellationToken)
    {
        try
        {
            var result = await _commandHandler.HandleAsync(command, cancellationToken);
            return Ok(result);
        }
        catch (Exception ex)
        {
            _logger.LogError(ex, "[PURE NATIVE COMMAND ERROR] {ErrorType}: {Message}", ex.GetType().Name, ex.Message);
            return StatusCode(500, ex.Message);
        }
    }

    [HttpGet("query")]
    public async ValueTask<IActionResult> Query([FromQuery] int id, CancellationToken cancellationToken)
    {
        try
        {
            var result = await _queryHandler.HandleAsync(new BenchmarkQuery(id), cancellationToken);
            return Ok(result);
        }
        catch (Exception ex)
        {
            _logger.LogError(ex, "[PURE NATIVE QUERY ERROR] {ErrorType}: {Message}", ex.GetType().Name, ex.Message);
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
