namespace CrmMonolith.Modules.Activity;

public class Activity
{
    public string Id { get; set; } = Guid.NewGuid().ToString();
    public string TenantId { get; set; } = "default";
    public string Title { get; set; } = default!;
    public string Type { get; set; } = default!;
    public DateTime DueDate { get; set; } = default!;
    public string? Notes { get; set; }
    public string? DealId { get; set; }
    public string? ContactId { get; set; }
}
