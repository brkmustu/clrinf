using ClrinfCS.Core;
using Xunit;

namespace ClrinfCS.Tests;

public class DispatcherTests
{
    private static readonly RequestContext Context = new("tenant", "workflow", "request");
    private sealed record Echo(string Value) : IRequest<string>;
    private sealed class EchoHandler : IRequestHandler<Echo, string>
    {
        public ValueTask<string> HandleAsync(Echo request, RequestContext context, CancellationToken cancellationToken)
        {
            cancellationToken.ThrowIfCancellationRequested();
            return ValueTask.FromResult($"{context.TenantId}:{request.Value}");
        }
    }
    private sealed class Trace(string label, List<string> calls) : IPipelineBehavior<Echo, string>
    {
        public async ValueTask<string> HandleAsync(Echo request, RequestContext context,
            CancellationToken cancellationToken, RequestHandlerDelegate<string> next)
        {
            calls.Add(label + ":before");
            var result = await next();
            calls.Add(label + ":after");
            return result;
        }
    }
    private sealed class Twice : IPipelineBehavior<Echo, string>
    {
        public async ValueTask<string> HandleAsync(Echo request, RequestContext context,
            CancellationToken cancellationToken, RequestHandlerDelegate<string> next) => await next() + await next();
    }

    [Fact]
    public async Task PreservesPipelineOrderAndContext()
    {
        var calls = new List<string>();
        var dispatcher = new Dispatcher.Builder().Register(new EchoHandler(),
            new Trace("outer", calls), new Trace("inner", calls)).Build();
        Assert.Equal("tenant:hello", await dispatcher.SendAsync(new Echo("hello"), Context));
        Assert.Equal(["outer:before", "inner:before", "inner:after", "outer:after"], calls);
    }

    [Fact]
    public async Task ReinvokingNextRunsWholeDownstreamPipeline()
    {
        var calls = new List<string>();
        var dispatcher = new Dispatcher.Builder().Register(new EchoHandler(),
            new Twice(), new Trace("inner", calls)).Build();
        Assert.Equal("tenant:xtenant:x", await dispatcher.SendAsync(new Echo("x"), Context));
        Assert.Equal(["inner:before", "inner:after", "inner:before", "inner:after"], calls);
    }

    [Fact]
    public async Task RegistrationIsExplicitAndCancellationPropagates()
    {
        var builder = new Dispatcher.Builder();
        var empty = builder.Build();
        builder.Register(new EchoHandler());
        Assert.Throws<InvalidOperationException>(() => builder.Register(new EchoHandler()));
        await Assert.ThrowsAsync<InvalidOperationException>(() => empty.SendAsync(new Echo("x"), Context).AsTask());
        await Assert.ThrowsAnyAsync<OperationCanceledException>(() =>
            builder.Build().SendAsync(new Echo("x"), Context, new CancellationToken(true)).AsTask());
        var dispatcher = builder.Build();
        var responses = await Task.WhenAll(Enumerable.Range(0, 20)
            .Select(i => dispatcher.SendAsync(new Echo(i.ToString()), Context).AsTask()));
        Assert.Equal(20, responses.Distinct().Count());
    }
}
