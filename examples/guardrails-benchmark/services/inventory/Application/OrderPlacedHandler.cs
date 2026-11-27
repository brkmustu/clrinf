using Inventory.Domain;

namespace Inventory.Application;

public sealed record OrderPlacedV1(string OrderId, string Warehouse);

public interface IOrderPlacedV1Handler
{
    void Handle(OrderPlacedV1 ev);
}

public sealed class OrderPlacedHandler : IOrderPlacedV1Handler
{
    private readonly StockReservedV1Publisher _publisher = new();

    public void Handle(OrderPlacedV1 ev)
    {
        var item = new StockItem("SKU-DEFAULT", 10);
        if (item.TryReserve(1))
        {
            _publisher.Publish(ev.OrderId, ev.Warehouse);
        }
    }
}

public sealed class StockReservedV1Publisher
{
    public void Publish(string orderId, string warehouse)
    {
        // Delegates to generated CloudEvents outbox publisher
    }
}
