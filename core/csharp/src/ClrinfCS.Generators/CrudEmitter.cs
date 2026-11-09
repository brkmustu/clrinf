using System.Collections.Generic;
using System.Linq;
using System.Text;

namespace ClrinfCS.Generators;

internal sealed class CrudEntityPlan
{
    public CrudEntityPlan(CrudEntityModel entity, bool[] generated, bool[] handlerDeclared, bool[] endpointDeclared)
    {
        Entity = entity;
        Generated = generated;
        HandlerDeclared = handlerDeclared;
        EndpointDeclared = endpointDeclared;
    }

    public CrudEntityModel Entity { get; }
    public bool[] Generated { get; }
    public bool[] HandlerDeclared { get; }
    public bool[] EndpointDeclared { get; }
}

internal static class CrudEmitter
{
    private const string GeneratedNamespace = "ClrinfCS.Generated";

    public static string EmitModels(CrudEntityModel entity)
    {
        var sb = Header();
        sb.AppendLine("using System.IO;");
        sb.AppendLine("using System.Text.Json;");
        sb.AppendLine("using ClrinfCS.Core;");
        sb.AppendLine();
        OpenNamespace(sb, entity.FeatureNamespace);

        // DTO: all readable properties, including Id.
        sb.AppendLine($"/// <summary>Wire representation of <see cref=\"{entity.EntityType}\"/>.</summary>");
        sb.AppendLine($"public sealed record {entity.Name}Dto");
        sb.AppendLine("{");
        foreach (var property in entity.Properties)
        {
            sb.AppendLine($"    public {property.Type} {property.Name} {{ get; init; }} = default!;");
        }
        sb.AppendLine("}");
        sb.AppendLine();

        sb.AppendLine($"internal static class {entity.Name}CrudMapper");
        sb.AppendLine("{");
        sb.AppendLine($"    public static {entity.Name}Dto ToDto({entity.EntityType} entity) => new()");
        sb.AppendLine("    {");
        foreach (var property in entity.Properties)
        {
            sb.AppendLine($"        {property.Name} = entity.{property.Name},");
        }
        sb.AppendLine("    };");
        sb.AppendLine();
        sb.AppendLine($"    public static void WriteDto(Utf8JsonWriter writer, {entity.Name}Dto value)");
        sb.AppendLine("    {");
        sb.AppendLine("        writer.WriteStartObject();");
        foreach (var property in entity.Properties)
        {
            EmitWriteProperty(sb, property);
        }
        sb.AppendLine("        writer.WriteEndObject();");
        sb.AppendLine("    }");
        sb.AppendLine();
        sb.AppendLine($"    public static byte[] ToJsonBytes({entity.Name}Dto value)");
        sb.AppendLine("    {");
        sb.AppendLine("        using var buffer = new MemoryStream();");
        sb.AppendLine("        using (var writer = new Utf8JsonWriter(buffer))");
        sb.AppendLine("        {");
        sb.AppendLine("            WriteDto(writer, value);");
        sb.AppendLine("        }");
        sb.AppendLine("        return buffer.ToArray();");
        sb.AppendLine("    }");
        sb.AppendLine();
        sb.AppendLine($"    public static byte[] ToJsonBytes(PagedResult<{entity.Name}Dto> page)");
        sb.AppendLine("    {");
        sb.AppendLine("        using var buffer = new MemoryStream();");
        sb.AppendLine("        using (var writer = new Utf8JsonWriter(buffer))");
        sb.AppendLine("        {");
        sb.AppendLine("            writer.WriteStartObject();");
        sb.AppendLine("            writer.WriteStartArray(\"items\");");
        sb.AppendLine("            foreach (var item in page.Items)");
        sb.AppendLine("            {");
        sb.AppendLine("                WriteDto(writer, item);");
        sb.AppendLine("            }");
        sb.AppendLine("            writer.WriteEndArray();");
        sb.AppendLine("            writer.WriteNumber(\"page\", page.Page);");
        sb.AppendLine("            writer.WriteNumber(\"pageSize\", page.PageSize);");
        sb.AppendLine("            writer.WriteNumber(\"totalCount\", page.TotalCount);");
        sb.AppendLine("            writer.WriteNumber(\"totalPages\", page.TotalPages);");
        sb.AppendLine("            writer.WriteEndObject();");
        sb.AppendLine("        }");
        sb.AppendLine("        return buffer.ToArray();");
        sb.AppendLine("    }");
        sb.AppendLine("}");
        CloseNamespace(sb);
        return sb.ToString();
    }

    private static void EmitWriteProperty(StringBuilder sb, CrudPropertyModel property)
    {
        switch (property.JsonKind)
        {
            case CrudPropertyJsonKind.Enum:
                sb.AppendLine($"        writer.WriteString(\"{property.CamelName}\", value.{property.Name}.ToString());");
                break;
            case CrudPropertyJsonKind.NullableEnum:
                sb.AppendLine($"        if (value.{property.Name}.HasValue) writer.WriteString(\"{property.CamelName}\", value.{property.Name}.Value.ToString()); else writer.WriteNull(\"{property.CamelName}\");");
                break;
            case CrudPropertyJsonKind.Parsable:
            case CrudPropertyJsonKind.NullableParsable:
                sb.AppendLine($"        if (value.{property.Name} is null) writer.WriteNull(\"{property.CamelName}\"); else writer.WriteString(\"{property.CamelName}\", value.{property.Name}.ToString());");
                break;
            default:
                sb.AppendLine($"        global::{GeneratedNamespace}.GeneratedCrudJsonSupport.WriteProperty(writer, \"{property.CamelName}\", value.{property.Name});");
                break;
        }
    }

    public static string EmitHandler(CrudEntityModel entity, CrudOperationKind operation)
    {
        var sb = Header();
        sb.AppendLine("using System;");
        sb.AppendLine("using System.Collections.Generic;");
        sb.AppendLine("using System.IO;");
        sb.AppendLine("using System.Linq;");
        sb.AppendLine("using System.Text.Json;");
        sb.AppendLine("using System.Text.Json.Serialization;");
        sb.AppendLine("using System.Threading;");
        sb.AppendLine("using System.Threading.Tasks;");
        sb.AppendLine("using ClrinfCS.Core;");
        if (entity.Dispatcher == DispatcherKind.MediatorNet)
        {
            sb.AppendLine("using MediatorNet = global::Mediator.Net.Contracts;");
        }
        sb.AppendLine();
        OpenNamespace(sb, entity.FeatureNamespace);

        EmitRequestRecord(sb, entity, operation);
        sb.AppendLine();

        var handler = entity.HandlerName(operation);
        var request = entity.RequestName(operation);
        var response = ResponseType(entity, operation);
        var mediatorInterface = entity.Dispatcher == DispatcherKind.MediatorNet
            ? $" : MediatorNet.IRequestHandler<{request}, {response}>"
            : string.Empty;

        sb.AppendLine($"/// <summary>Generated {operation} handler for {entity.Name}.</summary>");
        sb.AppendLine($"public sealed class {handler}{mediatorInterface}");
        sb.AppendLine("{");
        sb.AppendLine($"    private readonly ICrudStore<{entity.EntityType}, {entity.IdType}> _store;");
        var ruleInterface = RuleInterface(operation);
        if (ruleInterface is not null)
        {
            sb.AppendLine($"    private readonly IEnumerable<{ruleInterface}<{entity.EntityType}>> _rules;");
        }
        sb.AppendLine();

        sb.AppendLine($"    public {handler}(");
        sb.AppendLine($"        ICrudStore<{entity.EntityType}, {entity.IdType}> store" +
                      (ruleInterface is not null
                          ? $",\n        IEnumerable<{ruleInterface}<{entity.EntityType}>> rules)"
                          : ")"));
        sb.AppendLine("    {");
        sb.AppendLine("        _store = store ?? throw new ArgumentNullException(nameof(store));");
        if (ruleInterface is not null)
        {
            sb.AppendLine("        _rules = rules ?? throw new ArgumentNullException(nameof(rules));");
        }
        sb.AppendLine("    }");
        sb.AppendLine();

        EmitHandleAsync(sb, entity, operation, response);
        if (entity.Dispatcher == DispatcherKind.MediatorNet)
        {
            sb.AppendLine();
            sb.AppendLine($"    Task<{response}> MediatorNet.IRequestHandler<{request}, {response}>.Handle(");
            sb.AppendLine($"        MediatorNet.IReceiveContext<{request}> receiveContext, CancellationToken cancellationToken)");
            sb.AppendLine("    {");
            sb.AppendLine("        var context = receiveContext.Message.Context");
            sb.AppendLine("            ?? throw new InvalidOperationException(\"RequestContext was not supplied by the endpoint.\");");
            sb.AppendLine("        return HandleAsync(receiveContext.Message, context, cancellationToken).AsTask();");
            sb.AppendLine("    }");
        }

        sb.AppendLine("}");
        CloseNamespace(sb);
        return sb.ToString();
    }

    public static string EmitEndpoint(CrudEntityModel entity, CrudOperationKind operation)
    {
        var sb = Header();
        sb.AppendLine("using System;");
        sb.AppendLine("using System.Threading;");
        sb.AppendLine("using System.Threading.Tasks;");
        sb.AppendLine("using ClrinfCS.Core;");
        sb.AppendLine($"using global::{GeneratedNamespace};");
        sb.AppendLine("using Microsoft.AspNetCore.Builder;");
        sb.AppendLine("using Microsoft.AspNetCore.Http;");
        sb.AppendLine("using Microsoft.AspNetCore.Routing;");
        sb.AppendLine("using Microsoft.Extensions.DependencyInjection;");
        if (entity.Dispatcher == DispatcherKind.MediatorNet)
        {
            sb.AppendLine("using MediatorNet = global::Mediator.Net.Contracts;");
        }
        sb.AppendLine();
        OpenNamespace(sb, entity.FeatureNamespace);
        sb.AppendLine($"public static class {entity.EndpointName(operation)}");
        sb.AppendLine("{");

        var mapMethod = MapMethod(operation);
        var pattern = Pattern(operation);
        var request = entity.RequestName(operation);
        var response = ResponseType(entity, operation);

        sb.AppendLine("    public static IEndpointConventionBuilder Map(");
        sb.AppendLine("        IEndpointRouteBuilder group,");
        sb.AppendLine("        Func<HttpContext, RequestContext> contextFactory)");
        sb.AppendLine("    {");
        sb.AppendLine($"        var builder = group.{mapMethod}(\"{pattern}\", (RequestDelegate)(async http =>");
        sb.AppendLine("        {");
        sb.AppendLine("            var cancellationToken = http.RequestAborted;");
        sb.AppendLine("            var context = contextFactory(http);");
        sb.AppendLine("            var responseResult = await global::" + GeneratedNamespace + ".GeneratedCrudEndpointSupport.ExecuteAsync(context, async () =>");
        sb.AppendLine("            {");

        EmitEndpointBody(sb, entity, operation, request, response);

        sb.AppendLine("            }).ConfigureAwait(false);");
        sb.AppendLine("            await responseResult.ExecuteAsync(http).ConfigureAwait(false);");
        sb.AppendLine("        }));");
        sb.AppendLine();
        sb.AppendLine($"        builder.WithName(\"{operation}{entity.Name}\");");
        sb.AppendLine($"        builder.WithTags(\"{entity.PluralName}\");");
        EmitEndpointMetadata(sb, entity, operation, request);
        EmitProduces(sb, entity, operation);
        if (entity.Secured)
        {
            sb.AppendLine($"        builder.RequireAuthorization(\"{entity.Policy(operation)}\");");
        }
        sb.AppendLine("        return builder;");
        sb.AppendLine("    }");
        sb.AppendLine("}");
        CloseNamespace(sb);
        return sb.ToString();
    }

    private static void EmitEndpointMetadata(StringBuilder sb, CrudEntityModel entity, CrudOperationKind operation, string request)
    {
        switch (operation)
        {
            case CrudOperationKind.Create:
                sb.AppendLine($"        builder.Accepts<{request}>();");
                break;
            case CrudOperationKind.Update:
                sb.AppendLine($"        builder.WithMetadata(new CrudEndpointParameterMetadata(\"id\", \"path\", typeof({entity.IdType}), true));");
                sb.AppendLine($"        builder.Accepts<{request}>();");
                break;
            case CrudOperationKind.Delete:
            case CrudOperationKind.GetById:
                sb.AppendLine($"        builder.WithMetadata(new CrudEndpointParameterMetadata(\"id\", \"path\", typeof({entity.IdType}), true));");
                break;
            case CrudOperationKind.GetList:
                sb.AppendLine("        builder.WithMetadata(new CrudEndpointParameterMetadata(\"page\", \"query\", typeof(int?), false));");
                sb.AppendLine("        builder.WithMetadata(new CrudEndpointParameterMetadata(\"pageSize\", \"query\", typeof(int?), false));");
                sb.AppendLine("        builder.WithMetadata(new CrudEndpointParameterMetadata(\"sort\", \"query\", typeof(string), false));");
                break;
        }
    }

    private static void EmitProduces(StringBuilder sb, CrudEntityModel entity, CrudOperationKind operation)
    {
        var dto = $"{entity.Name}Dto";
        switch (operation)
        {
            case CrudOperationKind.Create:
                sb.AppendLine($"        builder.Produces<{dto}>(201);");
                sb.AppendLine("        builder.Produces<ErrorEnvelope>(422);");
                break;
            case CrudOperationKind.Update:
                sb.AppendLine($"        builder.Produces<{dto}>(200);");
                sb.AppendLine("        builder.Produces(404);");
                sb.AppendLine("        builder.Produces<ErrorEnvelope>(422);");
                sb.AppendLine("        builder.Produces<ErrorEnvelope>(409);");
                break;
            case CrudOperationKind.Delete:
                sb.AppendLine("        builder.Produces(204);");
                sb.AppendLine("        builder.Produces(404);");
                sb.AppendLine("        builder.Produces<ErrorEnvelope>(422);");
                sb.AppendLine("        builder.Produces<ErrorEnvelope>(409);");
                break;
            case CrudOperationKind.GetById:
                sb.AppendLine($"        builder.Produces<{dto}>(200);");
                sb.AppendLine("        builder.Produces(404);");
                break;
            case CrudOperationKind.GetList:
                sb.AppendLine($"        builder.Produces<PagedResult<{dto}>>(200);");
                sb.AppendLine($"        global::{GeneratedNamespace}.GeneratedCrudEndpointSupport.PreserveSchemaType<{dto}>();");
                break;
        }
    }

    private static void EmitEndpointBody(StringBuilder sb, CrudEntityModel entity, CrudOperationKind operation,
        string request, string response)
    {
        var mediator = entity.Dispatcher == DispatcherKind.MediatorNet;
        var handlerName = entity.HandlerName(operation);
        switch (operation)
        {
            case CrudOperationKind.Create:
                sb.AppendLine($"                var request = await {request}.ReadJsonAsync(http.Request.Body, cancellationToken).ConfigureAwait(false);");
                sb.AppendLine("                if (request is null) return Microsoft.AspNetCore.Http.Results.BadRequest();");
                if (mediator)
                {
                    sb.AppendLine("                var mediator = http.RequestServices.GetRequiredService<MediatorNet.IMediator>();");
                    sb.AppendLine("                var command = request with { Context = context };");
                    sb.AppendLine($"                var created = await mediator.RequestAsync<{request}, {response}>(command, cancellationToken).ConfigureAwait(false);");
                }
                else
                {
                    sb.AppendLine($"                var handler = http.RequestServices.GetRequiredService<{handlerName}>();");
                    sb.AppendLine("                var created = await handler.HandleAsync(request, context, cancellationToken).ConfigureAwait(false);");
                }
                sb.AppendLine($"                return global::{GeneratedNamespace}.GeneratedCrudEndpointSupport.Results.Created(http, $\"{entity.RoutePrefix}/{{created.{entity.IdPropertyName}}}\", {entity.Name}CrudMapper.ToJsonBytes(created));");
                break;

            case CrudOperationKind.Update:
                sb.AppendLine($"                if (!global::{GeneratedNamespace}.GeneratedCrudEndpointSupport.TryParseRoute<{entity.IdType}>(http, \"id\", out var id)) return Microsoft.AspNetCore.Http.Results.BadRequest();");
                sb.AppendLine($"                var request = await {request}.ReadJsonAsync(http.Request.Body, cancellationToken).ConfigureAwait(false);");
                sb.AppendLine("                if (request is null) return Microsoft.AspNetCore.Http.Results.BadRequest();");
                if (mediator)
                {
                    sb.AppendLine("                var mediator = http.RequestServices.GetRequiredService<MediatorNet.IMediator>();");
                    sb.AppendLine("                var command = request with { Id = id, Context = context };");
                    sb.AppendLine($"                var updated = await mediator.RequestAsync<{request}, {response}>(command, cancellationToken).ConfigureAwait(false);");
                }
                else
                {
                    sb.AppendLine($"                var handler = http.RequestServices.GetRequiredService<{handlerName}>();");
                    sb.AppendLine("                request = request with { Id = id };");
                    sb.AppendLine("                var updated = await handler.HandleAsync(request, context, cancellationToken).ConfigureAwait(false);");
                }
                sb.AppendLine($"                return updated is null ? Microsoft.AspNetCore.Http.Results.NotFound() : global::{GeneratedNamespace}.GeneratedCrudEndpointSupport.Results.Ok({entity.Name}CrudMapper.ToJsonBytes(updated));");
                break;

            case CrudOperationKind.Delete:
                sb.AppendLine($"                if (!global::{GeneratedNamespace}.GeneratedCrudEndpointSupport.TryParseRoute<{entity.IdType}>(http, \"id\", out var id)) return Microsoft.AspNetCore.Http.Results.BadRequest();");
                if (mediator)
                {
                    sb.AppendLine("                var mediator = http.RequestServices.GetRequiredService<MediatorNet.IMediator>();");
                    sb.AppendLine($"                var command = new {request} {{ Id = id, Context = context }};");
                    sb.AppendLine($"                var deleted = await mediator.RequestAsync<{request}, {response}>(command, cancellationToken).ConfigureAwait(false);");
                }
                else
                {
                    sb.AppendLine($"                var handler = http.RequestServices.GetRequiredService<{handlerName}>();");
                    sb.AppendLine($"                var deleted = await handler.HandleAsync(new {request} {{ Id = id }}, context, cancellationToken).ConfigureAwait(false);");
                }
                sb.AppendLine("                return deleted ? Microsoft.AspNetCore.Http.Results.NoContent() : Microsoft.AspNetCore.Http.Results.NotFound();");
                break;

            case CrudOperationKind.GetById:
                sb.AppendLine($"                if (!global::{GeneratedNamespace}.GeneratedCrudEndpointSupport.TryParseRoute<{entity.IdType}>(http, \"id\", out var id)) return Microsoft.AspNetCore.Http.Results.BadRequest();");
                if (mediator)
                {
                    sb.AppendLine("                var mediator = http.RequestServices.GetRequiredService<MediatorNet.IMediator>();");
                    sb.AppendLine($"                var query = new {request} {{ Id = id, Context = context }};");
                    sb.AppendLine($"                var item = await mediator.RequestAsync<{request}, {response}>(query, cancellationToken).ConfigureAwait(false);");
                }
                else
                {
                    sb.AppendLine($"                var handler = http.RequestServices.GetRequiredService<{handlerName}>();");
                    sb.AppendLine($"                var item = await handler.HandleAsync(new {request} {{ Id = id }}, context, cancellationToken).ConfigureAwait(false);");
                }
                sb.AppendLine($"                return item is null ? Microsoft.AspNetCore.Http.Results.NotFound() : global::{GeneratedNamespace}.GeneratedCrudEndpointSupport.Results.Ok({entity.Name}CrudMapper.ToJsonBytes(item));");
                break;

            case CrudOperationKind.GetList:
                sb.AppendLine($"                var page = global::{GeneratedNamespace}.GeneratedCrudEndpointSupport.GetQueryInt32(http, \"page\");");
                sb.AppendLine($"                var pageSize = global::{GeneratedNamespace}.GeneratedCrudEndpointSupport.GetQueryInt32(http, \"pageSize\");");
                sb.AppendLine($"                var sort = global::{GeneratedNamespace}.GeneratedCrudEndpointSupport.GetQueryString(http, \"sort\");");
                if (mediator)
                {
                    sb.AppendLine("                var mediator = http.RequestServices.GetRequiredService<MediatorNet.IMediator>();");
                    sb.AppendLine($"                var query = new {request} {{ Page = page ?? 1, PageSize = pageSize ?? CrudQuery.DefaultPageSize, Sort = sort, Context = context }};");
                    sb.AppendLine($"                var result = await mediator.RequestAsync<{request}, {response}>(query, cancellationToken).ConfigureAwait(false);");
                }
                else
                {
                    sb.AppendLine($"                var handler = http.RequestServices.GetRequiredService<{handlerName}>();");
                    sb.AppendLine($"                var query = new {request} {{ Page = page ?? 1, PageSize = pageSize ?? CrudQuery.DefaultPageSize, Sort = sort }};");
                    sb.AppendLine("                var result = await handler.HandleAsync(query, context, cancellationToken).ConfigureAwait(false);");
                }
                sb.AppendLine($"                return global::{GeneratedNamespace}.GeneratedCrudEndpointSupport.Results.Ok({entity.Name}CrudMapper.ToJsonBytes(result));");
                break;
        }
    }

    public static string EmitCentral(IReadOnlyList<CrudEntityPlan> entities)
    {
        var sb = Header();
        sb.AppendLine("using System;");
        sb.AppendLine("using ClrinfCS.Core;");
        sb.AppendLine("using Microsoft.AspNetCore.Builder;");
        sb.AppendLine("using Microsoft.AspNetCore.Http;");
        sb.AppendLine("using Microsoft.AspNetCore.Routing;");
        sb.AppendLine("using Microsoft.Extensions.DependencyInjection;");
        sb.AppendLine("using Microsoft.Extensions.DependencyInjection.Extensions;");
        sb.AppendLine();
        sb.AppendLine($"namespace {GeneratedNamespace};");
        sb.AppendLine();
        sb.AppendLine("/// <summary>Generated DI and endpoint registration. No reflection or assembly scanning.</summary>");
        sb.AppendLine("public static class GeneratedCrudExtensions");
        sb.AppendLine("{");
        sb.AppendLine("    public static IServiceCollection AddGeneratedCrud(this IServiceCollection services)");
        sb.AppendLine("    {");
        sb.AppendLine("        ArgumentNullException.ThrowIfNull(services);");
        foreach (var plan in entities)
        {
            var entity = plan.Entity;
            for (var i = 0; i < Operations.Length; i++)
            {
                if (plan.Generated[i] || plan.HandlerDeclared[i])
                {
                    sb.AppendLine($"        services.TryAddScoped<{entity.FeatureNamespace}.{entity.HandlerName(Operations[i])}>();");
                }
            }
        }
        sb.AppendLine("        return services;");
        sb.AppendLine("    }");
        sb.AppendLine();
        sb.AppendLine("    public static IEndpointRouteBuilder MapGeneratedCrudEndpoints(");
        sb.AppendLine("        this IEndpointRouteBuilder app,");
        sb.AppendLine("        Func<HttpContext, RequestContext> contextFactory)");
        sb.AppendLine("    {");
        sb.AppendLine("        ArgumentNullException.ThrowIfNull(app);");
        sb.AppendLine("        ArgumentNullException.ThrowIfNull(contextFactory);");
        foreach (var plan in entities)
        {
            var entity = plan.Entity;
            var mapped = false;
            for (var i = 0; i < Operations.Length; i++)
            {
                if (plan.Generated[i] || plan.EndpointDeclared[i]) mapped = true;
            }
            if (!mapped) continue;

            sb.AppendLine();
            sb.AppendLine($"        var {Variable(entity)}Group = app.MapGroup(\"{entity.RoutePrefix}\");");
            if (entity.Secured)
            {
                sb.AppendLine($"        {Variable(entity)}Group.RequireAuthorization();");
            }
            for (var i = 0; i < Operations.Length; i++)
            {
                if (!plan.Generated[i] && !plan.EndpointDeclared[i]) continue;
                sb.AppendLine($"        {entity.FeatureNamespace}.{entity.EndpointName(Operations[i])}.Map({Variable(entity)}Group, contextFactory);");
            }
        }
        sb.AppendLine("        return app;");
        sb.AppendLine("    }");
        sb.AppendLine("}");
        return sb.ToString();
    }

    public static string EmitDi(IReadOnlyList<CrudEntityPlan> entities)
    {
        var sb = Header();
        sb.AppendLine("using System;");
        sb.AppendLine("using Microsoft.Extensions.DependencyInjection;");
        sb.AppendLine("using Microsoft.Extensions.DependencyInjection.Extensions;");
        sb.AppendLine();
        sb.AppendLine($"namespace {GeneratedNamespace};");
        sb.AppendLine();
        sb.AppendLine("public static class GeneratedCrudExtensions");
        sb.AppendLine("{");
        sb.AppendLine("    public static IServiceCollection AddGeneratedCrud(this IServiceCollection services)");
        sb.AppendLine("    {");
        sb.AppendLine("        ArgumentNullException.ThrowIfNull(services);");
        foreach (var plan in entities)
        {
            var entity = plan.Entity;
            for (var i = 0; i < Operations.Length; i++)
            {
                if (plan.Generated[i] || plan.HandlerDeclared[i])
                {
                    sb.AppendLine($"        services.TryAddScoped<{entity.FeatureNamespace}.{entity.HandlerName(Operations[i])}>();");
                }
            }
        }
        sb.AppendLine("        return services;");
        sb.AppendLine("    }");
        sb.AppendLine("}");
        return sb.ToString();
    }

    public static string EmitJsonSupport()
    {
        var sb = Header();
        sb.AppendLine("using System;");
        sb.AppendLine("using System.Globalization;");
        sb.AppendLine("using System.IO;");
        sb.AppendLine("using System.Text.Json;");
        sb.AppendLine("using System.Threading;");
        sb.AppendLine("using System.Threading.Tasks;");
        sb.AppendLine();
        sb.AppendLine($"namespace {GeneratedNamespace};");
        sb.AppendLine();
        sb.AppendLine("/// <summary>AOT-safe UTF-8 JSON readers and writers for generated CRUD models.</summary>");
        sb.AppendLine("internal static class GeneratedCrudJsonSupport");
        sb.AppendLine("{");
        sb.AppendLine("    public static async ValueTask<JsonDocument?> TryParseJsonAsync(Stream body, CancellationToken cancellationToken)");
        sb.AppendLine("    {");
        sb.AppendLine("        try");
        sb.AppendLine("        {");
        sb.AppendLine("            return await JsonDocument.ParseAsync(body, cancellationToken: cancellationToken).ConfigureAwait(false);");
        sb.AppendLine("        }");
        sb.AppendLine("        catch (JsonException)");
        sb.AppendLine("        {");
        sb.AppendLine("            return null;");
        sb.AppendLine("        }");
        sb.AppendLine("    }");
        sb.AppendLine();
        sb.AppendLine("    public static void WriteProperty(Utf8JsonWriter writer, string name, string? value) => writer.WriteString(name, value);");
        sb.AppendLine("    public static void WriteProperty(Utf8JsonWriter writer, string name, bool value) => writer.WriteBoolean(name, value);");
        sb.AppendLine("    public static void WriteProperty(Utf8JsonWriter writer, string name, bool? value) { if (value.HasValue) writer.WriteBoolean(name, value.Value); else writer.WriteNull(name); }");
        sb.AppendLine("    public static void WriteProperty(Utf8JsonWriter writer, string name, int value) => writer.WriteNumber(name, value);");
        sb.AppendLine("    public static void WriteProperty(Utf8JsonWriter writer, string name, int? value) { if (value.HasValue) writer.WriteNumber(name, value.Value); else writer.WriteNull(name); }");
        sb.AppendLine("    public static void WriteProperty(Utf8JsonWriter writer, string name, long value) => writer.WriteNumber(name, value);");
        sb.AppendLine("    public static void WriteProperty(Utf8JsonWriter writer, string name, long? value) { if (value.HasValue) writer.WriteNumber(name, value.Value); else writer.WriteNull(name); }");
        sb.AppendLine("    public static void WriteProperty(Utf8JsonWriter writer, string name, short value) => writer.WriteNumber(name, value);");
        sb.AppendLine("    public static void WriteProperty(Utf8JsonWriter writer, string name, short? value) { if (value.HasValue) writer.WriteNumber(name, value.Value); else writer.WriteNull(name); }");
        sb.AppendLine("    public static void WriteProperty(Utf8JsonWriter writer, string name, byte value) => writer.WriteNumber(name, value);");
        sb.AppendLine("    public static void WriteProperty(Utf8JsonWriter writer, string name, byte? value) { if (value.HasValue) writer.WriteNumber(name, value.Value); else writer.WriteNull(name); }");
        sb.AppendLine("    public static void WriteProperty(Utf8JsonWriter writer, string name, uint value) => writer.WriteNumber(name, value);");
        sb.AppendLine("    public static void WriteProperty(Utf8JsonWriter writer, string name, uint? value) { if (value.HasValue) writer.WriteNumber(name, value.Value); else writer.WriteNull(name); }");
        sb.AppendLine("    public static void WriteProperty(Utf8JsonWriter writer, string name, ulong value) => writer.WriteNumber(name, value);");
        sb.AppendLine("    public static void WriteProperty(Utf8JsonWriter writer, string name, ulong? value) { if (value.HasValue) writer.WriteNumber(name, value.Value); else writer.WriteNull(name); }");
        sb.AppendLine("    public static void WriteProperty(Utf8JsonWriter writer, string name, decimal value) => writer.WriteNumber(name, value);");
        sb.AppendLine("    public static void WriteProperty(Utf8JsonWriter writer, string name, decimal? value) { if (value.HasValue) writer.WriteNumber(name, value.Value); else writer.WriteNull(name); }");
        sb.AppendLine("    public static void WriteProperty(Utf8JsonWriter writer, string name, double value) => writer.WriteNumber(name, value);");
        sb.AppendLine("    public static void WriteProperty(Utf8JsonWriter writer, string name, double? value) { if (value.HasValue) writer.WriteNumber(name, value.Value); else writer.WriteNull(name); }");
        sb.AppendLine("    public static void WriteProperty(Utf8JsonWriter writer, string name, float value) => writer.WriteNumber(name, value);");
        sb.AppendLine("    public static void WriteProperty(Utf8JsonWriter writer, string name, float? value) { if (value.HasValue) writer.WriteNumber(name, value.Value); else writer.WriteNull(name); }");
        sb.AppendLine("    public static void WriteProperty(Utf8JsonWriter writer, string name, Guid value) => writer.WriteString(name, value);");
        sb.AppendLine("    public static void WriteProperty(Utf8JsonWriter writer, string name, Guid? value) { if (value.HasValue) writer.WriteString(name, value.Value); else writer.WriteNull(name); }");
        sb.AppendLine("    public static void WriteProperty(Utf8JsonWriter writer, string name, DateTime value) => writer.WriteString(name, value);");
        sb.AppendLine("    public static void WriteProperty(Utf8JsonWriter writer, string name, DateTime? value) { if (value.HasValue) writer.WriteString(name, value.Value); else writer.WriteNull(name); }");
        sb.AppendLine("    public static void WriteProperty(Utf8JsonWriter writer, string name, DateTimeOffset value) => writer.WriteString(name, value);");
        sb.AppendLine("    public static void WriteProperty(Utf8JsonWriter writer, string name, DateTimeOffset? value) { if (value.HasValue) writer.WriteString(name, value.Value); else writer.WriteNull(name); }");
        sb.AppendLine("    public static void WriteProperty(Utf8JsonWriter writer, string name, DateOnly value) => writer.WriteString(name, value.ToString(\"O\", CultureInfo.InvariantCulture));");
        sb.AppendLine("    public static void WriteProperty(Utf8JsonWriter writer, string name, DateOnly? value) { if (value.HasValue) writer.WriteString(name, value.Value.ToString(\"O\", CultureInfo.InvariantCulture)); else writer.WriteNull(name); }");
        sb.AppendLine("    public static void WriteProperty(Utf8JsonWriter writer, string name, TimeOnly value) => writer.WriteString(name, value.ToString(\"O\", CultureInfo.InvariantCulture));");
        sb.AppendLine("    public static void WriteProperty(Utf8JsonWriter writer, string name, TimeOnly? value) { if (value.HasValue) writer.WriteString(name, value.Value.ToString(\"O\", CultureInfo.InvariantCulture)); else writer.WriteNull(name); }");
        sb.AppendLine();
        sb.AppendLine("    private static bool TryGet(JsonElement root, string camel, string pascal, out JsonElement element)");
        sb.AppendLine("    {");
        sb.AppendLine("        if (root.TryGetProperty(camel, out element) || root.TryGetProperty(pascal, out element))");
        sb.AppendLine("        {");
        sb.AppendLine("            return element.ValueKind != JsonValueKind.Null && element.ValueKind != JsonValueKind.Undefined;");
        sb.AppendLine("        }");
        sb.AppendLine("        foreach (var prop in root.EnumerateObject())");
        sb.AppendLine("        {");
        sb.AppendLine("            if (string.Equals(prop.Name, pascal, StringComparison.OrdinalIgnoreCase))");
        sb.AppendLine("            {");
        sb.AppendLine("                element = prop.Value;");
        sb.AppendLine("                return element.ValueKind != JsonValueKind.Null && element.ValueKind != JsonValueKind.Undefined;");
        sb.AppendLine("            }");
        sb.AppendLine("        }");
        sb.AppendLine("        element = default;");
        sb.AppendLine("        return false;");
        sb.AppendLine("    }");
        sb.AppendLine();
        sb.AppendLine("    public static string ReadString(JsonElement root, string camel, string pascal) => TryGet(root, camel, pascal, out var e) ? e.GetString() ?? string.Empty : string.Empty;");
        sb.AppendLine("    public static string? ReadNullableString(JsonElement root, string camel, string pascal) => TryGet(root, camel, pascal, out var e) ? e.GetString() : null;");
        sb.AppendLine("    public static bool ReadBoolean(JsonElement root, string camel, string pascal) => TryGet(root, camel, pascal, out var e) && (e.ValueKind == JsonValueKind.True || (e.ValueKind == JsonValueKind.String && bool.TryParse(e.GetString(), out var b) && b));");
        sb.AppendLine("    public static bool? ReadNullableBoolean(JsonElement root, string camel, string pascal) => TryGet(root, camel, pascal, out var e) ? ReadBoolean(root, camel, pascal) : null;");
        sb.AppendLine("    public static int ReadInt32(JsonElement root, string camel, string pascal) => TryGet(root, camel, pascal, out var e) && e.TryGetInt32(out var v) ? v : 0;");
        sb.AppendLine("    public static int? ReadNullableInt32(JsonElement root, string camel, string pascal) => TryGet(root, camel, pascal, out var e) && e.TryGetInt32(out var v) ? v : null;");
        sb.AppendLine("    public static long ReadInt64(JsonElement root, string camel, string pascal) => TryGet(root, camel, pascal, out var e) && e.TryGetInt64(out var v) ? v : 0L;");
        sb.AppendLine("    public static long? ReadNullableInt64(JsonElement root, string camel, string pascal) => TryGet(root, camel, pascal, out var e) && e.TryGetInt64(out var v) ? v : null;");
        sb.AppendLine("    public static short ReadInt16(JsonElement root, string camel, string pascal) => TryGet(root, camel, pascal, out var e) && e.TryGetInt16(out var v) ? v : (short)0;");
        sb.AppendLine("    public static short? ReadNullableInt16(JsonElement root, string camel, string pascal) => TryGet(root, camel, pascal, out var e) && e.TryGetInt16(out var v) ? v : null;");
        sb.AppendLine("    public static byte ReadByte(JsonElement root, string camel, string pascal) => TryGet(root, camel, pascal, out var e) && e.TryGetByte(out var v) ? v : (byte)0;");
        sb.AppendLine("    public static byte? ReadNullableByte(JsonElement root, string camel, string pascal) => TryGet(root, camel, pascal, out var e) && e.TryGetByte(out var v) ? v : null;");
        sb.AppendLine("    public static uint ReadUInt32(JsonElement root, string camel, string pascal) => TryGet(root, camel, pascal, out var e) && e.TryGetUInt32(out var v) ? v : 0U;");
        sb.AppendLine("    public static uint? ReadNullableUInt32(JsonElement root, string camel, string pascal) => TryGet(root, camel, pascal, out var e) && e.TryGetUInt32(out var v) ? v : null;");
        sb.AppendLine("    public static ulong ReadUInt64(JsonElement root, string camel, string pascal) => TryGet(root, camel, pascal, out var e) && e.TryGetUInt64(out var v) ? v : 0UL;");
        sb.AppendLine("    public static ulong? ReadNullableUInt64(JsonElement root, string camel, string pascal) => TryGet(root, camel, pascal, out var e) && e.TryGetUInt64(out var v) ? v : null;");
        sb.AppendLine("    public static decimal ReadDecimal(JsonElement root, string camel, string pascal) => TryGet(root, camel, pascal, out var e) && e.TryGetDecimal(out var v) ? v : 0m;");
        sb.AppendLine("    public static decimal? ReadNullableDecimal(JsonElement root, string camel, string pascal) => TryGet(root, camel, pascal, out var e) && e.TryGetDecimal(out var v) ? v : null;");
        sb.AppendLine("    public static double ReadDouble(JsonElement root, string camel, string pascal) => TryGet(root, camel, pascal, out var e) && e.TryGetDouble(out var v) ? v : 0d;");
        sb.AppendLine("    public static double? ReadNullableDouble(JsonElement root, string camel, string pascal) => TryGet(root, camel, pascal, out var e) && e.TryGetDouble(out var v) ? v : null;");
        sb.AppendLine("    public static float ReadSingle(JsonElement root, string camel, string pascal) => TryGet(root, camel, pascal, out var e) && e.TryGetSingle(out var v) ? v : 0f;");
        sb.AppendLine("    public static float? ReadNullableSingle(JsonElement root, string camel, string pascal) => TryGet(root, camel, pascal, out var e) && e.TryGetSingle(out var v) ? v : null;");
        sb.AppendLine("    public static Guid ReadGuid(JsonElement root, string camel, string pascal) => TryGet(root, camel, pascal, out var e) && e.TryGetGuid(out var v) ? v : Guid.Empty;");
        sb.AppendLine("    public static Guid? ReadNullableGuid(JsonElement root, string camel, string pascal) => TryGet(root, camel, pascal, out var e) && e.TryGetGuid(out var v) ? v : null;");
        sb.AppendLine("    public static DateTime ReadDateTime(JsonElement root, string camel, string pascal) => TryGet(root, camel, pascal, out var e) && e.TryGetDateTime(out var v) ? v : default;");
        sb.AppendLine("    public static DateTime? ReadNullableDateTime(JsonElement root, string camel, string pascal) => TryGet(root, camel, pascal, out var e) && e.TryGetDateTime(out var v) ? v : null;");
        sb.AppendLine("    public static DateTimeOffset ReadDateTimeOffset(JsonElement root, string camel, string pascal) => TryGet(root, camel, pascal, out var e) && e.TryGetDateTimeOffset(out var v) ? v : default;");
        sb.AppendLine("    public static DateTimeOffset? ReadNullableDateTimeOffset(JsonElement root, string camel, string pascal) => TryGet(root, camel, pascal, out var e) && e.TryGetDateTimeOffset(out var v) ? v : null;");
        sb.AppendLine("    public static DateOnly ReadDateOnly(JsonElement root, string camel, string pascal) => TryGet(root, camel, pascal, out var e) && DateOnly.TryParse(e.GetString(), CultureInfo.InvariantCulture, out var v) ? v : default;");
        sb.AppendLine("    public static DateOnly? ReadNullableDateOnly(JsonElement root, string camel, string pascal) => TryGet(root, camel, pascal, out var e) && DateOnly.TryParse(e.GetString(), CultureInfo.InvariantCulture, out var v) ? v : null;");
        sb.AppendLine("    public static TimeOnly ReadTimeOnly(JsonElement root, string camel, string pascal) => TryGet(root, camel, pascal, out var e) && TimeOnly.TryParse(e.GetString(), CultureInfo.InvariantCulture, out var v) ? v : default;");
        sb.AppendLine("    public static TimeOnly? ReadNullableTimeOnly(JsonElement root, string camel, string pascal) => TryGet(root, camel, pascal, out var e) && TimeOnly.TryParse(e.GetString(), CultureInfo.InvariantCulture, out var v) ? v : null;");
        sb.AppendLine("    public static TEnum ReadEnum<TEnum>(JsonElement root, string camel, string pascal) where TEnum : struct, Enum => TryGet(root, camel, pascal, out var e) && e.ValueKind == JsonValueKind.String && Enum.TryParse<TEnum>(e.GetString(), ignoreCase: true, out var v) ? v : default;");
        sb.AppendLine("    public static TEnum? ReadNullableEnum<TEnum>(JsonElement root, string camel, string pascal) where TEnum : struct, Enum => TryGet(root, camel, pascal, out var e) && e.ValueKind == JsonValueKind.String && Enum.TryParse<TEnum>(e.GetString(), ignoreCase: true, out var v) ? v : null;");
        sb.AppendLine("    public static T ReadParsable<T>(JsonElement root, string camel, string pascal) where T : IParsable<T> => TryGet(root, camel, pascal, out var e) && T.TryParse(e.ToString(), CultureInfo.InvariantCulture, out var v) && v is not null ? v : default!;");
        sb.AppendLine("    public static T? ReadNullableParsable<T>(JsonElement root, string camel, string pascal) where T : IParsable<T> => TryGet(root, camel, pascal, out var e) && T.TryParse(e.ToString(), CultureInfo.InvariantCulture, out var v) ? v : default;");
        sb.AppendLine("}");
        return sb.ToString();
    }

    public static string EmitSupport()
    {
        var sb = Header();
        sb.AppendLine("using System;");
        sb.AppendLine("using System.Diagnostics.CodeAnalysis;");
        sb.AppendLine("using System.Globalization;");
        sb.AppendLine("using System.IO;");
        sb.AppendLine("using System.Text.Json;");
        sb.AppendLine("using System.Threading.Tasks;");
        sb.AppendLine("using ClrinfCS.Core;");
        sb.AppendLine("using Microsoft.AspNetCore.Builder;");
        sb.AppendLine("using Microsoft.AspNetCore.Http;");
        sb.AppendLine("using Microsoft.AspNetCore.Http.Metadata;");
        sb.AppendLine();
        sb.AppendLine($"namespace {GeneratedNamespace};");
        sb.AppendLine();
        sb.AppendLine("/// <summary>Single HTTP error-mapping, parameter-binding and metadata point for all generated CRUD endpoints.</summary>");
        sb.AppendLine("internal static class GeneratedCrudEndpointSupport");
        sb.AppendLine("{");
        sb.AppendLine("    public static async Task<IResult> ExecuteAsync(RequestContext context, Func<Task<IResult>> action)");
        sb.AppendLine("    {");
        sb.AppendLine("        try");
        sb.AppendLine("        {");
        sb.AppendLine("            return await action().ConfigureAwait(false);");
        sb.AppendLine("        }");
        sb.AppendLine("        catch (BusinessRuleViolationException ex)");
        sb.AppendLine("        {");
        sb.AppendLine("            return Json(ex.ToErrorEnvelope(context), StatusCodes.Status422UnprocessableEntity);");
        sb.AppendLine("        }");
        sb.AppendLine("        catch (CrudConflictException ex)");
        sb.AppendLine("        {");
        sb.AppendLine("            var envelope = ErrorEnvelope.From(context, \"CRUD_CONFLICT\", ex.Message, retryable: true);");
        sb.AppendLine("            return Json(envelope, StatusCodes.Status409Conflict);");
        sb.AppendLine("        }");
        sb.AppendLine("    }");
        sb.AppendLine();
        sb.AppendLine("    public static bool TryParseRoute<T>(HttpContext http, string name, out T value)");
        sb.AppendLine("        where T : IParsable<T>");
        sb.AppendLine("    {");
        sb.AppendLine("        if (http.Request.RouteValues.TryGetValue(name, out var raw) &&");
        sb.AppendLine("            raw is not null &&");
        sb.AppendLine("            T.TryParse(raw.ToString(), CultureInfo.InvariantCulture, out var parsed) &&");
        sb.AppendLine("            parsed is not null)");
        sb.AppendLine("        {");
        sb.AppendLine("            value = parsed;");
        sb.AppendLine("            return true;");
        sb.AppendLine("        }");
        sb.AppendLine("        value = default!;");
        sb.AppendLine("        return false;");
        sb.AppendLine("    }");
        sb.AppendLine();
        sb.AppendLine("    public static int? GetQueryInt32(HttpContext http, string name) =>");
        sb.AppendLine("        http.Request.Query.TryGetValue(name, out var raw) &&");
        sb.AppendLine("        int.TryParse(raw.ToString(), NumberStyles.Integer, CultureInfo.InvariantCulture, out var parsed)");
        sb.AppendLine("            ? parsed");
        sb.AppendLine("            : null;");
        sb.AppendLine();
        sb.AppendLine("    public static string? GetQueryString(HttpContext http, string name) =>");
        sb.AppendLine("        http.Request.Query.TryGetValue(name, out var raw) && !string.IsNullOrWhiteSpace(raw)");
        sb.AppendLine("            ? raw.ToString()");
        sb.AppendLine("            : null;");
        sb.AppendLine();
        sb.AppendLine("    public static IEndpointConventionBuilder Produces<");
        sb.AppendLine("        [DynamicallyAccessedMembers(DynamicallyAccessedMemberTypes.PublicProperties | DynamicallyAccessedMemberTypes.PublicConstructors)] TResponse>(");
        sb.AppendLine("        this IEndpointConventionBuilder builder,");
        sb.AppendLine("        int statusCode = StatusCodes.Status200OK)");
        sb.AppendLine("    {");
        sb.AppendLine("        builder.WithMetadata(new ProducesResponseTypeMetadata(statusCode, typeof(TResponse), new[] { \"application/json\" }));");
        sb.AppendLine("        return builder;");
        sb.AppendLine("    }");
        sb.AppendLine();
        sb.AppendLine("    public static IEndpointConventionBuilder Produces(");
        sb.AppendLine("        this IEndpointConventionBuilder builder,");
        sb.AppendLine("        int statusCode)");
        sb.AppendLine("    {");
        sb.AppendLine("        builder.WithMetadata(new ProducesResponseTypeMetadata(statusCode, typeof(void), Array.Empty<string>()));");
        sb.AppendLine("        return builder;");
        sb.AppendLine("    }");
        sb.AppendLine();
        sb.AppendLine("    public static IEndpointConventionBuilder Accepts<");
        sb.AppendLine("        [DynamicallyAccessedMembers(DynamicallyAccessedMemberTypes.PublicProperties | DynamicallyAccessedMemberTypes.PublicConstructors)] TRequest>(");
        sb.AppendLine("        this IEndpointConventionBuilder builder,");
        sb.AppendLine("        string contentType = \"application/json\")");
        sb.AppendLine("    {");
        sb.AppendLine("        builder.WithMetadata(new AcceptsMetadata(new[] { contentType }, typeof(TRequest)));");
        sb.AppendLine("        return builder;");
        sb.AppendLine("    }");
        sb.AppendLine();
        sb.AppendLine("    public static void PreserveSchemaType<");
        sb.AppendLine("        [DynamicallyAccessedMembers(DynamicallyAccessedMemberTypes.PublicProperties | DynamicallyAccessedMemberTypes.PublicConstructors)] T>()");
        sb.AppendLine("    {");
        sb.AppendLine("    }");
        sb.AppendLine();
        sb.AppendLine("    public static class Results");
        sb.AppendLine("    {");
        sb.AppendLine("        public static IResult Created(HttpContext http, string location, byte[] utf8Json)");
        sb.AppendLine("        {");
        sb.AppendLine("            http.Response.Headers.Location = location;");
        sb.AppendLine("            return Microsoft.AspNetCore.Http.Results.Text((ReadOnlySpan<byte>)utf8Json, \"application/json; charset=utf-8\", statusCode: StatusCodes.Status201Created);");
        sb.AppendLine("        }");
        sb.AppendLine();
        sb.AppendLine("        public static IResult Ok(byte[] utf8Json) =>");
        sb.AppendLine("            Microsoft.AspNetCore.Http.Results.Text((ReadOnlySpan<byte>)utf8Json, \"application/json; charset=utf-8\", statusCode: StatusCodes.Status200OK);");
        sb.AppendLine("    }");
        sb.AppendLine();
        sb.AppendLine("    // Utf8JsonWriter keeps error responses AOT/trim safe (no reflection-based serialization).");
        sb.AppendLine("    private static IResult Json(ErrorEnvelope envelope, int statusCode)");
        sb.AppendLine("    {");
        sb.AppendLine("        using var buffer = new MemoryStream();");
        sb.AppendLine("        using (var writer = new Utf8JsonWriter(buffer))");
        sb.AppendLine("        {");
        sb.AppendLine("            writer.WriteStartObject();");
        sb.AppendLine("            writer.WriteString(\"error_code\", envelope.ErrorCode);");
        sb.AppendLine("            writer.WriteString(\"message\", envelope.Message);");
        sb.AppendLine("            writer.WriteString(\"correlation_id\", envelope.CorrelationId);");
        sb.AppendLine("            writer.WriteString(\"tenant_id\", envelope.TenantId);");
        sb.AppendLine("            writer.WriteBoolean(\"retryable\", envelope.Retryable);");
        sb.AppendLine("            if (envelope.Details is { } details)");
        sb.AppendLine("            {");
        sb.AppendLine("                writer.WritePropertyName(\"details\");");
        sb.AppendLine("                details.WriteTo(writer);");
        sb.AppendLine("            }");
        sb.AppendLine("            writer.WriteEndObject();");
        sb.AppendLine("        }");
        sb.AppendLine("        var json = System.Text.Encoding.UTF8.GetString(buffer.ToArray());");
        sb.AppendLine("        return Microsoft.AspNetCore.Http.Results.Text(json, \"application/json; charset=utf-8\", statusCode: statusCode);");
        sb.AppendLine("    }");
        sb.AppendLine("}");
        return sb.ToString();
    }

    public static readonly CrudOperationKind[] Operations =
    {
        CrudOperationKind.Create,
        CrudOperationKind.Update,
        CrudOperationKind.Delete,
        CrudOperationKind.GetById,
        CrudOperationKind.GetList
    };

    private static void EmitRequestRecord(StringBuilder sb, CrudEntityModel entity, CrudOperationKind operation)
    {
        var request = entity.RequestName(operation);
        var mediator = entity.Dispatcher == DispatcherKind.MediatorNet;
        var response = ResponseType(entity, operation);

        sb.AppendLine(mediator
            ? $"/// <summary>Generated {operation} message for {entity.Name}.</summary>"
            : $"/// <summary>Generated {operation} request for {entity.Name}.</summary>");
        sb.AppendLine(mediator
            ? $"public sealed record {request} : MediatorNet.IRequest<{response}>"
            : $"public sealed record {request}");
        sb.AppendLine("{");

        switch (operation)
        {
            case CrudOperationKind.Create:
                EmitDataProperties(sb, entity);
                break;
            case CrudOperationKind.Update:
                sb.AppendLine($"    [JsonIgnore] public {entity.IdType} Id {{ get; init; }} = default!;");
                EmitDataProperties(sb, entity);
                break;
            case CrudOperationKind.Delete:
            case CrudOperationKind.GetById:
                sb.AppendLine($"    public {entity.IdType} Id {{ get; init; }} = default!;");
                break;
            case CrudOperationKind.GetList:
                sb.AppendLine("    public int Page { get; init; } = 1;");
                sb.AppendLine("    public int PageSize { get; init; } = CrudQuery.DefaultPageSize;");
                sb.AppendLine("    public string? Sort { get; init; }");
                break;
        }

        if (mediator)
        {
            sb.AppendLine("    [JsonIgnore] public RequestContext Context { get; init; } = default!;");
        }

        if (operation is CrudOperationKind.Create or CrudOperationKind.Update)
        {
            sb.AppendLine();
            sb.AppendLine($"    public static async ValueTask<{request}?> ReadJsonAsync(Stream body, CancellationToken cancellationToken = default)");
            sb.AppendLine("    {");
            sb.AppendLine($"        using var doc = await global::{GeneratedNamespace}.GeneratedCrudJsonSupport.TryParseJsonAsync(body, cancellationToken).ConfigureAwait(false);");
            sb.AppendLine("        if (doc is null || doc.RootElement.ValueKind != JsonValueKind.Object) return null;");
            sb.AppendLine("        var root = doc.RootElement;");
            sb.AppendLine($"        return new {request}");
            sb.AppendLine("        {");
            foreach (var property in entity.Properties.Where(p => p.Writable && !IsId(entity, p)))
            {
                sb.AppendLine($"            {property.Name} = global::{GeneratedNamespace}.GeneratedCrudJsonSupport.{ReadMethod(property)}(root, \"{property.CamelName}\", \"{property.Name}\"),");
            }
            sb.AppendLine("        };");
            sb.AppendLine("    }");
        }

        sb.AppendLine("}");
    }

    private static string ReadMethod(CrudPropertyModel property) => property.JsonKind switch
    {
        CrudPropertyJsonKind.String => "ReadString",
        CrudPropertyJsonKind.NullableString => "ReadNullableString",
        CrudPropertyJsonKind.Boolean => "ReadBoolean",
        CrudPropertyJsonKind.NullableBoolean => "ReadNullableBoolean",
        CrudPropertyJsonKind.Int32 => "ReadInt32",
        CrudPropertyJsonKind.NullableInt32 => "ReadNullableInt32",
        CrudPropertyJsonKind.Int64 => "ReadInt64",
        CrudPropertyJsonKind.NullableInt64 => "ReadNullableInt64",
        CrudPropertyJsonKind.Int16 => "ReadInt16",
        CrudPropertyJsonKind.NullableInt16 => "ReadNullableInt16",
        CrudPropertyJsonKind.Byte => "ReadByte",
        CrudPropertyJsonKind.NullableByte => "ReadNullableByte",
        CrudPropertyJsonKind.UInt32 => "ReadUInt32",
        CrudPropertyJsonKind.NullableUInt32 => "ReadNullableUInt32",
        CrudPropertyJsonKind.UInt64 => "ReadUInt64",
        CrudPropertyJsonKind.NullableUInt64 => "ReadNullableUInt64",
        CrudPropertyJsonKind.Decimal => "ReadDecimal",
        CrudPropertyJsonKind.NullableDecimal => "ReadNullableDecimal",
        CrudPropertyJsonKind.Double => "ReadDouble",
        CrudPropertyJsonKind.NullableDouble => "ReadNullableDouble",
        CrudPropertyJsonKind.Single => "ReadSingle",
        CrudPropertyJsonKind.NullableSingle => "ReadNullableSingle",
        CrudPropertyJsonKind.Guid => "ReadGuid",
        CrudPropertyJsonKind.NullableGuid => "ReadNullableGuid",
        CrudPropertyJsonKind.DateTime => "ReadDateTime",
        CrudPropertyJsonKind.NullableDateTime => "ReadNullableDateTime",
        CrudPropertyJsonKind.DateTimeOffset => "ReadDateTimeOffset",
        CrudPropertyJsonKind.NullableDateTimeOffset => "ReadNullableDateTimeOffset",
        CrudPropertyJsonKind.DateOnly => "ReadDateOnly",
        CrudPropertyJsonKind.NullableDateOnly => "ReadNullableDateOnly",
        CrudPropertyJsonKind.TimeOnly => "ReadTimeOnly",
        CrudPropertyJsonKind.NullableTimeOnly => "ReadNullableTimeOnly",
        CrudPropertyJsonKind.Enum => $"ReadEnum<{property.UnderlyingType}>",
        CrudPropertyJsonKind.NullableEnum => $"ReadNullableEnum<{property.UnderlyingType}>",
        CrudPropertyJsonKind.NullableParsable => $"ReadNullableParsable<{property.UnderlyingType}>",
        _ => $"ReadParsable<{property.UnderlyingType}>"
    };

    private static void EmitDataProperties(StringBuilder sb, CrudEntityModel entity)
    {
        foreach (var property in entity.Properties.Where(p => p.Writable && !IsId(entity, p)))
        {
            sb.AppendLine($"    public {property.Type} {property.Name} {{ get; init; }} = default!;");
        }
    }

    private static void EmitHandleAsync(StringBuilder sb, CrudEntityModel entity, CrudOperationKind operation,
        string response)
    {
        var request = entity.RequestName(operation);
        sb.AppendLine($"    public async ValueTask<{response}> HandleAsync({request} request, RequestContext context,");
        sb.AppendLine("        CancellationToken cancellationToken = default)");
        sb.AppendLine("    {");
        sb.AppendLine("        ArgumentNullException.ThrowIfNull(request);");
        sb.AppendLine("        ArgumentNullException.ThrowIfNull(context);");
        sb.AppendLine("        cancellationToken.ThrowIfCancellationRequested();");
        sb.AppendLine();

        switch (operation)
        {
            case CrudOperationKind.Create:
                sb.AppendLine($"        var entity = new {entity.EntityType}");
                sb.AppendLine("        {");
                foreach (var property in entity.Properties.Where(p => p.Writable && !IsId(entity, p)))
                {
                    sb.AppendLine($"            {property.Name} = request.{property.Name},");
                }
                sb.AppendLine("        };");
                sb.AppendLine("        await CrudRuleRunner.RunBeforeCreateAsync(_rules, entity, context, cancellationToken).ConfigureAwait(false);");
                sb.AppendLine("        var created = await _store.AddAsync(entity, context, cancellationToken).ConfigureAwait(false);");
                sb.AppendLine($"        return {entity.Name}CrudMapper.ToDto(created);");
                break;

            case CrudOperationKind.Update:
                sb.AppendLine("        var entity = await _store.GetByIdAsync(request.Id, context, cancellationToken).ConfigureAwait(false);");
                sb.AppendLine("        if (entity is null) return null;");
                foreach (var property in entity.Properties.Where(p => p.Writable && !IsId(entity, p)))
                {
                    sb.AppendLine($"        entity.{property.Name} = request.{property.Name};");
                }
                sb.AppendLine("        await CrudRuleRunner.RunBeforeUpdateAsync(_rules, entity, context, cancellationToken).ConfigureAwait(false);");
                sb.AppendLine("        var updated = await _store.UpdateAsync(entity, context, cancellationToken).ConfigureAwait(false);");
                sb.AppendLine($"        return updated is null ? null : {entity.Name}CrudMapper.ToDto(updated);");
                break;

            case CrudOperationKind.Delete:
                sb.AppendLine("        var entity = await _store.GetByIdAsync(request.Id, context, cancellationToken).ConfigureAwait(false);");
                sb.AppendLine("        if (entity is null) return false;");
                sb.AppendLine("        await CrudRuleRunner.RunBeforeDeleteAsync(_rules, entity, context, cancellationToken).ConfigureAwait(false);");
                sb.AppendLine("        return await _store.DeleteAsync(entity, context, cancellationToken).ConfigureAwait(false);");
                break;

            case CrudOperationKind.GetById:
                sb.AppendLine("        var entity = await _store.GetByIdAsync(request.Id, context, cancellationToken).ConfigureAwait(false);");
                sb.AppendLine($"        return entity is null ? null : {entity.Name}CrudMapper.ToDto(entity);");
                break;

            case CrudOperationKind.GetList:
                sb.AppendLine("        var page = await _store.GetListAsync(");
                sb.AppendLine("            new CrudQuery(request.Page, request.PageSize, request.Sort), context, cancellationToken).ConfigureAwait(false);");
                sb.AppendLine($"        var items = page.Items.Select({entity.Name}CrudMapper.ToDto).ToArray();");
                sb.AppendLine($"        return new PagedResult<{entity.Name}Dto>(items, page.Page, page.PageSize, page.TotalCount);");
                break;
        }

        sb.AppendLine("    }");
    }

    private static string ResponseType(CrudEntityModel entity, CrudOperationKind operation) => operation switch
    {
        CrudOperationKind.Delete => "bool",
        CrudOperationKind.GetList => $"PagedResult<{entity.Name}Dto>",
        CrudOperationKind.Update => $"{entity.Name}Dto?",
        CrudOperationKind.GetById => $"{entity.Name}Dto?",
        _ => $"{entity.Name}Dto"
    };

    private static string? RuleInterface(CrudOperationKind operation) => operation switch
    {
        CrudOperationKind.Create => "IBeforeCreateRule",
        CrudOperationKind.Update => "IBeforeUpdateRule",
        CrudOperationKind.Delete => "IBeforeDeleteRule",
        _ => null
    };

    private static string MapMethod(CrudOperationKind operation) => operation switch
    {
        CrudOperationKind.Create => "MapPost",
        CrudOperationKind.Update => "MapPut",
        CrudOperationKind.Delete => "MapDelete",
        _ => "MapGet"
    };

    private static string Pattern(CrudOperationKind operation) => operation switch
    {
        CrudOperationKind.Create => "",
        CrudOperationKind.GetList => "",
        _ => "/{id}"
    };

    private static string Variable(CrudEntityModel entity) =>
        char.ToLowerInvariant(entity.Name[0]) + entity.Name.Substring(1) + "Crud";

    private static bool IsId(CrudEntityModel entity, CrudPropertyModel property) =>
        string.Equals(property.Name, entity.IdPropertyName, System.StringComparison.Ordinal);

    private static StringBuilder Header()
    {
        var sb = new StringBuilder();
        sb.AppendLine("// <auto-generated />");
        sb.AppendLine("#nullable enable");
        sb.AppendLine("#pragma warning disable CS1591");
        return sb;
    }

    private static void OpenNamespace(StringBuilder sb, string featureNamespace)
    {
        sb.AppendLine($"namespace {featureNamespace};");
        sb.AppendLine();
    }

    private static void CloseNamespace(StringBuilder sb)
    {
    }
}
