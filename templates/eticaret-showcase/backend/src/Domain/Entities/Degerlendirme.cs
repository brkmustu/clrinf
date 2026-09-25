using EticaretApp.Domain.Common.Entities;

namespace EticaretApp.Domain.Entities;

public class Degerlendirme : Entity<int>
{
    public int UrunId { get; set; }
    public string KullaniciId { get; set; } = string.Empty;
    public int Puan { get; set; } = 5;
    public string? Yorum { get; set; }
    public bool Onaylandi { get; set; } = true;

    public Degerlendirme() { }
    public Degerlendirme(int id) : base(id) { }
}
