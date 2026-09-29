using System.Reflection;
using System.Text.Json;
using System.Text.Json.Serialization;
using Microsoft.AspNetCore.Authorization;
using Microsoft.AspNetCore.Mvc;
using Microsoft.AspNetCore.Mvc.Controllers;
using Microsoft.AspNetCore.Mvc.Infrastructure;
using Microsoft.AspNetCore.Mvc.Routing;

namespace ClrinfCS.OpenApi;

public class OpenApiGenerator
{
    private readonly IActionDescriptorCollectionProvider _actionDescriptorCollectionProvider;
    private readonly OpenApiOptions _options;

    public OpenApiGenerator(IActionDescriptorCollectionProvider actionDescriptorCollectionProvider, OpenApiOptions options)
    {
        _actionDescriptorCollectionProvider = actionDescriptorCollectionProvider;
        _options = options;
    }

    public string GenerateJson()
    {
        var doc = GenerateDocument();
        var jsonOptions = new JsonSerializerOptions
        {
            PropertyNamingPolicy = JsonNamingPolicy.CamelCase,
            DefaultIgnoreCondition = JsonIgnoreCondition.WhenWritingNull,
            WriteIndented = true
        };

        return JsonSerializer.Serialize(doc, jsonOptions);
    }

    public Dictionary<string, object?> GenerateDocument()
    {
        var schemas = new Dictionary<string, object>();
        var paths = new Dictionary<string, Dictionary<string, object>>();

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
                                   type.GetGenericTypeDefinition() == typeof(IList<>)))
        {
            var elementType = type.GetGenericArguments()[0];
            return new Dictionary<string, object>
            {
                ["type"] = "array",
                ["items"] = GetOrCreateSchema(elementType, schemas)
            };
        }

        var schemaName = type.Name;

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
}
