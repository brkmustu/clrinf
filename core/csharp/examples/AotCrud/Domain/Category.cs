using ClrinfCS.Core;

namespace AotCrud.Domain;

/// <summary>
/// Standard parameter table. Handlers and Minimal API endpoints are generated
/// at compile time; only this entity lives on disk.
/// </summary>
[GenerateCrudEndpoints]
public class Category : ITenantScoped
{
    public int Id { get; set; }

    public string Name { get; set; } = string.Empty;

    public string Slug { get; set; } = string.Empty;

    /// <summary>Tenant comes from RequestContext, never from the client DTO.</summary>
    [CrudIgnore]
    public string TenantId { get; set; } = "default";
}
