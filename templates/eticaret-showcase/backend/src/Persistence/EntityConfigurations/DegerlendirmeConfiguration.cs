using Microsoft.EntityFrameworkCore;
using Microsoft.EntityFrameworkCore.Metadata.Builders;
using EticaretApp.Domain.Entities;

namespace EticaretApp.Persistence.EntityConfigurations;

public class DegerlendirmeConfiguration : IEntityTypeConfiguration<Degerlendirme>
{
    public void Configure(EntityTypeBuilder<Degerlendirme> builder)
    {
        builder.ToTable("Degerlendirmeler").HasKey(x => x.Id);
        builder.Property(x => x.Id).HasColumnName("Id").IsRequired();
    }
}
