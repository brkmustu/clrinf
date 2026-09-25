using EticaretApp.Application.Services.Security.JWT;
using EticaretApp.Infrastructure.Security.JWT;
using EticaretApp.Application.Services.ImageService;
using EticaretApp.Infrastructure.Adapters.ImageService;
using Microsoft.Extensions.DependencyInjection;

namespace EticaretApp.Infrastructure;

public static class InfrastructureServiceRegistration
{
    public static IServiceCollection AddInfrastructureServices(this IServiceCollection services)
    {
        services.AddScoped<ImageServiceBase, LocalFileImageServiceAdapter>();
                services.AddOptions<TokenOptions>().BindConfiguration("TokenOptions");
        services.AddScoped<ITokenHelper, JwtHelper>();
return services;
    }
}
