using ClrinfCS.Core;

namespace AotCrud.Domain;

[GenerateCrudEndpoints]
public class Brand : ITenantScoped
{
    public int Id { get; set; }

    public string Name { get; set; } = string.Empty;

    [CrudIgnore]
    public string TenantId { get; set; } = "default";
}
