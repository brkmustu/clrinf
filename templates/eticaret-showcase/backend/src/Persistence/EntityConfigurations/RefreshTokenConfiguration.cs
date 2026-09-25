using Microsoft.EntityFrameworkCore;
using Microsoft.EntityFrameworkCore.Metadata.Builders;
using EticaretApp.Domain.Entities;

namespace EticaretApp.Persistence.EntityConfigurations;

public class RefreshTokenConfiguration : IEntityTypeConfiguration<RefreshToken>
{
    public void Configure(EntityTypeBuilder<RefreshToken> builder)
    {
        builder.ToTable("RefreshTokens").HasKey(rt => rt.Id);
        builder.Property(rt => rt.Id).HasColumnName("Id").IsRequired();
        builder.Property(rt => rt.UserId).HasColumnName("UserId").IsRequired();
        builder.Property(rt => rt.Token).HasColumnName("Token").IsRequired();
        builder.Property(rt => rt.Expires).HasColumnName("Expires").IsRequired();
        builder.Property(rt => rt.Created).HasColumnName("Created").IsRequired();
        builder.Property(rt => rt.CreatedByIp).HasColumnName("CreatedByIp").IsRequired();

        builder.HasOne(rt => rt.User);
    }
}
