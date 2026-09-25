using Microsoft.EntityFrameworkCore;
using Microsoft.EntityFrameworkCore.Metadata.Builders;
using EticaretApp.Domain.Entities;

namespace EticaretApp.Persistence.EntityConfigurations;

public class OdemeConfiguration : IEntityTypeConfiguration<Odeme>
{
    public void Configure(EntityTypeBuilder<Odeme> builder)
    {
        builder.ToTable("Odemeler").HasKey(x => x.Id);
        builder.Property(x => x.Id).HasColumnName("Id").IsRequired();
    }
}
