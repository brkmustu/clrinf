using System.Reflection;
using System.Text.Json;
using ClrinfCS.Core;
using ClrinfCS.OpenApi;
using Microsoft.AspNetCore.Authorization;
using Microsoft.AspNetCore.Builder;
using Microsoft.AspNetCore.Http;
using Microsoft.AspNetCore.Mvc;
using Microsoft.AspNetCore.Mvc.Abstractions;
using Microsoft.AspNetCore.Mvc.Controllers;
using Microsoft.AspNetCore.Mvc.Infrastructure;
using Microsoft.AspNetCore.Mvc.ModelBinding;
using Microsoft.AspNetCore.Mvc.Routing;
using Microsoft.AspNetCore.Routing;
using Microsoft.Extensions.DependencyInjection;
using Xunit;

namespace ClrinfCS.Tests;

public class OpenApiTests
{
    [Authorize]
    public class ContractController : ControllerBase
    {
        [HttpPost]
        public ValueTask<ActionResult<CloudEvent>> Publish([FromBody] CloudEvent envelope, CancellationToken cancellationToken) =>
            ValueTask.FromResult<ActionResult<CloudEvent>>(envelope);

        [HttpGet, AllowAnonymous]
        public ErrorEnvelope Error() => ErrorEnvelope.From(new("t", "c", "r"), "NOT_FOUND", "Missing");
    }

    private sealed class Descriptors(params ActionDescriptor[] descriptors) : IActionDescriptorCollectionProvider
    {
        public ActionDescriptorCollection ActionDescriptors { get; } = new(descriptors, 1);
    }

    [Fact]
    public void DocumentsActualContractNamesRequiredFieldsAndAsyncResponses()
    {
        var method = typeof(ContractController).GetMethod(nameof(ContractController.Publish))!;
        var action = new ControllerActionDescriptor
        {
            ControllerName = "Contract", ActionName = "Publish", MethodInfo = method,
            ControllerTypeInfo = typeof(ContractController).GetTypeInfo(),
            AttributeRouteInfo = new AttributeRouteInfo { Template = "api/[controller]/{id:int}" },
            Parameters = method.GetParameters().Select(p => (ParameterDescriptor)new ControllerParameterDescriptor
            {
                Name = p.Name!, ParameterType = p.ParameterType, ParameterInfo = p,
                BindingInfo = new BindingInfo { BindingSource = p.ParameterType == typeof(CloudEvent) ? BindingSource.Body : null }
            }).ToList()
        };
        var errorMethod = typeof(ContractController).GetMethod(nameof(ContractController.Error))!;
        var error = new ControllerActionDescriptor
        {
            ControllerName = "Contract", ActionName = "Error", MethodInfo = errorMethod,
            ControllerTypeInfo = typeof(ContractController).GetTypeInfo(),
            AttributeRouteInfo = new AttributeRouteInfo { Template = "api/errors" }
        };
        using var doc = JsonDocument.Parse(new OpenApiGenerator(new Descriptors(action, error), new()).GenerateJson());
        var root = doc.RootElement;
        var operation = root.GetProperty("paths").GetProperty("/api/Contract/{id}").GetProperty("post");
        Assert.False(operation.TryGetProperty("parameters", out _));
        Assert.Equal("#/components/schemas/CloudEvent", operation.GetProperty("responses").GetProperty("200")
            .GetProperty("content").GetProperty("application/json").GetProperty("schema").GetProperty("$ref").GetString());
        Assert.True(operation.TryGetProperty("security", out _));
        Assert.False(root.GetProperty("paths").GetProperty("/api/errors").GetProperty("get").TryGetProperty("security", out _));
        var components = root.GetProperty("components");
        Assert.Equal("bearer", components.GetProperty("securitySchemes").GetProperty("Bearer").GetProperty("scheme").GetString());
        var schemas = components.GetProperty("schemas");
        var cloudEvent = schemas.GetProperty("CloudEvent");
        Assert.Equal("object", cloudEvent.GetProperty("properties").GetProperty("data").GetProperty("type").GetString());
        Assert.True(cloudEvent.GetProperty("properties").TryGetProperty("tenantid", out _));
        Assert.True(cloudEvent.GetProperty("properties").TryGetProperty("is_error", out _));
        var required = cloudEvent.GetProperty("required").EnumerateArray().Select(item => item.GetString()).ToArray();
        Assert.Contains("specversion", required);
        Assert.Contains("tenantid", required);
        Assert.DoesNotContain("is_error", required);
        Assert.True(schemas.GetProperty("ErrorEnvelope").GetProperty("properties").TryGetProperty("error_code", out _));
    }

    [Fact]
    public void DocumentationUiIsEmbedded()
    {
        using var stream = typeof(OpenApiGenerator).Assembly.GetManifestResourceStream("ClrinfCS.OpenApi.api-docs.html");
        Assert.NotNull(stream);
        using var reader = new StreamReader(stream);
        Assert.Contains("<html", reader.ReadToEnd(), StringComparison.OrdinalIgnoreCase);
    }

    private sealed record CategoryDto
    {
        public int Id { get; init; }
        public string Name { get; init; } = string.Empty;
    }

    private sealed record CreateCategoryRequest
    {
        public string Name { get; init; } = string.Empty;
    }

    [Fact]
    public void DocumentsMinimalApiCrudEndpointsWithMetadataParametersAndSchemas()
    {
        var builder = Microsoft.AspNetCore.Builder.WebApplication.CreateBuilder();
        builder.Services.AddClrinfOpenApi(o =>
        {
            o.Title = "AotCrud";
            o.Version = "1.0.0";
        });

        using var app = builder.Build();
        var group = app.MapGroup("/api/categories");

        var post = group.MapPost("", (Microsoft.AspNetCore.Http.RequestDelegate)(static _ => Task.CompletedTask));
        post.WithName("CreateCategory");
        post.WithTags("Categories");
        post.WithMetadata(new Microsoft.AspNetCore.Http.Metadata.AcceptsMetadata(new[] { "application/json" }, typeof(CreateCategoryRequest)));
        post.WithMetadata(new Microsoft.AspNetCore.Http.ProducesResponseTypeMetadata(201, typeof(CategoryDto), new[] { "application/json" }));
        post.WithMetadata(new Microsoft.AspNetCore.Http.ProducesResponseTypeMetadata(422, typeof(ErrorEnvelope), new[] { "application/json" }));

        var list = group.MapGet("", (Microsoft.AspNetCore.Http.RequestDelegate)(static _ => Task.CompletedTask));
        list.WithName("GetListCategory");
        list.WithTags("Categories");
        list.WithMetadata(new CrudEndpointParameterMetadata("page", "query", typeof(int?), false));
        list.WithMetadata(new CrudEndpointParameterMetadata("pageSize", "query", typeof(int?), false));
        list.WithMetadata(new CrudEndpointParameterMetadata("sort", "query", typeof(string), false));
        list.WithMetadata(new Microsoft.AspNetCore.Http.ProducesResponseTypeMetadata(200, typeof(PagedResult<CategoryDto>), new[] { "application/json" }));

        var getById = group.MapGet("/{id}", (Microsoft.AspNetCore.Http.RequestDelegate)(static _ => Task.CompletedTask));
        getById.WithName("GetByIdCategory");
        getById.WithTags("Categories");
        getById.WithMetadata(new CrudEndpointParameterMetadata("id", "path", typeof(int), true));
        getById.WithMetadata(new Microsoft.AspNetCore.Http.ProducesResponseTypeMetadata(200, typeof(CategoryDto), new[] { "application/json" }));
        getById.WithMetadata(new Microsoft.AspNetCore.Http.ProducesResponseTypeMetadata(404, typeof(void), Array.Empty<string>()));

        app.MapClrinfOpenApi();

        var generator = app.Services.GetRequiredService<OpenApiGenerator>();
        using var doc = JsonDocument.Parse(generator.GenerateJson());
        var root = doc.RootElement;
        var paths = root.GetProperty("paths");

        Assert.False(paths.TryGetProperty("/openapi/v1.json", out _));
        Assert.False(paths.TryGetProperty("/swagger", out _));

        var categoriesPath = paths.GetProperty("/api/categories");
        var postOp = categoriesPath.GetProperty("post");
        Assert.Equal("CreateCategory", postOp.GetProperty("operationId").GetString());
        Assert.Equal("Categories", postOp.GetProperty("tags")[0].GetString());
        Assert.Equal("#/components/schemas/CreateCategoryRequest",
            postOp.GetProperty("requestBody").GetProperty("content").GetProperty("application/json")
                .GetProperty("schema").GetProperty("$ref").GetString());
        Assert.Equal("#/components/schemas/CategoryDto",
            postOp.GetProperty("responses").GetProperty("201").GetProperty("content").GetProperty("application/json")
                .GetProperty("schema").GetProperty("$ref").GetString());

        var listOp = categoriesPath.GetProperty("get");
        Assert.Equal("GetListCategory", listOp.GetProperty("operationId").GetString());
        Assert.Equal(3, listOp.GetProperty("parameters").GetArrayLength());
        Assert.Equal("#/components/schemas/PagedResultOfCategoryDto",
            listOp.GetProperty("responses").GetProperty("200").GetProperty("content").GetProperty("application/json")
                .GetProperty("schema").GetProperty("$ref").GetString());

        var byIdOp = paths.GetProperty("/api/categories/{id}").GetProperty("get");
        var idParam = byIdOp.GetProperty("parameters")[0];
        Assert.Equal("id", idParam.GetProperty("name").GetString());
        Assert.Equal("path", idParam.GetProperty("in").GetString());
        Assert.True(idParam.GetProperty("required").GetBoolean());
        Assert.Equal("integer", idParam.GetProperty("schema").GetProperty("type").GetString());

        var schemas = root.GetProperty("components").GetProperty("schemas");
        Assert.True(schemas.TryGetProperty("CategoryDto", out _));
        Assert.True(schemas.TryGetProperty("CreateCategoryRequest", out _));
        var pagedSchema = schemas.GetProperty("PagedResultOfCategoryDto");
        Assert.Equal("array", pagedSchema.GetProperty("properties").GetProperty("items").GetProperty("type").GetString());
    }
}
