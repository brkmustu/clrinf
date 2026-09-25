using System;

namespace EticaretApp.Application.Common.Attributes;

[AttributeUsage(AttributeTargets.Class, Inherited = false, AllowMultiple = false)]
public sealed class RequireAuthorizationAttribute : Attribute
{
    public string[] Roles { get; }

    public RequireAuthorizationAttribute(params string[] roles)
    {
        Roles = roles ?? Array.Empty<string>();
    }
}
