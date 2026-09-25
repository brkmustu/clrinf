using EticaretApp.Domain.Common.Entities;

namespace EticaretApp.Domain.Entities;

public class Sepet : Entity<int>
{
    public string KullaniciId { get; set; } = string.Empty;
    public decimal ToplamTutar { get; set; }
    public decimal IndirimTutari { get; set; }
    public bool Kilitli { get; set; }

    public Sepet() { }
    public Sepet(int id) : base(id) { }
}
