using Microsoft.Extensions.Hosting;
using Microsoft.Extensions.Logging;
using System;
using System.Collections.Generic;
using System.Linq;
using System.Text;
using System.Threading.Tasks;

namespace EticaretApp.Infrastructure.BackgroundServices;

public class TimedHostedService : BackgroundService
{
    private readonly ILogger<TimedHostedService> _logger;
    private readonly TimeSpan _startTime; // Scheduled start time
    private readonly TimeSpan _interval; // Interval for repeating tasks

    public TimedHostedService(ILogger<TimedHostedService> logger)
    {
        _logger = logger ?? throw new ArgumentNullException(nameof(logger));
        _startTime = new TimeSpan(4, 0, 0); // Start time at 4:00 AM UTC
        _interval = TimeSpan.FromHours(1); // Run every 1 Hour
    }

    protected override async Task ExecuteAsync(CancellationToken stoppingToken)
    {
        _logger.LogInformation("Timed Hosted Service is starting.");

        var initialDelay = GetInitialDelay(_startTime);
        _logger.LogInformation("Service will start at {StartTime} (in {InitialDelay}).",
            DateTime.UtcNow.Add(initialDelay), initialDelay);

        await Task.Delay(initialDelay, stoppingToken);

        while (!stoppingToken.IsCancellationRequested)
        {
            try
            {
                await PerformTaskAsync(stoppingToken);
            }
            catch (Exception ex)
            {
                _logger.LogError(ex, "An error occurred while executing the hosted service task.");
            }

            _logger.LogInformation("Waiting until {NextRun} for the next execution.",
                DateTime.UtcNow.Add(_interval));

            await Task.Delay(_interval, stoppingToken);
        }

        _logger.LogInformation("Timed Hosted Service is stopping.");
    }

    private TimeSpan GetInitialDelay(TimeSpan startTime)
    {
        var now = DateTime.UtcNow;
        var todayStartTime = DateTime.UtcNow.Date.Add(startTime);

        return now < todayStartTime
            ? todayStartTime - now
            : todayStartTime.AddDays(1) - now;
    }

    private async Task PerformTaskAsync(CancellationToken stoppingToken)
    {
        _logger.LogInformation("Performing the scheduled task at {Time}.", DateTime.UtcNow);

        await Task.Delay(1000, stoppingToken);

        _logger.LogInformation("Task completed at {Time}.", DateTime.UtcNow);
    }
}
