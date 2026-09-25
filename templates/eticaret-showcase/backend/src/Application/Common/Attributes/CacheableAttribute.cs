using System;

namespace EticaretApp.Application.Common.Attributes;

[AttributeUsage(AttributeTargets.Class, Inherited = false, AllowMultiple = false)]
public sealed class CacheableAttribute : Attribute
{
    public string? CacheKey { get; set; }
    public string[]? CacheGroupKeys { get; set; }
    public int SlidingExpirationSeconds { get; set; } = 300;
}
