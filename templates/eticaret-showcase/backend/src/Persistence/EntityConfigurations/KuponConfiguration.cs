using Microsoft.EntityFrameworkCore;
using Microsoft.EntityFrameworkCore.Metadata.Builders;
using EticaretApp.Domain.Entities;

namespace EticaretApp.Persistence.EntityConfigurations;

public class KuponConfiguration : IEntityTypeConfiguration<Kupon>
{
    public void Configure(EntityTypeBuilder<Kupon> builder)
    {
        builder.ToTable("Kuponlar").HasKey(x => x.Id);
        builder.Property(x => x.Id).HasColumnName("Id").IsRequired();
    }
}
