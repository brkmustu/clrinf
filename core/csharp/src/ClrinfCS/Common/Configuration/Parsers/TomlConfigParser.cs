using System;
using System.Diagnostics;
using System.Text.Json;
using Domain.Configuration.Parsers;
using Domain.Configuration.Models;
using Tomlyn;

namespace Application.Common.Configuration.Parsers;

public class TomlConfigParser : IConfigParser
{
    public CodeGenConfig Parse(string content)
    {
        Debug.Assert(!string.IsNullOrEmpty(content), "TOML content string must not be empty");

        var options = new TomlSerializerOptions
        {
            PropertyNamingPolicy = JsonNamingPolicy.SnakeCaseLower
        };
        var config = TomlSerializer.Deserialize<CodeGenConfig>(content, options);
        return config ?? throw new InvalidOperationException("Failed to deserialize TOML content into CodeGenConfig.");
    }
}
