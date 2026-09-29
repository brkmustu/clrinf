using System.Text.Json;
using System.Text.Json.Serialization;

namespace ClrinfCS.Core;

public sealed record RequestContext
{
    [JsonConstructor]
    public RequestContext(string tenantId, string correlationId, string causationId)
    {
        TenantId = ContractGuard.Text(tenantId, nameof(tenantId));
        CorrelationId = ContractGuard.Text(correlationId, nameof(correlationId));
        CausationId = ContractGuard.Text(causationId, nameof(causationId));
    }

    [JsonPropertyName("tenant_id")] public string TenantId { get; }
    [JsonPropertyName("correlation_id")] public string CorrelationId { get; }
    [JsonPropertyName("causation_id")] public string CausationId { get; }

    public RequestContext CausedBy(string eventId) => new(TenantId, CorrelationId, eventId);
}

public sealed record ErrorEnvelope
{
    [JsonConstructor]
    public ErrorEnvelope(string errorCode, string message, string correlationId, string tenantId,
        bool retryable, JsonElement? details = null)
    {
        ErrorCode = ContractGuard.Text(errorCode, nameof(errorCode));
        Message = ContractGuard.Text(message, nameof(message));
        CorrelationId = ContractGuard.Text(correlationId, nameof(correlationId));
        TenantId = ContractGuard.Text(tenantId, nameof(tenantId));
        Retryable = retryable;
        Details = details.HasValue ? ContractGuard.Object(details.Value, nameof(details)) : null;
    }

    [JsonPropertyName("error_code")] public string ErrorCode { get; }
    [JsonPropertyName("message")] public string Message { get; }
    [JsonPropertyName("correlation_id")] public string CorrelationId { get; }
    [JsonPropertyName("tenant_id")] public string TenantId { get; }
    [JsonPropertyName("retryable")] public bool Retryable { get; }
    [JsonPropertyName("details")] public JsonElement? Details { get; }

    public static ErrorEnvelope From(RequestContext context, string code, string message,
        bool retryable = false, JsonElement? details = null) =>
        new(code, message, context.CorrelationId, context.TenantId, retryable, details);
}

public sealed record CloudEvent
{
    [JsonConstructor]
    public CloudEvent(string specVersion, string id, string source, string type, DateTimeOffset time,
        string dataContentType, string tenantId, string correlationId, string causationId,
        JsonElement data, bool? isError = null, bool? isCompensation = null, string? subject = null)
    {
        if (specVersion != "1.0") throw new ArgumentException("CloudEvents specversion must be 1.0.", nameof(specVersion));
        if (dataContentType != "application/json") throw new ArgumentException("datacontenttype must be application/json.", nameof(dataContentType));
        if (time == default) throw new ArgumentException("An event timestamp is required.", nameof(time));
        SpecVersion = specVersion;
        Id = ContractGuard.Text(id, nameof(id));
        Source = ContractGuard.Text(source, nameof(source));
        Type = ContractGuard.Text(type, nameof(type));
        Time = time;
        DataContentType = dataContentType;
        TenantId = ContractGuard.Text(tenantId, nameof(tenantId));
        CorrelationId = ContractGuard.Text(correlationId, nameof(correlationId));
        CausationId = ContractGuard.Text(causationId, nameof(causationId));
        Data = ContractGuard.Object(data, nameof(data));
        IsError = isError;
        IsCompensation = isCompensation;
        Subject = subject;
    }

    [JsonPropertyName("specversion")] public string SpecVersion { get; }
    [JsonPropertyName("id")] public string Id { get; }
    [JsonPropertyName("source")] public string Source { get; }
    [JsonPropertyName("type")] public string Type { get; }
    [JsonPropertyName("time")] public DateTimeOffset Time { get; }
    [JsonPropertyName("datacontenttype")] public string DataContentType { get; }
    [JsonPropertyName("tenantid")] public string TenantId { get; }
    [JsonPropertyName("correlationid")] public string CorrelationId { get; }
    [JsonPropertyName("causationid")] public string CausationId { get; }
    [JsonPropertyName("data")] public JsonElement Data { get; }
    [JsonPropertyName("is_error")] public bool? IsError { get; }
    [JsonPropertyName("is_compensation")] public bool? IsCompensation { get; }
    [JsonPropertyName("subject")] public string? Subject { get; }

    public RequestContext ToContext() => new(TenantId, CorrelationId, Id);

    public static CloudEvent Create(RequestContext context, string source, string type, JsonElement data,
        string? id = null, TimeProvider? clock = null) =>
        new("1.0", id ?? Guid.NewGuid().ToString(), source, type,
            (clock ?? TimeProvider.System).GetUtcNow(), "application/json",
            context.TenantId, context.CorrelationId, context.CausationId, data);
}

public static class ContractJson
{
    private static readonly JsonSerializerOptions Options = new()
    {
        RespectRequiredConstructorParameters = true,
        RespectNullableAnnotations = true,
        DefaultIgnoreCondition = JsonIgnoreCondition.WhenWritingNull
    };

    public static string Serialize<T>(T value) where T : notnull => JsonSerializer.Serialize(value, Options);

    public static T Deserialize<T>(string json) where T : notnull =>
        JsonSerializer.Deserialize<T>(json, Options) ?? throw new JsonException("A contract cannot be null.");
}

internal static class ContractGuard
{
    internal static string Text(string value, string name)
    {
        ArgumentException.ThrowIfNullOrWhiteSpace(value, name);
        return value;
    }

    internal static JsonElement Object(JsonElement value, string name)
    {
        if (value.ValueKind != JsonValueKind.Object)
            throw new ArgumentException("Contract data must be a JSON object.", name);
        return value.Clone();
    }
}
