using System.Reflection;
using System.Text.Json;
using ClrinfCS.Core;
using ClrinfCS.OpenApi;
using Microsoft.AspNetCore.Authorization;
using Microsoft.AspNetCore.Mvc;
using Microsoft.AspNetCore.Mvc.Abstractions;
using Microsoft.AspNetCore.Mvc.Controllers;
using Microsoft.AspNetCore.Mvc.Infrastructure;
using Microsoft.AspNetCore.Mvc.ModelBinding;
using Microsoft.AspNetCore.Mvc.Routing;
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
}
