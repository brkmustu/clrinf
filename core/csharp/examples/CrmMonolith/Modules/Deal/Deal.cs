namespace CrmMonolith.Modules.Deal;

public class Deal
{
    public string Id { get; set; } = Guid.NewGuid().ToString();
    public string TenantId { get; set; } = "default";
    public string Title { get; set; } = default!;
    public string ContactId { get; set; } = default!;
    public decimal Amount { get; set; } = default!;
    public string Stage { get; set; } = default!;
    public int Probability { get; set; } = default!;
}

