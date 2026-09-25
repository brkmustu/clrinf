using EticaretApp.Domain.Common.Entities;

namespace EticaretApp.Domain.Entities;

public class Siparis : Entity<int>
{
    public string SiparisNo { get; set; } = string.Empty;
    public string KullaniciId { get; set; } = string.Empty;
    public decimal ToplamTutar { get; set; }
    public string Durum { get; set; } = "Onaylandi";
    public string? KargoTakipNo { get; set; }

    public Siparis() { }
    public Siparis(int id) : base(id) { }
}
