namespace Inventory.Infrastructure;

public sealed class SqliteInventoryRepository
{
    public string ConnectionString { get; }

    public SqliteInventoryRepository(string connectionString)
    {
        ConnectionString = connectionString;
    }
}
