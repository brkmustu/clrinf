namespace EticaretApp.Application.Features.Kategoriler;

/// <summary>
/// Cedar ABAC / RBAC Uyumlu Yetki Talepleri (Operation Claims)
/// </summary>
public static class KategorilerOperationClaims
{
    private const string _resource = "Kategori";

    // Standart Roller
    public const string Admin = "Admin";
    public const string Manager = $"{_resource}Manager";
    public const string Reader = $"{_resource}Reader";

    // Cedar ABAC Resource:Action Yetki Formatı
    public const string Read = $"{_resource}:Read";
    public const string Write = $"{_resource}:Write";
    public const string Create = $"{_resource}:Create";
    public const string Update = $"{_resource}:Update";
    public const string Delete = $"{_resource}:Delete";

    // Cedar Action İsimleri
    public const string CedarActionCreate = $"{_resource}Create";
    public const string CedarActionUpdate = $"{_resource}Update";
    public const string CedarActionDelete = $"{_resource}Delete";
    public const string CedarActionRead = $"{_resource}Read";
}
