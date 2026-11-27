namespace Inventory.Domain;

public sealed record WarehousePolicy(string Code, int MaxBatchSize)
{
    public bool Allows(int requested) => requested > 0 && requested <= MaxBatchSize;
}
