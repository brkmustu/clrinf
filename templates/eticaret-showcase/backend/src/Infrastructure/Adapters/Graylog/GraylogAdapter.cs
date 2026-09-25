using System;
using System.Collections.Concurrent;
using System.Diagnostics;
using System.IO;
using System.Net.Sockets;
using System.Text;
using System.Text.Json;
using System.Threading;
using System.Threading.Channels;
using System.Threading.Tasks;
using Microsoft.Extensions.DependencyInjection;
using Microsoft.Extensions.Logging;

namespace EticaretApp.Infrastructure.Adapters.Graylog;

public class GraylogLoggerOptions
{
    public string Host { get; set; } = "127.0.0.1";
    public int Port { get; set; } = 12201;
    public string ApplicationName { get; set; } = "EticaretApp";
    public LogLevel MinimumLevel { get; set; } = LogLevel.Information;
    public int MaxQueueCapacity { get; set; } = 1000;
}

public sealed class GraylogLoggerProvider : ILoggerProvider
{
    private readonly GraylogLoggerOptions _options;
    private readonly ConcurrentDictionary<string, GraylogLogger> _loggers = new(StringComparer.OrdinalIgnoreCase);
    private readonly Channel<byte[]> _channel;
    private readonly CancellationTokenSource _cts = new();

    public GraylogLoggerProvider(GraylogLoggerOptions options)
    {
        Debug.Assert(options != null, "Graylog options must not be null");
        Debug.Assert(options.Port > 0 && options.Port <= 65535, "Port must be a valid UDP port number");
        Debug.Assert(options.MaxQueueCapacity > 0, "MaxQueueCapacity must be positive");

        _options = options;
        _channel = Channel.CreateBounded<byte[]>(new BoundedChannelOptions(options.MaxQueueCapacity)
        {
            SingleReader = true,
            SingleWriter = false,
            FullMode = BoundedChannelFullMode.DropOldest
        });
        Task.Run(ProcessQueueAsync);
    }

    public ILogger CreateLogger(string categoryName)
    {
        Debug.Assert(!string.IsNullOrEmpty(categoryName), "Category name must not be null or empty");
        return _loggers.GetOrAdd(categoryName, name => new GraylogLogger(name, _options, _channel.Writer));
    }

    private async Task ProcessQueueAsync()
    {
        using var client = new UdpClient();
        while (!_cts.Token.IsCancellationRequested)
        {
            try
            {
                if (await _channel.Reader.WaitToReadAsync(_cts.Token))
                {
                    while (_channel.Reader.TryRead(out var payload))
                    {
                        Debug.Assert(payload != null && payload.Length > 0, "Payload bytes must not be empty");
                        await client.SendAsync(payload, payload.Length, _options.Host, _options.Port);
                    }
                }
            }
            catch (OperationCanceledException)
            {
                break;
            }
            catch
            {
                // Fallback for UDP socket errors to prevent logger thread crash
            }
        }
    }

    public void Dispose()
    {
        _cts.Cancel();
        _cts.Dispose();
    }
}

public sealed class GraylogLogger : ILogger
{
    private readonly string _categoryName;
    private readonly GraylogLoggerOptions _options;
    private readonly ChannelWriter<byte[]> _writer;

    public GraylogLogger(string categoryName, GraylogLoggerOptions options, ChannelWriter<byte[]> writer)
    {
        Debug.Assert(!string.IsNullOrEmpty(categoryName), "Category name must not be empty");
        Debug.Assert(options != null, "Options must not be null");
        Debug.Assert(writer != null, "Writer must not be null");

        _categoryName = categoryName;
        _options = options;
        _writer = writer;
    }

    public IDisposable? BeginScope<TState>(TState state) where TState : notnull => null;

    public bool IsEnabled(LogLevel logLevel) => logLevel >= _options.MinimumLevel && logLevel != LogLevel.None;

    public void Log<TState>(LogLevel logLevel, EventId eventId, TState state, Exception? exception, Func<TState, Exception?, string> formatter)
    {
        Debug.Assert(formatter != null, "Formatter delegate must not be null");

        if (!IsEnabled(logLevel)) return;

        string message = formatter(state, exception);
        if (string.IsNullOrEmpty(message) && exception == null) return;

        var gelfPayload = new
        {
            version = "1.1",
            host = Environment.MachineName,
            short_message = message,
            full_message = exception?.ToString(),
            timestamp = DateTimeOffset.UtcNow.ToUnixTimeMilliseconds() / 1000.0,
            level = MapSyslogLevel(logLevel),
            _facility = _options.ApplicationName,
            _logger_name = _categoryName
        };

        byte[] bytes = JsonSerializer.SerializeToUtf8Bytes(gelfPayload);
        Debug.Assert(bytes.Length > 0, "Serialized GELF bytes must be non-empty");

        _writer.TryWrite(bytes);
    }

    private static int MapSyslogLevel(LogLevel level) => level switch
    {
        LogLevel.Trace or LogLevel.Debug => 7,
        LogLevel.Information => 6,
        LogLevel.Warning => 4,
        LogLevel.Error => 3,
        LogLevel.Critical => 2,
        _ => 1
    };
}

public static class GraylogLoggerExtensions
{
    public static ILoggingBuilder AddGraylog(this ILoggingBuilder builder, Action<GraylogLoggerOptions>? configure = null)
    {
        Debug.Assert(builder != null, "Logging builder must not be null");

        var options = new GraylogLoggerOptions();
        configure?.Invoke(options);
        builder.Services.AddSingleton<ILoggerProvider>(new GraylogLoggerProvider(options));
        return builder;
    }
}
