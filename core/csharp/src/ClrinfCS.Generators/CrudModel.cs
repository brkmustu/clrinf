using System.Collections.Immutable;
using Microsoft.CodeAnalysis;
using Microsoft.CodeAnalysis.Text;

namespace ClrinfCS.Generators;

/// <summary>Mirror of ClrinfCS.Core.CrudOperations. The generator never references the runtime assembly.</summary>
[System.Flags]
internal enum CrudOperationFlags
{
    None = 0,
    Create = 1,
    Update = 2,
    Delete = 4,
    GetById = 8,
    GetList = 16,
    All = Create | Update | Delete | GetById | GetList
}

internal enum DispatcherKind
{
    Default = 0,
    Native = 1,
    MediatorNet = 2
}

internal enum CrudOperationKind
{
    Create,
    Update,
    Delete,
    GetById,
    GetList
}

internal enum CrudPropertyJsonKind
{
    String,
    NullableString,
    Boolean,
    NullableBoolean,
    Int32,
    NullableInt32,
    Int64,
    NullableInt64,
    Int16,
    NullableInt16,
    Byte,
    NullableByte,
    UInt32,
    NullableUInt32,
    UInt64,
    NullableUInt64,
    Decimal,
    NullableDecimal,
    Double,
    NullableDouble,
    Single,
    NullableSingle,
    Guid,
    NullableGuid,
    DateTime,
    NullableDateTime,
    DateTimeOffset,
    NullableDateTimeOffset,
    DateOnly,
    NullableDateOnly,
    TimeOnly,
    NullableTimeOnly,
    Enum,
    NullableEnum,
    Parsable,
    NullableParsable
}

internal readonly struct CrudPropertyModel
{
    public CrudPropertyModel(
        string name,
        string type,
        bool writable,
        CrudPropertyJsonKind jsonKind = CrudPropertyJsonKind.String,
        string underlyingType = "string")
    {
        Name = name;
        CamelName = string.IsNullOrEmpty(name)
            ? string.Empty
            : char.ToLowerInvariant(name[0]) + name.Substring(1);
        Type = type;
        Writable = writable;
        JsonKind = jsonKind;
        UnderlyingType = underlyingType;
    }

    public string Name { get; }
    public string CamelName { get; }
    public string Type { get; }
    public bool Writable { get; }
    public CrudPropertyJsonKind JsonKind { get; }
    public string UnderlyingType { get; }
}

internal sealed class CrudEntityModel
{    public CrudEntityModel(
        string name,
        string entityNamespace,
        string featureNamespace,
        string pluralName,
        string idPropertyName,
        string idType,
        string routePrefix,
        bool secured,
        bool hasAspNetCore,
        bool hasMediatorNet,
        DispatcherKind dispatcher,
        CrudOperationFlags operations,
        ImmutableArray<CrudPropertyModel> properties)
    {
        Name = name;
        EntityNamespace = entityNamespace;
        EntityType = string.IsNullOrEmpty(entityNamespace)
            ? "global::" + name
            : "global::" + entityNamespace + "." + name;
        FeatureNamespace = featureNamespace;
        PluralName = pluralName;
        IdPropertyName = idPropertyName;
        IdType = idType;
        RoutePrefix = routePrefix;
        Secured = secured;
        HasAspNetCore = hasAspNetCore;
        HasMediatorNet = hasMediatorNet;
        Dispatcher = dispatcher;
        Operations = operations;
        Properties = properties;
    }

    public string Name { get; }
    public string EntityNamespace { get; }
    public string EntityType { get; }
    public string FeatureNamespace { get; }
    public string PluralName { get; }
    public string IdPropertyName { get; }
    public string IdType { get; }
    public string RoutePrefix { get; }
    public bool Secured { get; }
    public bool HasAspNetCore { get; }
    public bool HasMediatorNet { get; }
    public DispatcherKind Dispatcher { get; }
    public ImmutableArray<CrudPropertyModel> Properties { get; }

    public string LowerEntityName => Name.ToLowerInvariant();

    public string HandlerName(CrudOperationKind operation) => operation + Name + "Handler";
    public string EndpointName(CrudOperationKind operation) => operation + Name + "Endpoint";
    public string RequestName(CrudOperationKind operation) => operation + Name + "Request";

    public bool Includes(CrudOperationKind operation) => operation switch
    {
        CrudOperationKind.Create => (Operations & CrudOperationFlags.Create) != 0,
        CrudOperationKind.Update => (Operations & CrudOperationFlags.Update) != 0,
        CrudOperationKind.Delete => (Operations & CrudOperationFlags.Delete) != 0,
        CrudOperationKind.GetById => (Operations & CrudOperationFlags.GetById) != 0,
        CrudOperationKind.GetList => (Operations & CrudOperationFlags.GetList) != 0,
        _ => false
    };

    public CrudOperationFlags Operations { get; }

    public string Policy(CrudOperationKind operation) =>
        LowerEntityName + "." + operation.ToString().ToLowerInvariant();
}

/// <summary>Snapshot of the compile-time environment; kept small so the incremental cache stays useful.</summary>
internal readonly struct BuildEnvironment
{
    public BuildEnvironment(bool hasAspNetCore, bool hasMediatorNet)
    {
        HasAspNetCore = hasAspNetCore;
        HasMediatorNet = hasMediatorNet;
    }

    public bool HasAspNetCore { get; }
    public bool HasMediatorNet { get; }
}

internal readonly struct BuildOptions
{
    public BuildOptions(string? dispatcher, bool secured)
    {
        Dispatcher = dispatcher;
        Secured = secured;
    }

    public string? Dispatcher { get; }
    public bool Secured { get; }
}

/// <summary>Attribute-level data captured by the incremental syntax transform.</summary>
internal sealed class RawEntityModel
{
    public RawEntityModel(
        string name,
        string entityNamespace,
        string? routePrefix,
        DispatcherKind attributeDispatcher,
        CrudOperationFlags operations,
        string idPropertyName,
        string idType,
        ImmutableArray<CrudPropertyModel> properties)
    {
        Name = name;
        EntityNamespace = entityNamespace;
        RoutePrefix = routePrefix;
        AttributeDispatcher = attributeDispatcher;
        Operations = operations;
        IdPropertyName = idPropertyName;
        IdType = idType;
        Properties = properties;
    }

    public string Name { get; }
    public string EntityNamespace { get; }
    public string? RoutePrefix { get; }
    public DispatcherKind AttributeDispatcher { get; }
    public CrudOperationFlags Operations { get; }
    public string IdPropertyName { get; }
    public string IdType { get; }
    public ImmutableArray<CrudPropertyModel> Properties { get; }
}

internal static class SourceTextFactory
{
    public static SourceText Create(string code) => SourceText.From(code, System.Text.Encoding.UTF8);
}
