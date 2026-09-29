namespace CrmMonolith.Domain.Entities;

public class UserOperationClaim
{
    public string Id { get; set; } = Guid.NewGuid().ToString();
    public string UserId { get; set; } = string.Empty;
    public string OperationClaimId { get; set; } = string.Empty;
    public virtual User? User { get; set; }
    public virtual OperationClaim? OperationClaim { get; set; }
}
