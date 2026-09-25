using EticaretApp.Domain.Common.Entities;

namespace EticaretApp.Domain.Entities;

public class RefreshToken : Entity<int>
{
    public int UserId { get; set; }
    public string Token { get; set; } = string.Empty;
    public DateTime Expires { get; set; }
    public DateTime Created { get; set; }
    public string CreatedByIp { get; set; } = string.Empty;
    public DateTime? Revoked { get; set; }
    public string? RevokedByIp { get; set; }
    public string? ReplacedByToken { get; set; }
    public string? ReasonRevoked { get; set; }

    public virtual User User { get; set; } = null!;

    public RefreshToken()
    {
    }

    public RefreshToken(int id, int userId, string token, DateTime expires, DateTime created, string createdByIp)
        : base(id)
    {
        UserId = userId;
        Token = token;
        Expires = expires;
        Created = created;
        CreatedByIp = createdByIp;
    }
}
