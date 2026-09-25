using System;
using EticaretApp.Domain.Common.Entities;

namespace EticaretApp.Domain.Entities;

public class Kargo : Entity<int>
{
    public int SiparisId { get; set; }
    public string TakipNumarasi { get; set; } = string.Empty;
    public string KargoFirmasi { get; set; } = string.Empty;
    public string Durum { get; set; } = "Hazirlaniyor";
    public DateTime? TahminiTeslimTarihi { get; set; }

    public Kargo() { }
    public Kargo(int id) : base(id) { }
}
