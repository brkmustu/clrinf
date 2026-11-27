namespace Inventory.Domain;

public sealed class StockItem
{
    public string Sku { get; }
    public int Available { get; private set; }

    public StockItem(string sku, int available)
    {
        Sku = sku;
        Available = available;
    }

    public bool TryReserve(int quantity)
    {
        if (quantity <= 0 || quantity > Available)
            return false;
        Available -= quantity;
        return true;
    }
}
