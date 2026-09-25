using EticaretApp.Domain.Entities;
using Microsoft.EntityFrameworkCore;
using Microsoft.EntityFrameworkCore.Metadata.Builders;

namespace EticaretApp.Persistence.EntityConfigurations;

public class SepetConfiguration : IEntityTypeConfiguration<Sepet>
{
    public void Configure(EntityTypeBuilder<Sepet> builder)
    {
        builder.ToTable("Sepetler").HasKey(s => s.Id);

        builder.Property(s => s.Id).HasColumnName("Id").IsRequired();
        builder.Property(s => s.KullaniciId).HasColumnName("KullaniciId").IsRequired();
        builder.Property(s => s.ToplamTutar).HasColumnName("ToplamTutar").IsRequired();
        builder.Property(s => s.IndirimTutari).HasColumnName("IndirimTutari").IsRequired();
        builder.Property(s => s.Kilitli).HasColumnName("Kilitli").IsRequired();
        builder.Property(s => s.CreatedDate).HasColumnName("CreatedDate").IsRequired();
        builder.Property(s => s.UpdatedDate).HasColumnName("UpdatedDate");
        builder.Property(s => s.DeletedDate).HasColumnName("DeletedDate");

        builder.HasQueryFilter(s => !s.DeletedDate.HasValue);
    }
}