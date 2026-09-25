using System;
using Microsoft.EntityFrameworkCore;
using Microsoft.Extensions.Configuration;
using Microsoft.Extensions.DependencyInjection;
using EticaretApp.Persistence.Contexts;
using EticaretApp.Application.Services.Repositories;
using EticaretApp.Persistence.Repositories;

namespace EticaretApp.Persistence;

public static class PersistenceServiceRegistration
{
    public static IServiceCollection AddPersistenceServices(this IServiceCollection services, IConfiguration configuration)
    {
        var connectionString = configuration.GetConnectionString("DefaultConnection");
        if (!string.IsNullOrEmpty(connectionString) && !connectionString.Contains("InMemory", StringComparison.OrdinalIgnoreCase))
        {
            services.AddDbContext<BaseDbContext>(options => 
                options.UseNpgsql(connectionString)
                       .ConfigureWarnings(w => w.Ignore(Microsoft.EntityFrameworkCore.Diagnostics.RelationalEventId.PendingModelChangesWarning)));
        }
        else
        {
            services.AddDbContext<BaseDbContext>(options => options.UseInMemoryDatabase("EticaretAppDb"));
        }
        
        services.AddScoped(typeof(EticaretApp.Persistence.Common.Functional.QueryExecutor<>));
        services.AddScoped(typeof(EticaretApp.Persistence.Common.Functional.QueryHandler<,>));

        services.AddScoped<IUserRepository, UserRepository>();
        services.AddScoped<IRefreshTokenRepository, RefreshTokenRepository>();
        services.AddScoped<IUserOperationClaimRepository, UserOperationClaimRepository>();
        services.AddScoped<IOperationClaimRepository, OperationClaimRepository>();
        services.AddScoped<IUrunRepository, UrunRepository>();
        services.AddScoped<ISepetRepository, SepetRepository>();
        services.AddScoped<ISiparisRepository, SiparisRepository>();
        services.AddScoped<IStokRepository, StokRepository>();
        services.AddScoped<IKampanyaRepository, KampanyaRepository>();
        services.AddScoped<IKategoriRepository, KategoriRepository>();
        services.AddScoped<IOdemeRepository, OdemeRepository>();
        services.AddScoped<IKargoRepository, KargoRepository>();
        services.AddScoped<IKuponRepository, KuponRepository>();
        services.AddScoped<IDegerlendirmeRepository, DegerlendirmeRepository>();

        return services;
    }
}
