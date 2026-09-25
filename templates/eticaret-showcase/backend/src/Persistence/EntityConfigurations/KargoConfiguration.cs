using Microsoft.EntityFrameworkCore;
using Microsoft.EntityFrameworkCore.Metadata.Builders;
using EticaretApp.Domain.Entities;

namespace EticaretApp.Persistence.EntityConfigurations;

public class KargoConfiguration : IEntityTypeConfiguration<Kargo>
{
    public void Configure(EntityTypeBuilder<Kargo> builder)
    {
        builder.ToTable("Kargolar").HasKey(x => x.Id);
        builder.Property(x => x.Id).HasColumnName("Id").IsRequired();
    }
}
