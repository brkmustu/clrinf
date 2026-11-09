using Microsoft.CodeAnalysis;

namespace ClrinfCS.Generators;

internal static class CrudDiagnostics
{
    private const string Category = "ClrinfCS.Crud";

    public static readonly DiagnosticDescriptor MissingId = new(
        id: "CLRCRUD001",
        title: "CRUD entity needs an Id property",
        messageFormat: "Entity '{0}' is marked with [GenerateCrudEndpoints] but has no public 'Id' property",
        category: Category,
        defaultSeverity: DiagnosticSeverity.Error,
        isEnabledByDefault: true);

    public static readonly DiagnosticDescriptor ExcludedOperationWithoutOverride = new(
        id: "CLRCRUD002",
        title: "CRUD operation excluded without a manual implementation",
        messageFormat: "Entity '{0}' excludes '{1}' but no '{2}' type was found; the operation will not be routed",
        category: Category,
        defaultSeverity: DiagnosticSeverity.Warning,
        isEnabledByDefault: true);

    public static readonly DiagnosticDescriptor MediatorNetUnavailable = new(
        id: "CLRCRUD003",
        title: "Mediator.Net dispatcher selected but package is unavailable",
        messageFormat: "Entity '{0}' selects MediatorNet but Mediator.Net.Contracts types cannot be resolved; add the Mediator.Net package or switch ClrinfCrudDispatcher to native",
        category: Category,
        defaultSeverity: DiagnosticSeverity.Error,
        isEnabledByDefault: true);
}
