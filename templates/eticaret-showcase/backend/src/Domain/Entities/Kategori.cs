using System;
using EticaretApp.Domain.Common.Entities;

namespace EticaretApp.Domain.Entities;

public class Kategori : Entity<int>
{
    public string Ad { get; set; } = string.Empty;
    public string Slug { get; set; } = string.Empty;
    public int? UstKategoriId { get; set; }
    public int Sira { get; set; }
    public bool Aktif { get; set; } = true;

    public Kategori() { }
    public Kategori(int id) : base(id) { }
}
