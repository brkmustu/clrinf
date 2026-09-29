using System.Globalization;
using System.Text.Json.Nodes;
using ClrinfCS.Core;
using Microsoft.Data.Sqlite;

namespace ClrinfCS.Adapters.Sqlite;

/// <summary>Durable, single-file reference store. Operations use short synchronous SQLite transactions.</summary>
public sealed class SqliteStore : ITransactionalStore, IOutbox
{
    private readonly string connectionString;
    private readonly TimeProvider clock;

    public SqliteStore(string databasePath, TimeProvider? clock = null)
    {
        ArgumentException.ThrowIfNullOrWhiteSpace(databasePath);
        if (databasePath == ":memory:") throw new ArgumentException("A persistent file path is required.", nameof(databasePath));
        this.clock = clock ?? TimeProvider.System;
        connectionString = new SqliteConnectionStringBuilder
        {
            DataSource = Path.GetFullPath(databasePath), Mode = SqliteOpenMode.ReadWriteCreate,
            Pooling = false, DefaultTimeout = 30
        }.ToString();
        using var connection = Open();
        using var command = connection.CreateCommand();
        command.CommandText = """
            PRAGMA journal_mode=WAL;
            CREATE TABLE IF NOT EXISTS clrinf_state (
                tenant TEXT NOT NULL, key TEXT NOT NULL, value TEXT NOT NULL,
                PRIMARY KEY(tenant, key));
            CREATE TABLE IF NOT EXISTS clrinf_inbox (
                tenant TEXT NOT NULL, consumer TEXT NOT NULL, source TEXT NOT NULL, event_id TEXT NOT NULL,
                content TEXT,
                PRIMARY KEY(tenant, consumer, source, event_id));
            CREATE TABLE IF NOT EXISTS clrinf_outbox (
                sequence INTEGER PRIMARY KEY AUTOINCREMENT,
                tenant TEXT NOT NULL, source TEXT NOT NULL, event_id TEXT NOT NULL, payload TEXT NOT NULL,
                available_at INTEGER NOT NULL, lease_until INTEGER, lease_token TEXT,
                attempts INTEGER NOT NULL DEFAULT 0, completed INTEGER NOT NULL DEFAULT 0,
                UNIQUE(tenant, source, event_id));
            CREATE INDEX IF NOT EXISTS clrinf_outbox_pending
                ON clrinf_outbox(tenant, completed, available_at, lease_until);
            """;
        command.ExecuteNonQuery();
        using var migration = connection.BeginTransaction(deferred: false);
        var hasContent = false;
        using (var columns = Command(connection, migration, "PRAGMA table_info(clrinf_inbox)"))
        using (var reader = columns.ExecuteReader())
            while (reader.Read())
                hasContent |= reader.GetString(1) == "content";
        if (!hasContent)
        {
            using var addColumn = Command(connection, migration, "ALTER TABLE clrinf_inbox ADD COLUMN content TEXT");
            addColumn.ExecuteNonQuery();
        }
        migration.Commit();
    }

    public T Execute<T>(RequestContext context, Func<IStorageTransaction, T> operation,
        CancellationToken cancellationToken = default)
    {
        ArgumentNullException.ThrowIfNull(context);
        ArgumentNullException.ThrowIfNull(operation);
        if (IsAsyncResult(typeof(T)))
            throw new ArgumentException("SQLite transaction callbacks must be synchronous.", nameof(operation));
        cancellationToken.ThrowIfCancellationRequested();
        using var connection = Open();
        using var transaction = connection.BeginTransaction(deferred: false);
        var session = new Session(context, connection, transaction, clock);
        var result = operation(session);
        if (result is not null && IsAsyncResult(result.GetType()))
            throw new ArgumentException("SQLite transaction callbacks must not return asynchronous work, including boxed results.", nameof(operation));
        cancellationToken.ThrowIfCancellationRequested();
        transaction.Commit();
        return result;
    }

    private static bool IsAsyncResult(Type type) =>
        typeof(Task).IsAssignableFrom(type) || type == typeof(ValueTask) ||
        (type.IsGenericType && type.GetGenericTypeDefinition() == typeof(ValueTask<>));

    public IReadOnlyList<OutboxDelivery> Claim(RequestContext context, int limit, TimeSpan lease)
    {
        ArgumentNullException.ThrowIfNull(context);
        if (limit <= 0) throw new ArgumentOutOfRangeException(nameof(limit));
        if (lease < TimeSpan.FromMilliseconds(1)) throw new ArgumentOutOfRangeException(nameof(lease));
        using var connection = Open();
        using var transaction = connection.BeginTransaction(deferred: false);
        var now = clock.GetUtcNow().ToUnixTimeMilliseconds();
        var until = checked(now + (long)lease.TotalMilliseconds);
        var pending = new List<(long Sequence, CloudEvent Event, int Attempts)>();
        using (var select = Command(connection, transaction, """
            SELECT sequence, payload, attempts FROM clrinf_outbox
            WHERE tenant=$tenant AND completed=0 AND available_at<=$now
                AND (lease_until IS NULL OR lease_until<=$now)
            ORDER BY sequence LIMIT $limit
            """, ("$tenant", context.TenantId), ("$now", now), ("$limit", limit)))
        using (var reader = select.ExecuteReader())
            while (reader.Read())
                pending.Add((reader.GetInt64(0), ContractJson.Deserialize<CloudEvent>(reader.GetString(1)), reader.GetInt32(2) + 1));

        var result = new List<OutboxDelivery>();
        foreach (var item in pending)
        {
            var token = Guid.NewGuid().ToString();
            using var update = Command(connection, transaction, """
                UPDATE clrinf_outbox SET lease_until=$until, lease_token=$token, attempts=attempts+1
                WHERE sequence=$sequence
                """, ("$until", until), ("$token", token), ("$sequence", item.Sequence));
            update.ExecuteNonQuery();
            result.Add(new(item.Event, token, item.Attempts));
        }
        transaction.Commit();
        return result;
    }

    public bool Complete(RequestContext context, OutboxDelivery delivery) => UpdateDelivery(context, delivery, null);

    public bool Abandon(RequestContext context, OutboxDelivery delivery, TimeSpan retryDelay)
    {
        if (retryDelay < TimeSpan.Zero) throw new ArgumentOutOfRangeException(nameof(retryDelay));
        return UpdateDelivery(context, delivery, retryDelay);
    }

    private bool UpdateDelivery(RequestContext context, OutboxDelivery delivery, TimeSpan? retryDelay)
    {
        ArgumentNullException.ThrowIfNull(context);
        ArgumentNullException.ThrowIfNull(delivery);
        if (delivery.Event.TenantId != context.TenantId)
            throw new ArgumentException("Delivery tenant does not match the trusted context.", nameof(delivery));
        using var connection = Open();
        var now = clock.GetUtcNow().ToUnixTimeMilliseconds();
        using var command = Command(connection, null, """
            UPDATE clrinf_outbox SET completed=$completed, available_at=$available,
                lease_until=NULL, lease_token=NULL
            WHERE tenant=$tenant AND source=$source AND event_id=$id AND completed=0
                AND lease_token=$token AND lease_until>$now
            """, ("$completed", retryDelay.HasValue ? 0 : 1),
            ("$available", checked(now + (long)(retryDelay?.TotalMilliseconds ?? 0))),
            ("$tenant", context.TenantId), ("$source", delivery.Event.Source),
            ("$id", delivery.Event.Id), ("$token", delivery.LeaseToken), ("$now", now));
        return command.ExecuteNonQuery() == 1;
    }

    private SqliteConnection Open()
    {
        var connection = new SqliteConnection(connectionString);
        connection.Open();
        return connection;
    }

    private static SqliteCommand Command(SqliteConnection connection, SqliteTransaction? transaction,
        string sql, params (string Name, object Value)[] parameters)
    {
        var command = connection.CreateCommand();
        command.Transaction = transaction;
        command.CommandText = sql;
        foreach (var (name, value) in parameters) command.Parameters.AddWithValue(name, value);
        return command;
    }

    private sealed class Session(RequestContext context, SqliteConnection connection,
        SqliteTransaction transaction, TimeProvider clock) : IStorageTransaction
    {
        public RequestContext Context { get; } = context;

        public string? Get(string key)
        {
            ArgumentException.ThrowIfNullOrWhiteSpace(key);
            using var command = Command(connection, transaction,
                "SELECT value FROM clrinf_state WHERE tenant=$tenant AND key=$key",
                ("$tenant", Context.TenantId), ("$key", key));
            return command.ExecuteScalar() as string;
        }

        public void Set(string key, string value)
        {
            ArgumentException.ThrowIfNullOrWhiteSpace(key);
            ArgumentNullException.ThrowIfNull(value);
            using var command = Command(connection, transaction, """
                INSERT INTO clrinf_state(tenant,key,value) VALUES($tenant,$key,$value)
                ON CONFLICT(tenant,key) DO UPDATE SET value=excluded.value
                """, ("$tenant", Context.TenantId), ("$key", key), ("$value", value));
            command.ExecuteNonQuery();
        }

        public bool Delete(string key)
        {
            ArgumentException.ThrowIfNullOrWhiteSpace(key);
            using var command = Command(connection, transaction,
                "DELETE FROM clrinf_state WHERE tenant=$tenant AND key=$key",
                ("$tenant", Context.TenantId), ("$key", key));
            return command.ExecuteNonQuery() == 1;
        }

        public bool TryAccept(string consumer, string source, string eventId)
        {
            ArgumentException.ThrowIfNullOrWhiteSpace(consumer);
            ArgumentException.ThrowIfNullOrWhiteSpace(source);
            ArgumentException.ThrowIfNullOrWhiteSpace(eventId);
            using var command = Command(connection, transaction, """
                INSERT INTO clrinf_inbox(tenant,consumer,source,event_id)
                VALUES($tenant,$consumer,$source,$id) ON CONFLICT DO NOTHING
                """, ("$tenant", Context.TenantId), ("$consumer", consumer), ("$source", source), ("$id", eventId));
            return command.ExecuteNonQuery() == 1;
        }

        public bool TryAccept(string consumer, CloudEvent envelope)
        {
            ArgumentException.ThrowIfNullOrWhiteSpace(consumer);
            ArgumentNullException.ThrowIfNull(envelope);
            if (envelope.TenantId != Context.TenantId)
                throw new ArgumentException("Event tenant does not match the trusted context.", nameof(envelope));
            var content = JsonNode.Parse(ContractJson.Serialize(envelope))!;
            content["time"] = envelope.Time.ToUniversalTime().ToString("O", CultureInfo.InvariantCulture);
            using var insert = Command(connection, transaction, """
                INSERT INTO clrinf_inbox(tenant,consumer,source,event_id,content)
                VALUES($tenant,$consumer,$source,$id,$content) ON CONFLICT DO NOTHING
                """, ("$tenant", Context.TenantId), ("$consumer", consumer), ("$source", envelope.Source),
                ("$id", envelope.Id), ("$content", content.ToJsonString()));
            if (insert.ExecuteNonQuery() == 1) return true;

            using var select = Command(connection, transaction, """
                SELECT content FROM clrinf_inbox
                WHERE tenant=$tenant AND consumer=$consumer AND source=$source AND event_id=$id
                """, ("$tenant", Context.TenantId), ("$consumer", consumer),
                ("$source", envelope.Source), ("$id", envelope.Id));
            var stored = select.ExecuteScalar();
            if (stored is not string previous)
                throw new InboxConflictException("The existing inbox identity has no recorded content; content-aware replay cannot be verified.");
            if (!JsonNode.DeepEquals(JsonNode.Parse(previous), content))
                throw new InboxConflictException("The inbox identity was already used with different event content.");
            return false;
        }

        public void Enqueue(CloudEvent envelope)
        {
            ArgumentNullException.ThrowIfNull(envelope);
            if (envelope.TenantId != Context.TenantId)
                throw new ArgumentException("Event tenant does not match the trusted context.", nameof(envelope));
            using var command = Command(connection, transaction, """
                INSERT INTO clrinf_outbox(tenant,source,event_id,payload,available_at)
                VALUES($tenant,$source,$id,$payload,$now)
                """, ("$tenant", Context.TenantId), ("$source", envelope.Source), ("$id", envelope.Id),
                ("$payload", ContractJson.Serialize(envelope)), ("$now", clock.GetUtcNow().ToUnixTimeMilliseconds()));
            command.ExecuteNonQuery();
        }
    }
}
