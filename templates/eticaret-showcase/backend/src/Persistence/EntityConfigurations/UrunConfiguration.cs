using EticaretApp.Domain.Entities;
using Microsoft.EntityFrameworkCore;
using Microsoft.EntityFrameworkCore.Metadata.Builders;

namespace EticaretApp.Persistence.EntityConfigurations;

public class UrunConfiguration : IEntityTypeConfiguration<Urun>
{
    public void Configure(EntityTypeBuilder<Urun> builder)
    {
        builder.ToTable("Urunlar").HasKey(u => u.Id);

        builder.Property(u => u.Id).HasColumnName("Id").IsRequired();
        builder.Property(u => u.Sku).HasColumnName("Sku").IsRequired();
        builder.Property(u => u.Baslik).HasColumnName("Baslik").IsRequired();
        builder.Property(u => u.Aciklama).HasColumnName("Aciklama");
        builder.Property(u => u.BirimFiyat).HasColumnName("BirimFiyat").IsRequired();
        builder.Property(u => u.ParaBirimi).HasColumnName("ParaBirimi").IsRequired();
        builder.Property(u => u.KategoriYolu).HasColumnName("KategoriYolu").IsRequired();
        builder.Property(u => u.MusaitStok).HasColumnName("MusaitStok").IsRequired();
        builder.Property(u => u.Yayinda).HasColumnName("Yayinda").IsRequired();
        builder.Property(u => u.CreatedDate).HasColumnName("CreatedDate").IsRequired();
        builder.Property(u => u.UpdatedDate).HasColumnName("UpdatedDate");
        builder.Property(u => u.DeletedDate).HasColumnName("DeletedDate");

        builder.HasQueryFilter(u => !u.DeletedDate.HasValue);
    }
}