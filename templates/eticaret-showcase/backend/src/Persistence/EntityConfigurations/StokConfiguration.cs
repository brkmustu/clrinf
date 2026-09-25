using EticaretApp.Domain.Entities;
using Microsoft.EntityFrameworkCore;
using Microsoft.EntityFrameworkCore.Metadata.Builders;

namespace EticaretApp.Persistence.EntityConfigurations;

public class StokConfiguration : IEntityTypeConfiguration<Stok>
{
    public void Configure(EntityTypeBuilder<Stok> builder)
    {
        builder.ToTable("Stoklar").HasKey(s => s.Id);

        builder.Property(s => s.Id).HasColumnName("Id").IsRequired();
        builder.Property(s => s.Sku).HasColumnName("Sku").IsRequired();
        builder.Property(s => s.Miktar).HasColumnName("Miktar").IsRequired();
        builder.Property(s => s.RezerveMiktar).HasColumnName("RezerveMiktar").IsRequired();
        builder.Property(s => s.CreatedDate).HasColumnName("CreatedDate").IsRequired();
        builder.Property(s => s.UpdatedDate).HasColumnName("UpdatedDate");
        builder.Property(s => s.DeletedDate).HasColumnName("DeletedDate");

        builder.HasQueryFilter(s => !s.DeletedDate.HasValue);
    }
}