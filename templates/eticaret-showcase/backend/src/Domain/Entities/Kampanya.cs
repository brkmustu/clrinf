using EticaretApp.Domain.Common.Entities;

namespace EticaretApp.Domain.Entities;

public class Kampanya : Entity<int>
{
    public string Kod { get; set; } = string.Empty;
    public string Baslik { get; set; } = string.Empty;
    public decimal Deger { get; set; }
    public bool Aktif { get; set; } = true;

    public Kampanya() { }
    public Kampanya(int id) : base(id) { }
}
