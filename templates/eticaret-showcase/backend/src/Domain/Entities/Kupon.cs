using System;
using EticaretApp.Domain.Common.Entities;

namespace EticaretApp.Domain.Entities;

public class Kupon : Entity<int>
{
    public string Kod { get; set; } = string.Empty;
    public decimal IndirimTutari { get; set; }
    public decimal MinimumSepetTutari { get; set; }
    public int KullanimLimiti { get; set; }
    public int KullanilanAdet { get; set; }
    public DateTime SonKullanmaTarihi { get; set; }
    public bool Aktif { get; set; } = true;

    public Kupon() { }
    public Kupon(int id) : base(id) { }
}
