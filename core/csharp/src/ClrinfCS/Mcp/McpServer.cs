using System;
using System.Collections.Generic;
using System.IO;
using System.Text.Json;
using System.Text.Json.Nodes;
using System.Threading;
using System.Threading.Tasks;
using ClrinfCS.Lint;
using ClrinfCS.Migration;

namespace ClrinfCS.Mcp;

public class McpServer
{
    public static async Task RunAsync(CancellationToken cancellationToken = default)
    {
        using var reader = new StreamReader(Console.OpenStandardInput());
        using var writer = new StreamWriter(Console.OpenStandardOutput()) { AutoFlush = true };

        while (!cancellationToken.IsCancellationRequested)
        {
            var line = await reader.ReadLineAsync(cancellationToken);
            if (line == null) break;
            if (string.IsNullOrWhiteSpace(line)) continue;

            try
            {
                var json = JsonNode.Parse(line);
                if (json is not JsonObject request) continue;

                var id = request["id"];
                var method = request["method"]?.GetValue<string>();

                if (string.IsNullOrEmpty(method)) continue;

                // Handle notifications (no id)
                if (id == null)
                {
                    continue;
                }

                var response = await HandleMethodAsync(method, request["params"] as JsonObject, id);
                if (response != null)
                {
                    await writer.WriteLineAsync(response.ToJsonString());
                }
            }
            catch (Exception ex)
            {
                var errResponse = new JsonObject
                {
                    ["jsonrpc"] = "2.0",
                    ["error"] = new JsonObject
                    {
                        ["code"] = -32603,
                        ["message"] = ex.Message
                    }
                };
                await writer.WriteLineAsync(errResponse.ToJsonString());
            }
        }
    }

    private static async Task<JsonObject?> HandleMethodAsync(string method, JsonObject? parameters, JsonNode id)
    {
        var response = new JsonObject
        {
            ["jsonrpc"] = "2.0",
            ["id"] = id.DeepClone()
        };

        switch (method)
        {
            case "initialize":
                response["result"] = new JsonObject
                {
                    ["protocolVersion"] = "2024-11-05",
                    ["capabilities"] = new JsonObject
                    {
                        ["tools"] = new JsonObject()
                    },
                    ["serverInfo"] = new JsonObject
                    {
                        ["name"] = "clrinfcs-mcp",
                        ["version"] = "1.0.0"
                    }
                };
                return response;

            case "tools/list":
                response["result"] = new JsonObject
                {
                    ["tools"] = GetAvailableTools()
                };
                return response;

            case "tools/call":
                var toolName = parameters?["name"]?.GetValue<string>();
                var toolArgs = parameters?["arguments"] as JsonObject;
                var toolResult = await ExecuteToolAsync(toolName, toolArgs);
                response["result"] = toolResult;
                return response;

            default:
                response["error"] = new JsonObject
                {
                    ["code"] = -32601,
                    ["message"] = $"Method '{method}' not found"
                };
                return response;
        }
    }

    private static JsonArray GetAvailableTools()
    {
        return new JsonArray
        {
            new JsonObject
            {
                ["name"] = "clrinfcs_lint",
                ["description"] = "Roslyn AST ile mimari kuralları ve katman sınırlarını (Domain saflığı, kural sözleşmeleri, controller bağımlılıkları) deterministik olarak denetler.",
                ["inputSchema"] = new JsonObject
                {
                    ["type"] = "object",
                    ["properties"] = new JsonObject
                    {
                        ["path"] = new JsonObject
                        {
                            ["type"] = "string",
                            ["description"] = "Denetlenecek proje veya kaynak dizini (varsayılan: geçerli dizin)"
                        },
                        ["strict"] = new JsonObject
                        {
                            ["type"] = "boolean",
                            ["description"] = "Uyarıları da hata say"
                        }
                    }
                }
            },
            new JsonObject
            {
                ["name"] = "clrinfcs_migrate",
                ["description"] = "Projeyi Roslyn AST ile bir mimari profilden diğerine dönüştürür (flat -> layered -> clean-cqrs).",
                ["inputSchema"] = new JsonObject
                {
                    ["type"] = "object",
                    ["properties"] = new JsonObject
                    {
                        ["project_path"] = new JsonObject { ["type"] = "string", ["description"] = "Dönüştürülecek proje dizini" },
                        ["target_arch"] = new JsonObject { ["type"] = "string", ["description"] = "Hedef mimari: layered veya clean-cqrs" },
                        ["dry_run"] = new JsonObject { ["type"] = "boolean", ["description"] = "Diske yazmadan simülasyon yap" }
                    },
                    ["required"] = new JsonArray { "target_arch" }
                }
            },
            new JsonObject
            {
                ["name"] = "clrinfcs_add_rule",
                ["description"] = "Mevcut hiçbir dosyaya dokunmadan, bağımsız ve derleme zamanı tip-güvenli bir IBusinessRule<TCommand> kural sınıfı oluşturur.",
                ["inputSchema"] = new JsonObject
                {
                    ["type"] = "object",
                    ["properties"] = new JsonObject
                    {
                        ["project_path"] = new JsonObject { ["type"] = "string", ["description"] = "Proje kök dizini" },
                        ["module"] = new JsonObject { ["type"] = "string", ["description"] = "Modül veya Entity adı (örn. Product)" },
                        ["rule_name"] = new JsonObject { ["type"] = "string", ["description"] = "Kural sınıfı adı (örn. MaxDiscountLimitRule)" },
                        ["command_name"] = new JsonObject { ["type"] = "string", ["description"] = "Hedef komut tipi (örn. CreateProductCommand)" },
                        ["error_code"] = new JsonObject { ["type"] = "string", ["description"] = "Hata kodu (örn. DISCOUNT_EXCEEDED)" },
                        ["error_message"] = new JsonObject { ["type"] = "string", ["description"] = "Kural ihlali hata mesajı" }
                    },
                    ["required"] = new JsonArray { "module", "rule_name", "command_name" }
                }
            }
        };
    }

    private static async Task<JsonObject> ExecuteToolAsync(string? toolName, JsonObject? args)
    {
        try
        {
            switch (toolName)
            {
                case "clrinfcs_lint":
                {
                    var path = args?["path"]?.GetValue<string>() ?? Environment.CurrentDirectory;
                    var violations = await ArchAnalyzer.AnalyzeDirectoryAsync(path);
                    var text = violations.Count == 0
                        ? "✔ Tebrikler! Hiçbir mimari veya kural ihlali bulunamadı."
                        : JsonSerializer.Serialize(violations, new JsonSerializerOptions { WriteIndented = true });

                    return new JsonObject
                    {
                        ["content"] = new JsonArray
                        {
                            new JsonObject { ["type"] = "text", ["text"] = text }
                        },
                        ["isError"] = violations.Any(v => v.Severity == "Error")
                    };
                }

                case "clrinfcs_migrate":
                {
                    var projectPath = args?["project_path"]?.GetValue<string>() ?? Environment.CurrentDirectory;
                    var targetArch = args?["target_arch"]?.GetValue<string>() ?? "clean-cqrs";
                    var dryRun = args?["dry_run"]?.GetValue<bool>() ?? false;

                    var result = await ProjectMigrationEngine.MigrateAsync(projectPath, targetArch, dryRun);
                    return new JsonObject
                    {
                        ["content"] = new JsonArray
                        {
                            new JsonObject
                            {
                                ["type"] = "text",
                                ["text"] = result.Success
                                    ? $"✔ Migrasyon başarıyla tamamlandı: {result.Message}\nTaşınan öğeler: {result.PlanItems.Count}"
                                    : $"✖ Migrasyon hatası: {result.Message}"
                            }
                        },
                        ["isError"] = !result.Success
                    };
                }

                case "clrinfcs_add_rule":
                {
                    var projectPath = args?["project_path"]?.GetValue<string>() ?? Environment.CurrentDirectory;
                    var module = args?["module"]?.GetValue<string>() ?? "Common";
                    var ruleName = args?["rule_name"]?.GetValue<string>() ?? "SampleRule";
                    var commandName = args?["command_name"]?.GetValue<string>() ?? "SampleCommand";
                    var errorCode = args?["error_code"]?.GetValue<string>() ?? "RULE_VIOLATION";
                    var errorMessage = args?["error_message"]?.GetValue<string>() ?? "Business rule violation occurred.";

                    if (!ruleName.EndsWith("Rule")) ruleName += "Rule";

                    // Determine target path
                    var rulesDir = Path.Combine(projectPath, "src");
                    if (Directory.Exists(rulesDir))
                    {
                        var appDir = Directory.GetDirectories(rulesDir, "*.Application").FirstOrDefault() ?? rulesDir;
                        rulesDir = Path.Combine(appDir, "Features", module, "Rules");
                    }
                    else
                    {
                        rulesDir = Path.Combine(projectPath, "Rules");
                    }

                    Directory.CreateDirectory(rulesDir);
                    var filePath = Path.Combine(rulesDir, $"{ruleName}.cs");

                    var code = $@"using System.Threading;
using System.Threading.Tasks;
using ClrinfCS.Core;

namespace Application.Features.{module}.Rules;

public sealed class {ruleName} : IBusinessRule<{commandName}>
{{
    public int Priority => 1;

    public ValueTask<RuleResult> EvaluateAsync({commandName} command, RequestContext requestContext, CancellationToken cancellationToken = default)
    {{
        // TODO: Is kuralı mantığını burada icra edin.
        // Ornek: if (command.Value > 100) return ValueTask.FromResult(RuleResult.Failed(""{errorCode}"", ""{errorMessage}""));

        return ValueTask.FromResult(RuleResult.Success());
    }}
}}
";
                    await File.WriteAllTextAsync(filePath, code);

                    return new JsonObject
                    {
                        ["content"] = new JsonArray
                        {
                            new JsonObject
                            {
                                ["type"] = "text",
                                ["text"] = $"✔ İzole iş kuralı oluşturuldu: {filePath}\nMevcut hiçbir dosyaya dokunulmadı."
                            }
                        },
                        ["isError"] = false
                    };
                }

                default:
                    return new JsonObject
                    {
                        ["content"] = new JsonArray
                        {
                            new JsonObject { ["type"] = "text", ["text"] = $"Bilinmeyen araç: {toolName}" }
                        },
                        ["isError"] = true
                    };
            }
        }
        catch (Exception ex)
        {
            return new JsonObject
            {
                ["content"] = new JsonArray
                {
                    new JsonObject { ["type"] = "text", ["text"] = $"Hata: {ex.Message}" }
                },
                ["isError"] = true
            };
        }
    }
}
