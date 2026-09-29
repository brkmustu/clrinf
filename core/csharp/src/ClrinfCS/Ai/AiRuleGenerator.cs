using System;
using System.IO;
using System.Net.Http;
using System.Text;
using System.Text.Json;
using System.Text.Json.Nodes;
using System.Threading.Tasks;

namespace ClrinfCS.Ai;

public record AiRuleGenerationResult(
    bool Success,
    string RuleFilePath,
    string TestFilePath,
    string Message
);

public class AiRuleGenerator
{
    public static async Task<AiRuleGenerationResult> GenerateRuleAsync(
        string projectPath,
        string module,
        string ruleName,
        string commandName,
        string naturalLanguageRule,
        string? llmEndpoint = null,
        string? llmModel = null)
    {
        if (!ruleName.EndsWith("Rule"))
            ruleName += "Rule";

        string errorCode = ruleName.Replace("Rule", "").ToUpperInvariant() + "_VIOLATION";

        // Find Application / Features directory
        string rootDir = Path.GetFullPath(projectPath);
        string appDir = Path.Combine(rootDir, "src");
        if (Directory.Exists(appDir))
        {
            var found = Directory.GetDirectories(appDir, "*.Application");
            if (found.Length > 0) appDir = found[0];
        }
        else
        {
            appDir = rootDir;
        }

        string rulesDir = Path.Combine(appDir, "Features", module, "Rules");
        Directory.CreateDirectory(rulesDir);
        string ruleFilePath = Path.Combine(rulesDir, $"{ruleName}.cs");

        // Find Tests directory
        string testsDir = Path.Combine(rootDir, "tests", $"{module}Tests", "Rules");
        if (!Directory.Exists(Path.Combine(rootDir, "tests")))
        {
            testsDir = Path.Combine(rulesDir, "Tests");
        }
        Directory.CreateDirectory(testsDir);
        string testFilePath = Path.Combine(testsDir, $"{ruleName}Tests.cs");

        string ruleLogic = "// TODO: Kural mantığını uygulayın.\n        // if (command is invalid) return ValueTask.FromResult(RuleResult.Failed(\"" + errorCode + "\", \"" + naturalLanguageRule + "\"));";

        // Optional LLM synthesis
        if (!string.IsNullOrWhiteSpace(llmEndpoint))
        {
            var synthesized = await TrySynthesizeWithLlmAsync(llmEndpoint, llmModel ?? "deepseek-coder", naturalLanguageRule, commandName, errorCode);
            if (!string.IsNullOrWhiteSpace(synthesized))
            {
                ruleLogic = synthesized;
            }
        }

        string ruleCode = $@"using System.Threading;
using System.Threading.Tasks;
using ClrinfCS.Core;

namespace Application.Features.{module}.Rules;

/// <summary>
/// İş Kuralı: {naturalLanguageRule}
/// </summary>
public sealed class {ruleName} : IBusinessRule<{commandName}>
{{
    public int Priority => 1;

    public ValueTask<RuleResult> EvaluateAsync({commandName} command, RequestContext requestContext, CancellationToken cancellationToken = default)
    {{
        cancellationToken.ThrowIfCancellationRequested();

        {ruleLogic}

        return ValueTask.FromResult(RuleResult.Success());
    }}
}}
";

        string testCode = $@"using System.Threading.Tasks;
using Application.Features.{module}.Rules;
using ClrinfCS.Core;
using Xunit;

namespace Tests.{module}.Rules;

public class {ruleName}Tests
{{
    private static readonly RequestContext Context = new(""tenant-default"", ""corr-123"", ""cause-456"");

    [Fact]
    public async Task WhenConditionIsMet_RuleSucceeds()
    {{
        var rule = new {ruleName}();
        var command = new {commandName}();

        var result = await rule.EvaluateAsync(command, Context);

        Assert.True(result.IsSuccess);
    }}

    [Fact]
    public async Task WhenConditionViolated_RuleFailsWithExpectedErrorCode()
    {{
        var rule = new {ruleName}();
        var command = new {commandName}();

        var result = await rule.EvaluateAsync(command, Context);

        // Doğrulama örneği:
        // Assert.False(result.IsSuccess);
        // Assert.Equal(""{errorCode}"", result.ErrorCode);
    }}
}}
";

        await File.WriteAllTextAsync(ruleFilePath, ruleCode);
        await File.WriteAllTextAsync(testFilePath, testCode);

        return new AiRuleGenerationResult(
            true,
            ruleFilePath,
            testFilePath,
            $"✔ İzole iş kuralı ve xUnit testleri başarıyla üretildi.\nKural: {ruleFilePath}\nTest: {testFilePath}\nMevcut hiçbir dosyaya dokunulmadı."
        );
    }

    private static async Task<string?> TrySynthesizeWithLlmAsync(string endpoint, string model, string ruleText, string commandName, string errorCode)
    {
        try
        {
            using var client = new HttpClient { Timeout = TimeSpan.FromSeconds(15) };
            var prompt = $"Write only the C# evaluation body (inside EvaluateAsync) for this business rule: '{ruleText}'. Command type is '{commandName}'. If condition fails, return 'RuleResult.Failed(\"{errorCode}\", \"{ruleText}\");'. Do not write markdown, code fences, or explanations. Only C# statements.";

            var payload = new JsonObject
            {
                ["model"] = model,
                ["prompt"] = prompt,
                ["stream"] = false
            };

            var content = new StringContent(payload.ToJsonString(), Encoding.UTF8, "application/json");
            var res = await client.PostAsync(endpoint.TrimEnd('/') + "/api/generate", content);
            if (res.IsSuccessStatusCode)
            {
                var responseBody = await res.Content.ReadAsStringAsync();
                var json = JsonNode.Parse(responseBody);
                var generated = json?["response"]?.GetValue<string>();
                if (!string.IsNullOrWhiteSpace(generated))
                {
                    return generated.Trim('`', ' ', '\n', '\r');
                }
            }
        }
        catch
        {
            // Fall back to template logic on any connection or LLM error
        }
        return null;
    }
}
