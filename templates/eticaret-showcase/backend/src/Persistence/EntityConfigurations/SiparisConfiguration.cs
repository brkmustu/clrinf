using EticaretApp.Domain.Entities;
using Microsoft.EntityFrameworkCore;
using Microsoft.EntityFrameworkCore.Metadata.Builders;

namespace EticaretApp.Persistence.EntityConfigurations;

public class SiparisConfiguration : IEntityTypeConfiguration<Siparis>
{
    public void Configure(EntityTypeBuilder<Siparis> builder)
    {
        builder.ToTable("Siparisler").HasKey(s => s.Id);

        builder.Property(s => s.Id).HasColumnName("Id").IsRequired();
        builder.Property(s => s.SiparisNo).HasColumnName("SiparisNo").IsRequired();
        builder.Property(s => s.KullaniciId).HasColumnName("KullaniciId").IsRequired();
        builder.Property(s => s.ToplamTutar).HasColumnName("ToplamTutar").IsRequired();
        builder.Property(s => s.Durum).HasColumnName("Durum").IsRequired();
        builder.Property(s => s.KargoTakipNo).HasColumnName("KargoTakipNo");
        builder.Property(s => s.CreatedDate).HasColumnName("CreatedDate").IsRequired();
        builder.Property(s => s.UpdatedDate).HasColumnName("UpdatedDate");
        builder.Property(s => s.DeletedDate).HasColumnName("DeletedDate");

        builder.HasQueryFilter(s => !s.DeletedDate.HasValue);
    }
}