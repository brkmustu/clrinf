using System.Collections;
using System.Diagnostics.CodeAnalysis;
using System.Globalization;
using System.Reflection;
using System.Runtime.CompilerServices;
using System.Text;
using System.Text.Json;
using System.Text.Json.Serialization;
using Microsoft.AspNetCore.Authorization;
using Microsoft.AspNetCore.Http.Metadata;
using Microsoft.AspNetCore.Mvc;
using Microsoft.AspNetCore.Mvc.ApiExplorer;
using Microsoft.AspNetCore.Mvc.Controllers;
using Microsoft.AspNetCore.Mvc.Infrastructure;
using Microsoft.AspNetCore.Mvc.Routing;
using Microsoft.AspNetCore.Routing;

namespace ClrinfCS.OpenApi;

public class OpenApiGenerator
{
    private static readonly HashSet<string> IgnoredPaths = new(StringComparer.OrdinalIgnoreCase)
    {
        "/openapi/v1.json",
        "/swagger",
        "/api-docs",
        "/dsi-arch/api"
    };

    private readonly IActionDescriptorCollectionProvider? _actionDescriptorCollectionProvider;
    private readonly IEnumerable<EndpointDataSource>? _endpointDataSources;
    private readonly OpenApiOptions _options;

    public OpenApiGenerator(IActionDescriptorCollectionProvider actionDescriptorCollectionProvider, OpenApiOptions options)
        : this(actionDescriptorCollectionProvider, null, options)
    {
    }

    public OpenApiGenerator(IEnumerable<EndpointDataSource> endpointDataSources, OpenApiOptions options)
        : this(null, endpointDataSources, options)
    {
    }

    public OpenApiGenerator(
        IActionDescriptorCollectionProvider? actionDescriptorCollectionProvider,
        IEnumerable<EndpointDataSource>? endpointDataSources,
        OpenApiOptions options)
    {
        _actionDescriptorCollectionProvider = actionDescriptorCollectionProvider;
        _endpointDataSources = endpointDataSources;
        _options = options;
    }

    public string GenerateJson()
    {
        var doc = GenerateDocument();
        using var buffer = new MemoryStream();
        using (var writer = new Utf8JsonWriter(buffer, new JsonWriterOptions { Indented = true }))
        {
            WriteJsonValue(writer, doc);
        }

        return Encoding.UTF8.GetString(buffer.ToArray());
    }

    public Dictionary<string, object?> GenerateDocument()
    {
        var schemas = new Dictionary<string, object>();
        var paths = new Dictionary<string, Dictionary<string, object>>();

        if (_actionDescriptorCollectionProvider is not null)
        {
            var actions = _actionDescriptorCollectionProvider.ActionDescriptors.Items
                .OfType<ControllerActionDescriptor>();

            foreach (var action in actions)
            {
                var controllerName = action.ControllerName;
                var routeTemplate = action.AttributeRouteInfo?.Template ?? $"api/{controllerName}";

                var pathKey = "/" + routeTemplate
                    .Replace("[controller]", controllerName, StringComparison.OrdinalIgnoreCase)
                    .Replace("[action]", action.ActionName, StringComparison.OrdinalIgnoreCase)
                    .TrimStart('/');

                pathKey = System.Text.RegularExpressions.Regex.Replace(pathKey, @"\{([a-zA-Z0-9_]+):[^}]+\}", "{$1}");

                var httpMethods = GetHttpMethods(action);

                foreach (var httpMethod in httpMethods)
                {
                    if (!paths.TryGetValue(pathKey, out var pathItem))
                    {
                        pathItem = new Dictionary<string, object>();
                        paths[pathKey] = pathItem;
                    }

                    var operation = BuildOperation(action, controllerName, schemas);
                    pathItem[httpMethod.ToLowerInvariant()] = operation;
                }
            }
        }

        if (_endpointDataSources is not null)
        {
            foreach (var endpoint in _endpointDataSources.SelectMany(static ds => ds.Endpoints).OfType<RouteEndpoint>())
            {
                if (endpoint.Metadata.GetMetadata<ControllerActionDescriptor>() is not null)
                {
                    continue;
                }

                if (endpoint.Metadata.GetMetadata<IExcludeFromDescriptionMetadata>()?.ExcludeFromDescription == true)
                {
                    continue;
                }

                var rawPattern = endpoint.RoutePattern.RawText ?? "/";
                var pathKey = "/" + rawPattern.TrimStart('/');
                if (pathKey.Length > 1)
                {
                    pathKey = pathKey.TrimEnd('/');
                }

                pathKey = System.Text.RegularExpressions.Regex.Replace(pathKey, @"\{([a-zA-Z0-9_]+):[^}]+\}", "{$1}");
                if (IgnoredPaths.Contains(pathKey))
                {
                    continue;
                }

                var httpMethods = endpoint.Metadata.GetMetadata<IHttpMethodMetadata>()?.HttpMethods;
                if (httpMethods is null || httpMethods.Count == 0)
                {
                    continue;
                }

                foreach (var httpMethod in httpMethods)
                {
                    if (!paths.TryGetValue(pathKey, out var pathItem))
                    {
                        pathItem = new Dictionary<string, object>();
                        paths[pathKey] = pathItem;
                    }

                    var methodKey = httpMethod.ToLowerInvariant();
                    if (!pathItem.ContainsKey(methodKey))
                    {
                        pathItem[methodKey] = BuildEndpointOperation(endpoint, httpMethod, pathKey, schemas);
                    }
                }
            }
        }

        var root = new Dictionary<string, object?>
        {
            ["openapi"] = "3.0.1",
            ["info"] = new Dictionary<string, object>
            {
                ["title"] = _options.Title,
                ["version"] = _options.Version,
                ["description"] = _options.Description
            },
            ["paths"] = paths,
            ["components"] = new Dictionary<string, object>
            {
                ["schemas"] = schemas,
                ["securitySchemes"] = new Dictionary<string, object>
                {
                    ["Bearer"] = new Dictionary<string, object>
                    {
                        ["type"] = "http", ["scheme"] = "bearer", ["bearerFormat"] = "JWT"
                    }
                }
            }
        };

        return root;
    }

    private Dictionary<string, object> BuildEndpointOperation(
        RouteEndpoint endpoint,
        string httpMethod,
        string pathKey,
        Dictionary<string, object> schemas)
    {
        var tagsMetadata = endpoint.Metadata.GetMetadata<ITagsMetadata>();
        var tags = tagsMetadata is { Tags.Count: > 0 }
            ? tagsMetadata.Tags.ToArray()
            : new[] { InferTag(pathKey) };

        var endpointName = endpoint.Metadata.GetMetadata<IEndpointNameMetadata>()?.EndpointName;
        var operationId = !string.IsNullOrWhiteSpace(endpointName)
            ? endpointName!
            : $"{tags[0]}_{httpMethod.ToUpperInvariant()}_{pathKey.Trim('/').Replace('/', '_').Replace("{", "").Replace("}", "")}";
        var summary = endpointName ?? $"{httpMethod.ToUpperInvariant()} {pathKey}";

        var isAnonymous = endpoint.Metadata.GetMetadata<IAllowAnonymous>() is not null;
        var isAuthorize = !isAnonymous && endpoint.Metadata.GetMetadata<IAuthorizeData>() is not null;

        var parameters = new List<Dictionary<string, object>>();
        var seenParams = new HashSet<string>(StringComparer.OrdinalIgnoreCase);

        foreach (var meta in endpoint.Metadata)
        {
            if (meta is ITuple { Length: 4 } tuple &&
                tuple[0] is string paramName &&
                tuple[1] is string location &&
                tuple[2] is Type paramType &&
                tuple[3] is bool required &&
                seenParams.Add(paramName))
            {
                parameters.Add(new Dictionary<string, object>
                {
                    ["name"] = paramName,
                    ["in"] = location,
                    ["required"] = location == "path" || (required && !IsNullable(paramType)),
                    ["schema"] = GetOrCreateSchema(paramType, schemas)
                });
            }
        }

        foreach (var routeParam in endpoint.RoutePattern.Parameters)
        {
            if (seenParams.Add(routeParam.Name))
            {
                parameters.Add(new Dictionary<string, object>
                {
                    ["name"] = routeParam.Name,
                    ["in"] = "path",
                    ["required"] = true,
                    ["schema"] = new Dictionary<string, object> { ["type"] = "string" }
                });
            }
        }

        Dictionary<string, object>? requestBody = null;
        var accepts = endpoint.Metadata.GetOrderedMetadata<IAcceptsMetadata>()
            .FirstOrDefault(static m => m.RequestType is not null);
        if (accepts?.RequestType is { } requestType)
        {
            var contentType = accepts.ContentTypes.Count > 0 ? accepts.ContentTypes[0] : "application/json";
            requestBody = new Dictionary<string, object>
            {
                ["required"] = !accepts.IsOptional,
                ["content"] = new Dictionary<string, object>
                {
                    [contentType] = new Dictionary<string, object>
                    {
                        ["schema"] = GetOrCreateSchema(requestType, schemas)
                    }
                }
            };
        }

        var responses = new Dictionary<string, object>();
        var producesMetadata = endpoint.Metadata.GetOrderedMetadata<IProducesResponseTypeMetadata>();
        foreach (var produces in producesMetadata)
        {
            var statusCode = produces.StatusCode == 0 ? 200 : produces.StatusCode;
            var statusKey = statusCode.ToString(CultureInfo.InvariantCulture);
            var description = DescribeStatus(statusCode);

            if (produces.Type is null || produces.Type == typeof(void))
            {
                responses[statusKey] = new Dictionary<string, object>
                {
                    ["description"] = description
                };
            }
            else
            {
                var returnSchema = GetOrCreateSchema(produces.Type, schemas);
                responses[statusKey] = new Dictionary<string, object>
                {
                    ["description"] = description,
                    ["content"] = new Dictionary<string, object>
                    {
                        ["application/json"] = new Dictionary<string, object>
                        {
                            ["schema"] = returnSchema
                        }
                    }
                };
            }
        }

        var responseProviders = endpoint.Metadata.GetOrderedMetadata<IApiResponseMetadataProvider>();
        foreach (var provider in responseProviders)
        {
            var statusCode = provider.StatusCode == 0 ? 200 : provider.StatusCode;
            var statusKey = statusCode.ToString(CultureInfo.InvariantCulture);
            if (responses.ContainsKey(statusKey)) continue;
            var description = DescribeStatus(statusCode);

            if (provider.Type is null || provider.Type == typeof(void))
            {
                responses[statusKey] = new Dictionary<string, object>
                {
                    ["description"] = description
                };
            }
            else
            {
                var returnSchema = GetOrCreateSchema(provider.Type, schemas);
                responses[statusKey] = new Dictionary<string, object>
                {
                    ["description"] = description,
                    ["content"] = new Dictionary<string, object>
                    {
                        ["application/json"] = new Dictionary<string, object>
                        {
                            ["schema"] = returnSchema
                        }
                    }
                };
            }
        }

        if (responses.Count == 0)
        {
            responses["200"] = new Dictionary<string, object>
            {
                ["description"] = "Success"
            };
        }

        var operation = new Dictionary<string, object>
        {
            ["tags"] = tags,
            ["summary"] = summary,
            ["operationId"] = operationId,
            ["responses"] = responses
        };

        if (parameters.Count > 0)
        {
            operation["parameters"] = parameters;
        }

        if (requestBody is not null)
        {
            operation["requestBody"] = requestBody;
        }

        if (isAuthorize)
        {
            operation["security"] = new[]
            {
                new Dictionary<string, string[]>
                {
                    ["Bearer"] = Array.Empty<string>()
                }
            };
        }

        return operation;
    }

    private static string InferTag(string pathKey)
    {
        var segments = pathKey.Split('/', StringSplitOptions.RemoveEmptyEntries);
        foreach (var segment in segments)
        {
            if (!segment.Equals("api", StringComparison.OrdinalIgnoreCase) && !segment.StartsWith('{'))
            {
                return char.ToUpperInvariant(segment[0]) + segment[1..];
            }
        }

        return "Default";
    }

    private static string DescribeStatus(int statusCode) => statusCode switch
    {
        200 => "Success",
        201 => "Created",
        204 => "No Content",
        400 => "Bad Request",
        401 => "Unauthorized",
        403 => "Forbidden",
        404 => "Not Found",
        409 => "Conflict",
        422 => "Unprocessable Entity",
        _ => "Response"
    };

    private static List<string> GetHttpMethods(ControllerActionDescriptor action)
    {
        var methods = new List<string>();

        var httpMethodAttributes = action.MethodInfo.GetCustomAttributes()
            .OfType<HttpMethodAttribute>();

        foreach (var attr in httpMethodAttributes)
        {
            methods.AddRange(attr.HttpMethods);
        }

        if (methods.Count == 0)
        {
            methods.Add("GET");
        }

        return methods.Distinct(StringComparer.OrdinalIgnoreCase).ToList();
    }

    private Dictionary<string, object> BuildOperation(ControllerActionDescriptor action, string controllerName, Dictionary<string, object> schemas)
    {
        var operationId = $"{controllerName}_{action.ActionName}";
        var summary = action.ActionName;

        var parameters = new List<Dictionary<string, object>>();
        Dictionary<string, object>? requestBody = null;

        var isAnonymous = action.MethodInfo.GetCustomAttribute<AllowAnonymousAttribute>() != null ||
                          action.ControllerTypeInfo.GetCustomAttribute<AllowAnonymousAttribute>() != null;
        var isAuthorize = !isAnonymous && (action.MethodInfo.GetCustomAttribute<AuthorizeAttribute>() != null ||
                          action.ControllerTypeInfo.GetCustomAttribute<AuthorizeAttribute>() != null);

        foreach (var param in action.Parameters)
        {
            var paramType = param.ParameterType;
            var bindingSource = param.BindingInfo?.BindingSource;
            if (paramType == typeof(CancellationToken) ||
                bindingSource == Microsoft.AspNetCore.Mvc.ModelBinding.BindingSource.Services)
                continue;

            if (bindingSource == Microsoft.AspNetCore.Mvc.ModelBinding.BindingSource.Body ||
                param.ParameterType.GetCustomAttribute<FromBodyAttribute>() != null ||
                (param is ControllerParameterDescriptor ctrlParam && ctrlParam.ParameterInfo.GetCustomAttribute<FromBodyAttribute>() != null))
            {
                var schemaRef = GetOrCreateSchema(paramType, schemas);
                requestBody = new Dictionary<string, object>
                {
                    ["required"] = true,
                    ["content"] = new Dictionary<string, object>
                    {
                        ["application/json"] = new Dictionary<string, object>
                        {
                            ["schema"] = schemaRef
                        }
                    }
                };
            }
            else
            {
                var inLocation = "query";
                if (bindingSource == Microsoft.AspNetCore.Mvc.ModelBinding.BindingSource.Path ||
                    action.AttributeRouteInfo?.Template?.Contains($"{{{param.Name}", StringComparison.OrdinalIgnoreCase) == true)
                {
                    inLocation = "path";
                }
                else if (bindingSource == Microsoft.AspNetCore.Mvc.ModelBinding.BindingSource.Header)
                {
                    inLocation = "header";
                }

                var paramSchema = GetOrCreateSchema(paramType, schemas);

                parameters.Add(new Dictionary<string, object>
                {
                    ["name"] = param.Name,
                    ["in"] = inLocation,
                    ["required"] = inLocation == "path" || !IsNullable(paramType),
                    ["schema"] = paramSchema
                });
            }
        }

        var returnType = action.MethodInfo.ReturnType;
        returnType = UnwrapTaskAndActionResult(returnType);

        var responses = new Dictionary<string, object>();

        if (returnType == typeof(void))
        {
            responses["204"] = new Dictionary<string, object>
            {
                ["description"] = "No Content"
            };
        }
        else
        {
            var returnSchema = GetOrCreateSchema(returnType, schemas);
            responses["200"] = new Dictionary<string, object>
            {
                ["description"] = "Success",
                ["content"] = new Dictionary<string, object>
                {
                    ["application/json"] = new Dictionary<string, object>
                    {
                        ["schema"] = returnSchema
                    }
                }
            };
        }

        var operation = new Dictionary<string, object>
        {
            ["tags"] = new[] { controllerName },
            ["summary"] = summary,
            ["operationId"] = operationId,
            ["responses"] = responses
        };

        if (parameters.Count > 0)
        {
            operation["parameters"] = parameters;
        }

        if (requestBody != null)
        {
            operation["requestBody"] = requestBody;
        }

        if (isAuthorize)
        {
            operation["security"] = new[]
            {
                new Dictionary<string, string[]>
                {
                    ["Bearer"] = Array.Empty<string>()
                }
            };
        }

        return operation;
    }

    private static Type UnwrapTaskAndActionResult(Type type)
    {
        if (type.IsGenericType && (type.GetGenericTypeDefinition() == typeof(Task<>) ||
                                  type.GetGenericTypeDefinition() == typeof(ValueTask<>)))
        {
            type = type.GetGenericArguments()[0];
        }
        else if (type == typeof(Task) || type == typeof(ValueTask))
        {
            return typeof(void);
        }

        if (type.IsGenericType && type.GetGenericTypeDefinition() == typeof(ActionResult<>))
        {
            type = type.GetGenericArguments()[0];
        }

        return type;
    }

    private static bool IsNullable(Type type)
    {
        if (!type.IsValueType) return true;
        return Nullable.GetUnderlyingType(type) != null;
    }

    [UnconditionalSuppressMessage("Trimming", "IL2070",
        Justification = "Endpoint contract types are preserved via DynamicallyAccessedMembers on Produces/Accepts metadata helpers.")]
    private Dictionary<string, object> GetOrCreateSchema(Type type, Dictionary<string, object> schemas)
    {
        type = Nullable.GetUnderlyingType(type) ?? type;

        if (type == typeof(JsonElement) || type == typeof(JsonDocument))
            return new Dictionary<string, object> { ["type"] = "object", ["additionalProperties"] = true };

        if (type == typeof(string) || type == typeof(char))
            return new Dictionary<string, object> { ["type"] = "string" };

        if (type == typeof(Guid))
            return new Dictionary<string, object> { ["type"] = "string", ["format"] = "uuid" };

        if (type == typeof(DateTime) || type == typeof(DateTimeOffset))
            return new Dictionary<string, object> { ["type"] = "string", ["format"] = "date-time" };

        if (type == typeof(int) || type == typeof(long) || type == typeof(short) || type == typeof(byte))
            return new Dictionary<string, object> { ["type"] = "integer" };

        if (type == typeof(double) || type == typeof(float) || type == typeof(decimal))
            return new Dictionary<string, object> { ["type"] = "number" };

        if (type == typeof(bool))
            return new Dictionary<string, object> { ["type"] = "boolean" };

        if (type.IsEnum)
        {
            var enumValues = Enum.GetNames(type);
            return new Dictionary<string, object>
            {
                ["type"] = "string",
                ["enum"] = enumValues
            };
        }

        if (type.IsArray)
        {
            var elementType = type.GetElementType()!;
            return new Dictionary<string, object>
            {
                ["type"] = "array",
                ["items"] = GetOrCreateSchema(elementType, schemas)
            };
        }

        if (type.IsGenericType && (type.GetGenericTypeDefinition() == typeof(List<>) ||
                                   type.GetGenericTypeDefinition() == typeof(IEnumerable<>) ||
                                   type.GetGenericTypeDefinition() == typeof(ICollection<>) ||
                                   type.GetGenericTypeDefinition() == typeof(IList<>) ||
                                   type.GetGenericTypeDefinition() == typeof(IReadOnlyList<>) ||
                                   type.GetGenericTypeDefinition() == typeof(IReadOnlyCollection<>)))
        {
            var elementType = type.GetGenericArguments()[0];
            return new Dictionary<string, object>
            {
                ["type"] = "array",
                ["items"] = GetOrCreateSchema(elementType, schemas)
            };
        }

        var schemaName = type.IsGenericType
            ? type.Name.Split('`')[0] + "Of" + string.Join("And", type.GetGenericArguments().Select(static a => a.Name))
            : type.Name;

        if (!schemas.ContainsKey(schemaName))
        {
            var properties = new Dictionary<string, object>();
            schemas[schemaName] = new Dictionary<string, object>
            {
                ["type"] = "object",
                ["properties"] = properties
            };

            var props = type.GetProperties(BindingFlags.Public | BindingFlags.Instance);
            var constructor = type.GetConstructors().FirstOrDefault(c => c.GetCustomAttribute<JsonConstructorAttribute>() != null);
            var requiredParameters = constructor?.GetParameters().Where(p => !p.HasDefaultValue)
                .Select(p => p.Name).ToHashSet(StringComparer.OrdinalIgnoreCase);
            var required = new List<string>();
            foreach (var prop in props)
            {
                if (prop.GetIndexParameters().Length != 0 ||
                    prop.GetCustomAttribute<JsonIgnoreAttribute>()?.Condition == JsonIgnoreCondition.Always)
                    continue;
                var propName = prop.GetCustomAttribute<JsonPropertyNameAttribute>()?.Name
                    ?? JsonNamingPolicy.CamelCase.ConvertName(prop.Name);
                properties[propName] = GetOrCreateSchema(prop.PropertyType, schemas);
                if (requiredParameters?.Contains(prop.Name) == true ||
                    prop.GetCustomAttribute<JsonRequiredAttribute>() != null)
                    required.Add(propName);
            }
            if (required.Count > 0)
                ((Dictionary<string, object>)schemas[schemaName])["required"] = required;
        }

        return new Dictionary<string, object>
        {
            ["$ref"] = $"#/components/schemas/{schemaName}"
        };
    }

    private static void WriteJsonValue(Utf8JsonWriter writer, object? value)
    {
        switch (value)
        {
            case null:
                writer.WriteNullValue();
                break;
            case string s:
                writer.WriteStringValue(s);
                break;
            case bool b:
                writer.WriteBooleanValue(b);
                break;
            case int i:
                writer.WriteNumberValue(i);
                break;
            case long l:
                writer.WriteNumberValue(l);
                break;
            case double d:
                writer.WriteNumberValue(d);
                break;
            case decimal m:
                writer.WriteNumberValue(m);
                break;
            case IDictionary<string, object?> dictNull:
                writer.WriteStartObject();
                foreach (var kv in dictNull)
                {
                    if (kv.Value is null) continue;
                    writer.WritePropertyName(kv.Key);
                    WriteJsonValue(writer, kv.Value);
                }
                writer.WriteEndObject();
                break;
            case IDictionary<string, Dictionary<string, object>> dictPaths:
                writer.WriteStartObject();
                foreach (var kv in dictPaths)
                {
                    writer.WritePropertyName(kv.Key);
                    WriteJsonValue(writer, kv.Value);
                }
                writer.WriteEndObject();
                break;
            case IDictionary<string, string[]> dictStrArr:
                writer.WriteStartObject();
                foreach (var kv in dictStrArr)
                {
                    writer.WritePropertyName(kv.Key);
                    WriteJsonValue(writer, kv.Value);
                }
                writer.WriteEndObject();
                break;
            case IEnumerable enumerable:
                writer.WriteStartArray();
                foreach (var item in enumerable)
                {
                    WriteJsonValue(writer, item);
                }
                writer.WriteEndArray();
                break;
            default:
                writer.WriteStringValue(value.ToString());
                break;
        }
    }
}
