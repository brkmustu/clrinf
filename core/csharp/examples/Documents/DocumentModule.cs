using System.Text.Json;
using ClrinfCS.Core;

namespace ClrinfCS.Examples.Documents;

public sealed record ApproveDocument(string DocumentId) : IRequest<string>;

public sealed class ApproveDocumentHandler(ITransactionalStore store) : IRequestHandler<ApproveDocument, string>
{
    public ValueTask<string> HandleAsync(ApproveDocument request, RequestContext context, CancellationToken cancellationToken)
    {
        ArgumentException.ThrowIfNullOrWhiteSpace(request.DocumentId);
        var id = store.Execute(context, tx =>
        {
            var envelope = CloudEvent.Create(context, "clrinf/document-service",
                DocumentProjection.EventType, JsonSerializer.SerializeToElement(new { document_id = request.DocumentId }));
            tx.Set($"documents/{request.DocumentId}", "approved");
            tx.Enqueue(envelope);
            return envelope.Id;
        }, cancellationToken);
        return ValueTask.FromResult(id);
    }
}

public static class DocumentProjection
{
    public const string EventType = "com.clrinf.documents.DocumentApproved.v1";

    public static bool Apply(ITransactionalStore store, RequestContext trustedContext, CloudEvent envelope)
    {
        if (envelope.TenantId != trustedContext.TenantId)
            throw new ArgumentException("The event tenant does not match the authenticated tenant.", nameof(envelope));
        if (envelope.Type != EventType)
            throw new ArgumentException($"Unsupported event type: {envelope.Type}", nameof(envelope));
        if (!envelope.Data.TryGetProperty("document_id", out var property) || property.ValueKind != JsonValueKind.String ||
            string.IsNullOrWhiteSpace(property.GetString()))
            throw new ArgumentException("document_id must be a nonempty string.", nameof(envelope));
        var documentId = property.GetString()!;
        return store.Execute(envelope.ToContext(), tx =>
        {
            if (!tx.TryAccept("document-projection", envelope)) return false;
            tx.Set($"projection/{documentId}", "approved");
            return true;
        });
    }
}
