using EticaretApp.Domain.Entities;
using Microsoft.EntityFrameworkCore;
using Microsoft.EntityFrameworkCore.Metadata.Builders;

namespace EticaretApp.Persistence.EntityConfigurations;

public class KampanyaConfiguration : IEntityTypeConfiguration<Kampanya>
{
    public void Configure(EntityTypeBuilder<Kampanya> builder)
    {
        builder.ToTable("Kampanyalar").HasKey(k => k.Id);

        builder.Property(k => k.Id).HasColumnName("Id").IsRequired();
        builder.Property(k => k.Kod).HasColumnName("Kod").IsRequired();
        builder.Property(k => k.Baslik).HasColumnName("Baslik").IsRequired();
        builder.Property(k => k.Deger).HasColumnName("Deger").IsRequired();
        builder.Property(k => k.Aktif).HasColumnName("Aktif").IsRequired();
        builder.Property(k => k.CreatedDate).HasColumnName("CreatedDate").IsRequired();
        builder.Property(k => k.UpdatedDate).HasColumnName("UpdatedDate");
        builder.Property(k => k.DeletedDate).HasColumnName("DeletedDate");

        builder.HasQueryFilter(k => !k.DeletedDate.HasValue);
    }
}