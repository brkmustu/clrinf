using EticaretApp.Domain.Common.Entities;

namespace EticaretApp.Domain.Entities;

public class Stok : Entity<int>
{
    public string Sku { get; set; } = string.Empty;
    public int Miktar { get; set; }
    public int RezerveMiktar { get; set; }

    public Stok() { }
    public Stok(int id) : base(id) { }
}
