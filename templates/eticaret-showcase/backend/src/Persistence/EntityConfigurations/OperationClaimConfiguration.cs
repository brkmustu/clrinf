using System.Collections.Generic;
using Microsoft.EntityFrameworkCore;
using Microsoft.EntityFrameworkCore.Metadata.Builders;
using EticaretApp.Domain.Entities;
using EticaretApp.Application.Moduller.Urunlar;
using EticaretApp.Application.Moduller.Sepetler;
using EticaretApp.Application.Moduller.Siparisler;
using EticaretApp.Application.Moduller.Stoklar;
using EticaretApp.Application.Moduller.Kampanyalar;

namespace EticaretApp.Persistence.EntityConfigurations;

public class OperationClaimConfiguration : IEntityTypeConfiguration<OperationClaim>
{
    public void Configure(EntityTypeBuilder<OperationClaim> builder)
    {
        builder.ToTable("OperationClaims").HasKey(oc => oc.Id);
        builder.Property(oc => oc.Id).HasColumnName("Id").IsRequired();
        builder.Property(oc => oc.Name).HasColumnName("Name").IsRequired();
        builder.HasIndex(oc => oc.Name, "UK_OperationClaims_Name").IsUnique();

        builder.HasData(getFeatureOperationClaims());
    }

    private IEnumerable<OperationClaim> getFeatureOperationClaims()
    {
        int lastId = 0;
        List<OperationClaim> featureOperationClaims = new()
        {
            new() { Id = ++lastId, Name = "Admin" },
            new() { Id = ++lastId, Name = "User" }
        };
        
        #region Urunlar CRUD
        featureOperationClaims.AddRange(
            [
                new() { Id = ++lastId, Name = UrunlarOperationClaims.Admin },
                new() { Id = ++lastId, Name = UrunlarOperationClaims.Read },
                new() { Id = ++lastId, Name = UrunlarOperationClaims.Write },
                new() { Id = ++lastId, Name = UrunlarOperationClaims.Create },
                new() { Id = ++lastId, Name = UrunlarOperationClaims.Update },
                new() { Id = ++lastId, Name = UrunlarOperationClaims.Delete },
            ]
        );
        #endregion
        
        #region Sepetler CRUD
        featureOperationClaims.AddRange(
            [
                new() { Id = ++lastId, Name = SepetlerOperationClaims.Admin },
                new() { Id = ++lastId, Name = SepetlerOperationClaims.Read },
                new() { Id = ++lastId, Name = SepetlerOperationClaims.Write },
                new() { Id = ++lastId, Name = SepetlerOperationClaims.Create },
                new() { Id = ++lastId, Name = SepetlerOperationClaims.Update },
                new() { Id = ++lastId, Name = SepetlerOperationClaims.Delete },
            ]
        );
        #endregion
        
        #region Siparisler CRUD
        featureOperationClaims.AddRange(
            [
                new() { Id = ++lastId, Name = SiparislerOperationClaims.Admin },
                new() { Id = ++lastId, Name = SiparislerOperationClaims.Read },
                new() { Id = ++lastId, Name = SiparislerOperationClaims.Write },
                new() { Id = ++lastId, Name = SiparislerOperationClaims.Create },
                new() { Id = ++lastId, Name = SiparislerOperationClaims.Update },
                new() { Id = ++lastId, Name = SiparislerOperationClaims.Delete },
            ]
        );
        #endregion
        
        #region Stoklar CRUD
        featureOperationClaims.AddRange(
            [
                new() { Id = ++lastId, Name = StoklarOperationClaims.Admin },
                new() { Id = ++lastId, Name = StoklarOperationClaims.Read },
                new() { Id = ++lastId, Name = StoklarOperationClaims.Write },
                new() { Id = ++lastId, Name = StoklarOperationClaims.Create },
                new() { Id = ++lastId, Name = StoklarOperationClaims.Update },
                new() { Id = ++lastId, Name = StoklarOperationClaims.Delete },
            ]
        );
        #endregion
        
        #region Kampanyalar CRUD
        featureOperationClaims.AddRange(
            [
                new() { Id = ++lastId, Name = KampanyalarOperationClaims.Admin },
                new() { Id = ++lastId, Name = KampanyalarOperationClaims.Read },
                new() { Id = ++lastId, Name = KampanyalarOperationClaims.Write },
                new() { Id = ++lastId, Name = KampanyalarOperationClaims.Create },
                new() { Id = ++lastId, Name = KampanyalarOperationClaims.Update },
                new() { Id = ++lastId, Name = KampanyalarOperationClaims.Delete },
            ]
        );
        #endregion

        var distinctClaims = new List<OperationClaim>();
        var seenNames = new HashSet<string>();
        int id = 1;

        foreach (var claim in featureOperationClaims)
        {
            if (seenNames.Add(claim.Name))
            {
                distinctClaims.Add(new OperationClaim { Id = id++, Name = claim.Name });
            }
        }

        return distinctClaims;
    }
}
