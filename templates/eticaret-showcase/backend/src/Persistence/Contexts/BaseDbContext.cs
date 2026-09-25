using EticaretApp.Domain.Entities;
using Microsoft.EntityFrameworkCore;
using System.Reflection;

namespace EticaretApp.Persistence.Contexts;

public class BaseDbContext : DbContext
{
    public DbSet<User> Users { get; set; }
    public DbSet<OperationClaim> OperationClaims { get; set; }
    public DbSet<UserOperationClaim> UserOperationClaims { get; set; }
    public DbSet<RefreshToken> RefreshTokens { get; set; }
    public DbSet<Urun> Urunlar { get; set; }
    public DbSet<Sepet> Sepetler { get; set; }
    public DbSet<Siparis> Siparisler { get; set; }
    public DbSet<Stok> Stoklar { get; set; }
    public DbSet<Kampanya> Kampanyalar { get; set; }
    public DbSet<Kategori> Kategoriler { get; set; }
    public DbSet<Odeme> Odemeler { get; set; }
    public DbSet<Kargo> Kargolar { get; set; }
    public DbSet<Kupon> Kuponlar { get; set; }
    public DbSet<Degerlendirme> Degerlendirmeler { get; set; }

    public BaseDbContext(DbContextOptions<BaseDbContext> options) : base(options)
    {
    }

    protected override void OnModelCreating(ModelBuilder modelBuilder)
    {
        modelBuilder.ApplyConfigurationsFromAssembly(Assembly.GetExecutingAssembly());
        base.OnModelCreating(modelBuilder);
    }
}
