using System.Text.Json;
using System.Text.Json.Nodes;
using ClrinfCS.Core;
using Xunit;

namespace ClrinfCS.Tests;

public class ContractTests
{
    internal static string FixtureDirectory
    {
        get
        {
            var configured = Environment.GetEnvironmentVariable("CLRINF_CONFORMANCE_DIR");
            if (configured is not null)
                return Directory.Exists(configured) ? configured
                    : throw new DirectoryNotFoundException($"CLRINF_CONFORMANCE_DIR does not exist: {configured}");
            for (var directory = new DirectoryInfo(AppContext.BaseDirectory); directory is not null; directory = directory.Parent)
            {
                var path = Path.Combine(directory.FullName, "tests", "conformance", "fixtures");
                if (Directory.Exists(path)) return path;
            }
            throw new DirectoryNotFoundException("Shared conformance fixtures are required. Set CLRINF_CONFORMANCE_DIR to their directory.");
        }
    }

    [Theory]
    [InlineData("context")]
    [InlineData("error")]
    [InlineData("event")]
    public void SharedValidContractRoundTrips(string kind)
    {
        using var fixtures = JsonDocument.Parse(File.ReadAllText(Path.Combine(FixtureDirectory, "valid.json")));
        var original = fixtures.RootElement.GetProperty(kind).GetRawText();
        var serialized = RoundTrip(kind, original);
        var expected = JsonNode.Parse(original)!;
        var actual = JsonNode.Parse(serialized)!;
        if (kind == "event")
        {
            Assert.Equal(DateTimeOffset.Parse(expected["time"]!.GetValue<string>()),
                DateTimeOffset.Parse(actual["time"]!.GetValue<string>()));
            expected["time"] = actual["time"]!.GetValue<string>();
        }
        Assert.True(JsonNode.DeepEquals(expected, actual), serialized);
    }

    [Theory]
    [InlineData("context")]
    [InlineData("error")]
    [InlineData("event")]
    public void SharedInvalidContractsAreRejected(string kind)
    {
        using var fixtures = JsonDocument.Parse(File.ReadAllText(Path.Combine(FixtureDirectory, "invalid.json")));
        foreach (var test in fixtures.RootElement.GetProperty(kind).EnumerateArray())
        {
            var exception = Record.Exception(() => RoundTrip(kind, test.GetProperty("value").GetRawText()));
            Assert.True(exception is ArgumentException or JsonException,
                $"Expected validation failure for {kind}: {test.GetProperty("name")}");
        }
    }

    [Fact]
    public void RequiredBooleanCannotDefaultWhenMissing()
    {
        Assert.Throws<JsonException>(() => ContractJson.Deserialize<ErrorEnvelope>(
            """{"error_code":"X","message":"failed","tenant_id":"t","correlation_id":"c"}"""));
    }

    [Fact]
    public void ContextAndPayloadAreValidatedAndOwned()
    {
        Assert.Throws<ArgumentException>(() => new RequestContext("", "c", "r"));
        var context = new RequestContext("tenant", "workflow", "request");
        CloudEvent envelope;
        using (var payload = JsonDocument.Parse("""{"value":1}"""))
            envelope = CloudEvent.Create(context, "clrinf/documents", "documents.Changed.v1", payload.RootElement, "event");
        Assert.Equal(1, envelope.Data.GetProperty("value").GetInt32());
        Assert.Equal(context.CausedBy("event"), envelope.ToContext());
        var error = ErrorEnvelope.From(context, "NOT_FOUND", "Missing");
        Assert.Equal(context.TenantId, error.TenantId);
        Assert.False(error.Retryable);
        Assert.DoesNotContain("details", ContractJson.Serialize(error));
        using var invalid = JsonDocument.Parse("[]");
        Assert.Throws<ArgumentException>(() => ErrorEnvelope.From(context, "X", "failed", details: invalid.RootElement));
    }

    private static string RoundTrip(string kind, string value) => kind switch
    {
        "context" => ContractJson.Serialize(ContractJson.Deserialize<RequestContext>(value)),
        "error" => ContractJson.Serialize(ContractJson.Deserialize<ErrorEnvelope>(value)),
        "event" => ContractJson.Serialize(ContractJson.Deserialize<CloudEvent>(value)),
        _ => throw new ArgumentOutOfRangeException(nameof(kind))
    };
}
