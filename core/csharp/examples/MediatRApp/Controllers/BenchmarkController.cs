using System;
using System.Threading.Tasks;
using MediatR;
using MediatRApp.Features.Benchmark;
using Microsoft.AspNetCore.Mvc;
using Microsoft.Extensions.Logging;

namespace MediatRApp.Controllers;

[ApiController]
[Route("api/[controller]")]
public class BenchmarkController : ControllerBase
{
    private readonly IMediator _mediator;
    private readonly ILogger<BenchmarkController> _logger;

    public BenchmarkController(IMediator mediator, ILogger<BenchmarkController> logger)
    {
        _mediator = mediator;
        _logger = logger;
    }

    [HttpPost("command")]
    public async Task<IActionResult> ExecuteCommand([FromBody] BenchmarkCommand command)
    {
        try
        {
            var result = await _mediator.Send(command);
            return Ok(result);
        }
        catch (Exception ex)
        {
            _logger.LogError(ex, "[MEDIATR COMMAND ERROR] {ErrorType}: {Message}", ex.GetType().Name, ex.Message);
            return StatusCode(500, ex.Message);
        }
    }

    [HttpGet("query")]
    public async Task<IActionResult> ExecuteQuery([FromQuery] int id)
    {
        try
        {
            var result = await _mediator.Send(new BenchmarkQuery(id));
            return Ok(result);
        }
        catch (Exception ex)
        {
            _logger.LogError(ex, "[MEDIATR QUERY ERROR] {ErrorType}: {Message}", ex.GetType().Name, ex.Message);
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
