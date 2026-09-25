using System;

namespace EticaretApp.Application.Common.Attributes;

[AttributeUsage(AttributeTargets.Class, Inherited = false, AllowMultiple = false)]
public sealed class MapEndpointAttribute : Attribute
{
    public string Route { get; }
    public string HttpMethod { get; }

    public MapEndpointAttribute(string route, string httpMethod = "POST")
    {
        Route = route;
        HttpMethod = httpMethod;
    }
}
