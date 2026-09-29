using System.Text.Json;
using ClrinfCS.Adapters.Sqlite;
using ClrinfCS.Core;
using ClrinfCS.Examples.Documents;
using Microsoft.Data.Sqlite;
using Xunit;

namespace ClrinfCS.Tests;

public sealed class SqliteStoreTests : IDisposable
{
    private readonly string path = Path.Combine(Path.GetTempPath(), $"clrinf-{Guid.NewGuid():N}.db");
    private readonly Clock clock = new();
    private static readonly RequestContext Context = new("tenant-a", "workflow", "request");
    private static readonly RequestContext Other = new("tenant-b", "workflow", "request");
    private sealed class Clock : TimeProvider
    {
        public DateTimeOffset Now { get; set; } = new(2026, 9, 9, 0, 0, 0, TimeSpan.Zero);
        public override DateTimeOffset GetUtcNow() => Now;
    }
    private CloudEvent Event(RequestContext? context = null, string id = "event", string data = """{"document_id":"doc"}""") =>
        CloudEvent.Create(context ?? Context, "clrinf/documents", "documents.Changed.v1",
            JsonSerializer.Deserialize<JsonElement>(data), id, clock);
    private SqliteStore Store() => new(path, clock);

    [Fact]
    public void StateInboxAndOutboxCommitAndSurviveReopen()
    {
        var store = Store();
        store.Execute(Context, tx =>
        {
            Assert.True(tx.TryAccept("projection", "producer", "input"));
            tx.Set("document", "approved");
            tx.Enqueue(Event());
            return 0;
        });
        var restarted = Store();
        restarted.Execute(Context, tx =>
        {
            Assert.Equal("approved", tx.Get("document"));
            Assert.False(tx.TryAccept("projection", "producer", "input"));
            return 0;
        });
        var delivery = Assert.Single(restarted.Claim(Context, 10, TimeSpan.FromSeconds(30)));
        Assert.Equal(1, delivery.Attempts);
        Assert.True(restarted.Complete(Context, delivery));
        Assert.Empty(Store().Claim(Context, 10, TimeSpan.FromSeconds(30)));
    }

    [Fact]
    public void FailureAndCancellationRollbackAllThreeStores()
    {
        var store = Store();
        Assert.Throws<InvalidOperationException>(() => store.Execute<int>(Context, tx =>
        {
            tx.Set("document", "approved");
            Assert.True(tx.TryAccept("consumer", "source", "input"));
            tx.Enqueue(Event());
            throw new InvalidOperationException("business failure");
        }));
        Assert.Null(store.Execute(Context, tx => tx.Get("document")));
        Assert.Empty(store.Claim(Context, 1, TimeSpan.FromSeconds(30)));
        Assert.True(store.Execute(Context, tx => tx.TryAccept("consumer", "source", "input")));
        using var cancellation = new CancellationTokenSource();
        Assert.Throws<OperationCanceledException>(() => store.Execute(Context, tx =>
        {
            tx.Set("document", "cancelled");
            cancellation.Cancel();
            return 0;
        }, cancellation.Token));
        Assert.Null(store.Execute(Context, tx => tx.Get("document")));
    }

    [Fact]
    public void TenantKeysAndConsumerIdentitiesAreIsolated()
    {
        var store = Store();
        foreach (var context in new[] { Context, Other })
            store.Execute(context, tx =>
            {
                tx.Set("document", context.TenantId);
                Assert.True(tx.TryAccept("consumer", "source", "event"));
                Assert.True(tx.TryAccept("other-consumer", "source", "event"));
                Assert.True(tx.TryAccept("consumer", "other-source", "event"));
                tx.Enqueue(Event(context));
                return 0;
            });
        Assert.Equal("tenant-a", store.Execute(Context, tx => tx.Get("document")));
        Assert.Equal("tenant-b", store.Execute(Other, tx => tx.Get("document")));
        var a = Assert.Single(store.Claim(Context, 1, TimeSpan.FromSeconds(30)));
        var b = Assert.Single(store.Claim(Other, 1, TimeSpan.FromSeconds(30)));
        Assert.Throws<ArgumentException>(() => store.Complete(Other, a));
        Assert.Throws<ArgumentException>(() => store.Execute(Context, tx => { tx.Enqueue(Event(Other, "new")); return 0; }));
        Assert.True(store.Complete(Context, a));
        Assert.True(store.Complete(Other, b));
        Assert.True(store.Execute(Context, tx => tx.Delete("document")));
        Assert.False(store.Execute(Context, tx => tx.Delete("document")));
        Assert.Equal("tenant-b", store.Execute(Other, tx => tx.Get("document")));
    }

    [Fact]
    public void ExpiredLeasesRetryAndFenceStaleWorkersAfterRestart()
    {
        var store = Store();
        store.Execute(Context, tx => { tx.Enqueue(Event()); return 0; });
        var first = Assert.Single(store.Claim(Context, 1, TimeSpan.FromSeconds(10)));
        Assert.Empty(store.Claim(Context, 1, TimeSpan.FromSeconds(10)));
        clock.Now += TimeSpan.FromSeconds(10);
        Assert.False(store.Complete(Context, first));
        var restarted = Store();
        var second = Assert.Single(restarted.Claim(Context, 1, TimeSpan.FromSeconds(10)));
        Assert.Equal(2, second.Attempts);
        Assert.NotEqual(first.LeaseToken, second.LeaseToken);
        Assert.False(restarted.Complete(Context, first));
        Assert.False(restarted.Abandon(Context, first, TimeSpan.Zero));
        Assert.True(restarted.Abandon(Context, second, TimeSpan.FromSeconds(5)));
        Assert.Empty(restarted.Claim(Context, 1, TimeSpan.FromSeconds(10)));
        clock.Now += TimeSpan.FromSeconds(5);
        var third = Assert.Single(restarted.Claim(Context, 1, TimeSpan.FromSeconds(10)));
        Assert.Equal(3, third.Attempts);
        Assert.True(restarted.Complete(Context, third));
        Assert.False(restarted.Complete(Context, third));
    }

    [Fact]
    public async Task ConcurrentConsumersApplyOneLocalMutation()
    {
        var stores = Enumerable.Range(0, 12).Select(_ => Store()).ToArray();
        var applied = await Task.WhenAll(stores.Select(store => Task.Run(() =>
            store.Execute(Context, tx =>
            {
                if (!tx.TryAccept("consumer", "source", "event")) return false;
                tx.Set("count", ((int.Parse(tx.Get("count") ?? "0")) + 1).ToString());
                tx.Enqueue(Event());
                return true;
            }))));
        Assert.Single(applied, value => value);
        Assert.Equal("1", Store().Execute(Context, tx => tx.Get("count")));
        Assert.Single(Store().Claim(Context, 10, TimeSpan.FromSeconds(30)));
    }

    [Fact]
    public async Task ConcurrentPublishersCannotClaimSameActiveLease()
    {
        var store = Store();
        store.Execute(Context, tx =>
        {
            for (var i = 0; i < 20; i++) tx.Enqueue(Event(id: i.ToString()));
            return 0;
        });
        var workers = Enumerable.Range(0, 8).Select(_ => Store()).ToArray();
        var results = await Task.WhenAll(workers.Select(worker => Task.Run(() =>
            worker.Claim(Context, 4, TimeSpan.FromMinutes(1)))));
        var deliveries = results.SelectMany(result => result).ToArray();
        Assert.Equal(20, deliveries.Length);
        Assert.Equal(20, deliveries.Select(item => item.Event.Id).Distinct().Count());
    }

    [Fact]
    public async Task AsyncCallbacksCannotCommitBeforeTheirWorkCompletes()
    {
        var invoked = false;
        await Assert.ThrowsAsync<ArgumentException>(() => Store().Execute(Context, tx =>
        {
            invoked = true;
            return Task.CompletedTask;
        }));
        Assert.False(invoked);
    }

    [Fact]
    public void ContentAwareInboxSurvivesReopenAndRejectsChangedPayloadAtomically()
    {
        var store = Store();
        var envelope = Event(data: """{"document_id":"doc","metadata":{"revision":1,"tags":["a","b"]}}""");
        store.Execute(Context, tx =>
        {
            Assert.True(tx.TryAccept("consumer", envelope));
            tx.Set("document", "approved");
            return 0;
        });
        var restarted = Store();
        var reordered = Event(data: """{"metadata":{"tags":["a","b"],"revision":1},"document_id":"doc"}""");
        Assert.False(restarted.Execute(Context, tx => tx.TryAccept("consumer", reordered)));
        Assert.Throws<InboxConflictException>(() => restarted.Execute(Context, tx =>
        {
            tx.Set("document", "must-roll-back");
            tx.Enqueue(Event(id: "outgoing"));
            return tx.TryAccept("consumer", Event(data: """{"document_id":"changed"}"""));
        }));
        Assert.Equal("approved", restarted.Execute(Context, tx => tx.Get("document")));
        Assert.Empty(restarted.Claim(Context, 1, TimeSpan.FromSeconds(30)));
        Assert.False(restarted.Execute(Context, tx => tx.TryAccept("consumer", envelope)));
    }

    [Fact]
    public void ContentAwareInboxIncludesMetadataAndNormalizesTimestampOffsets()
    {
        var store = Store();
        var envelope = Event();
        Assert.True(store.Execute(Context, tx => tx.TryAccept("consumer", envelope)));
        var equivalentOffset = ContractJson.Deserialize<CloudEvent>(
            ContractJson.Serialize(envelope).Replace("2026-09-09T00:00:00+00:00", "2026-09-09T03:00:00+03:00"));
        Assert.Equal(TimeSpan.FromHours(3), equivalentOffset.Time.Offset);
        Assert.False(store.Execute(Context, tx => tx.TryAccept("consumer", equivalentOffset)));
        Assert.Throws<InboxConflictException>(() => store.Execute(Context,
            tx => tx.TryAccept("consumer", Event(new RequestContext(Context.TenantId, "different-workflow", Context.CausationId)))));
        Assert.True(store.Execute(Context, tx => tx.TryAccept("other-consumer", envelope)));
        Assert.True(store.Execute(Other, tx => tx.TryAccept("consumer", Event(Other))));
        Assert.Throws<ArgumentException>(() => store.Execute(Other, tx => tx.TryAccept("consumer", envelope)));
    }

    [Fact]
    public async Task ConcurrentContentAwareConsumersApplyOnce()
    {
        var stores = Enumerable.Range(0, 8).Select(_ => Store()).ToArray();
        var envelope = Event();
        var applied = await Task.WhenAll(stores.Select(store => Task.Run(() => store.Execute(Context, tx =>
        {
            if (!tx.TryAccept("consumer", envelope)) return false;
            tx.Set("document", "approved");
            return true;
        }))));
        Assert.Single(applied, value => value);
        Assert.Equal("approved", Store().Execute(Context, tx => tx.Get("document")));
    }

    [Fact]
    public async Task LegacyInboxMigratesOnConcurrentReopenWithoutInventingContent()
    {
        using (var connection = new SqliteConnection(new SqliteConnectionStringBuilder { DataSource = path, Pooling = false }.ToString()))
        {
            connection.Open();
            using var command = connection.CreateCommand();
            command.CommandText = """
                PRAGMA journal_mode=WAL;
                CREATE TABLE clrinf_inbox (
                    tenant TEXT NOT NULL, consumer TEXT NOT NULL, source TEXT NOT NULL, event_id TEXT NOT NULL,
                    PRIMARY KEY(tenant,consumer,source,event_id));
                INSERT INTO clrinf_inbox VALUES('tenant-a','consumer','clrinf/documents','event');
                """;
            command.ExecuteNonQuery();
        }
        var stores = await Task.WhenAll(Enumerable.Range(0, 6).Select(_ => Task.Run(Store)));
        foreach (var store in stores)
        {
            Assert.False(store.Execute(Context, tx => tx.TryAccept("consumer", "clrinf/documents", "event")));
            Assert.Throws<InboxConflictException>(() => store.Execute(Context, tx => tx.TryAccept("consumer", Event())));
        }
        Assert.True(stores[0].Execute(Context, tx => tx.TryAccept("consumer", Event(id: "new-event"))));
        Assert.False(Store().Execute(Context, tx => tx.TryAccept("consumer", Event(id: "new-event"))));
    }

    [Fact]
    public void DocumentProjectionUsesContentAwareInbox()
    {
        var store = Store();
        var original = CloudEvent.Create(Context, "clrinf/documents", DocumentProjection.EventType,
            JsonSerializer.SerializeToElement(new { document_id = "original" }), "input", clock);
        var changed = CloudEvent.Create(Context, "clrinf/documents", DocumentProjection.EventType,
            JsonSerializer.SerializeToElement(new { document_id = "changed" }), "input", clock);
        Assert.True(DocumentProjection.Apply(store, Context, original));
        Assert.False(DocumentProjection.Apply(Store(), Context, original));
        Assert.Throws<InboxConflictException>(() => DocumentProjection.Apply(store, Context, changed));
        Assert.Equal("approved", store.Execute(Context, tx => tx.Get("projection/original")));
        Assert.Null(store.Execute(Context, tx => tx.Get("projection/changed")));
    }

    [Theory]
    [InlineData("task")]
    [InlineData("value-task")]
    [InlineData("generic-value-task")]
    public void BoxedAsyncResultsRollbackLocalWrites(string kind)
    {
        object asyncResult = kind switch
        {
            "task" => Task.CompletedTask,
            "value-task" => ValueTask.CompletedTask,
            "generic-value-task" => ValueTask.FromResult(1),
            _ => throw new ArgumentOutOfRangeException(nameof(kind))
        };
        var store = Store();
        Assert.Throws<ArgumentException>(() => store.Execute(Context, tx =>
        {
            tx.Set("document", "must-roll-back");
            tx.TryAccept("consumer", Event());
            tx.Enqueue(Event());
            return asyncResult;
        }));
        Assert.Null(store.Execute(Context, tx => tx.Get("document")));
        Assert.Empty(store.Claim(Context, 1, TimeSpan.FromSeconds(30)));
        Assert.True(store.Execute(Context, tx => tx.TryAccept("consumer", Event())));
    }

    [Fact]
    public void DuplicateOutboxIdentityRollsBackBusinessMutation()
    {
        var store = Store();
        store.Execute(Context, tx => { tx.Enqueue(Event()); return 0; });
        Assert.Throws<Microsoft.Data.Sqlite.SqliteException>(() => store.Execute(Context, tx =>
        {
            tx.Set("document", "must-not-commit");
            tx.Enqueue(Event());
            return 0;
        }));
        Assert.Null(store.Execute(Context, tx => tx.Get("document")));
    }

    public void Dispose()
    {
        File.Delete(path);
        File.Delete(path + "-wal");
        File.Delete(path + "-shm");
    }
}
