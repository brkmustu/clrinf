namespace CrmMonolith.Modules.Contact;

using Microsoft.Extensions.DependencyInjection;

public static class ContactModule
{
    public static IServiceCollection AddContactModule(this IServiceCollection services)
    {
        // Register module handlers, validators, and rules here
        return services;
    }
}
