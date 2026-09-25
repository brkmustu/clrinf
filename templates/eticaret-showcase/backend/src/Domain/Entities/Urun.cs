using EticaretApp.Domain.Common.Entities;

namespace EticaretApp.Domain.Entities;

public class Urun : Entity<int>
{
    public string Sku { get; set; } = string.Empty;
    public string Baslik { get; set; } = string.Empty;
    public string? Aciklama { get; set; }
    public decimal BirimFiyat { get; set; }
    public string ParaBirimi { get; set; } = "USD";
    public string KategoriYolu { get; set; } = string.Empty;
    public int MusaitStok { get; set; }
    public bool Yayinda { get; set; } = true;

    public Urun() { }
    public Urun(int id) : base(id) { }
}
