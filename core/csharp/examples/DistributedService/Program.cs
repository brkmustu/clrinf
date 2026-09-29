using System.Text;
using System.Text.Json;
using ClrinfCS.Adapters.Sqlite;
using ClrinfCS.Core;
using ClrinfCS.Examples.Documents;

if (args.Length != 3 || args[0] is not ("serve" or "send"))
    throw new ArgumentException("Usage: DistributedService <serve|send> <database-file> <receiver-url>");

// Local demonstration only: real ingress must derive the tenant from verified authentication.
var context = new RequestContext("tenant-demo", Guid.NewGuid().ToString(), Guid.NewGuid().ToString());
var store = new SqliteStore(args[1]);
if (args[0] == "send")
{
    var dispatcher = new Dispatcher.Builder().Register(new ApproveDocumentHandler(store)).Build();
    await dispatcher.SendAsync(new ApproveDocument("document-example"), context);
    using var client = new HttpClient { BaseAddress = new Uri(args[2]), Timeout = TimeSpan.FromSeconds(10) };
    foreach (var delivery in store.Claim(context, 100, TimeSpan.FromSeconds(30)))
    {
        using var content = new StringContent(ContractJson.Serialize(delivery.Event), Encoding.UTF8, "application/cloudevents+json");
        using var response = await client.PostAsync("/events", content);
        if (!response.IsSuccessStatusCode)
        {
            if (!store.Abandon(context, delivery, TimeSpan.FromSeconds(5)))
                throw new InvalidOperationException("The outbox lease expired before retry could be scheduled.");
            response.EnsureSuccessStatusCode();
        }
        if (!store.Complete(context, delivery)) throw new InvalidOperationException("The outbox lease expired.");
        Console.WriteLine($"Delivered {delivery.Event.Id}");
    }
    return;
}

var builder = WebApplication.CreateBuilder();
builder.WebHost.UseUrls(args[2]);
var app = builder.Build();
app.MapGet("/health", () => Results.Ok(new { status = "ready" }));
app.MapPost("/events", async (HttpRequest request) =>
{
    using var reader = new StreamReader(request.Body);
    var json = await reader.ReadToEndAsync(request.HttpContext.RequestAborted);
    CloudEvent envelope;
    try
    {
        envelope = ContractJson.Deserialize<CloudEvent>(json);
        var applied = DocumentProjection.Apply(store, context, envelope);
        return Results.Ok(new { applied });
    }
    catch (InboxConflictException)
    {
        return Results.Content(
            ContractJson.Serialize(ErrorEnvelope.From(context, "IDEMPOTENCY_CONFLICT", "Event identity conflicts with recorded content.")),
            "application/json", statusCode: StatusCodes.Status409Conflict);
    }
    catch (JsonException)
    {
        return InvalidContract();
    }
    catch (ArgumentException)
    {
        return InvalidContract();
    }
});
app.MapGet("/documents/{id}", (string id) =>
    Results.Ok(new { status = store.Execute(context, tx => tx.Get($"projection/{id}")) }));
await app.RunAsync();

IResult InvalidContract() => Results.Content(
    ContractJson.Serialize(ErrorEnvelope.From(context, "INVALID_CONTRACT", "Invalid event or tenant.")),
    "application/json", statusCode: StatusCodes.Status400BadRequest);
