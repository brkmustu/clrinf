using System.Linq;
using System.Reflection;
using Microsoft.Extensions.DependencyInjection;
using EticaretApp.Application.Common.Pipeline;
using EticaretApp.Application.Common.Behaviors;
using EticaretApp.Application.Common.Localization;
using FluentValidation;

namespace EticaretApp.Application;

public static class ApplicationServiceRegistration
{
    public static IServiceCollection AddApplicationServices(this IServiceCollection services)
    {
        var assembly = Assembly.GetExecutingAssembly();

        // Native ICommandHandler & IQueryHandler scanning
        foreach (var type in assembly.GetTypes().Where(t => !t.IsAbstract && !t.IsInterface))
        {
            foreach (var i in type.GetInterfaces().Where(i => i.IsGenericType && (i.GetGenericTypeDefinition() == typeof(ICommandHandler<,>) || i.GetGenericTypeDefinition() == typeof(IQueryHandler<,>))))
            {
                services.AddScoped(i, type);
            }
        }

        // Native IValidator scanning
        foreach (var type in assembly.GetTypes().Where(t => !t.IsAbstract && !t.IsInterface))
        {
            foreach (var i in type.GetInterfaces().Where(i => i.IsGenericType && i.GetGenericTypeDefinition() == typeof(IValidator<>)))
            {
                services.AddScoped(i, type);
            }
        }
        
        // Native BusinessRules scanning
        foreach (var type in assembly.GetTypes().Where(t => !t.IsAbstract && !t.IsInterface && t.Name.EndsWith("BusinessRules")))
        {
            services.AddScoped(type);
        }

        // Custom CQRS Dispatcher
        services.AddScoped<IDispatcher, Dispatcher>();

        // Native Localization Service
        services.AddSingleton<ILocalizationService, LocalizationManager>();

        // HttpContextAccessor for security claims
        services.AddHttpContextAccessor();

        // Pipeline Behaviors
        services.AddScoped(typeof(IPipelineBehavior<,>), typeof(AuthorizationBehavior<,>));
        services.AddScoped(typeof(IPipelineBehavior<,>), typeof(ValidationBehavior<,>));
        services.AddScoped(typeof(IPipelineBehavior<,>), typeof(LoggingBehavior<,>));

        return services;
    }
}
