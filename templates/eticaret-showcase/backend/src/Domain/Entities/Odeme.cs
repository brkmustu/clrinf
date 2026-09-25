using EticaretApp.Domain.Common.Entities;

namespace EticaretApp.Domain.Entities;

public class Odeme : Entity<int>
{
    public int SiparisId { get; set; }
    public string OdemeNumarasi { get; set; } = string.Empty;
    public decimal Tutar { get; set; }
    public string ParaBirimi { get; set; } = "TRY";
    public string OdemeYontemi { get; set; } = "KrediKarti";
    public string Durum { get; set; } = "Basarili";
    public string? IslemKodu { get; set; }

    public Odeme() { }
    public Odeme(int id) : base(id) { }
}
