namespace ClrinfCS.Core;

public interface ITenantStorage
{
    string? Get(string key);
    void Set(string key, string value);
    bool Delete(string key);
}

public interface IInbox
{
    /// <summary>Identity-only deduplication; the caller must guarantee immutable event content.</summary>
    bool TryAccept(string consumer, string source, string eventId);

    /// <summary>Reserve an event and reject conflicting content for an existing identity.</summary>
    bool TryAccept(string consumer, CloudEvent envelope) =>
        throw new NotSupportedException("This inbox adapter does not support content-aware deduplication.");
}

public sealed class InboxConflictException(string message) : InvalidOperationException(message);

public interface IOutboxWriter
{
    void Enqueue(CloudEvent envelope);
}

public interface IStorageTransaction : ITenantStorage, IInbox, IOutboxWriter
{
    RequestContext Context { get; }
}

public interface ITransactionalStore
{
    /// <summary>Commit all writes on success; rollback state, inbox and outbox on exception.</summary>
    T Execute<T>(RequestContext context, Func<IStorageTransaction, T> operation,
        CancellationToken cancellationToken = default);
}

public sealed record OutboxDelivery(CloudEvent Event, string LeaseToken, int Attempts);

public interface IOutbox
{
    IReadOnlyList<OutboxDelivery> Claim(RequestContext context, int limit, TimeSpan lease);
    bool Complete(RequestContext context, OutboxDelivery delivery);
    bool Abandon(RequestContext context, OutboxDelivery delivery, TimeSpan retryDelay);
}
