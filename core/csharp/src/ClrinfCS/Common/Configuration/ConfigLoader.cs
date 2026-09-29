using System;
using System.Diagnostics;
using System.IO;
using Domain.Configuration.Models;
using Domain.Configuration.Parsers;
using Application.Common.Configuration.Parsers;

namespace Application.Common.Configuration;

public static class ConfigLoader
{
    public static CodeGenConfig Load(string filePath)
    {
        Debug.Assert(!string.IsNullOrEmpty(filePath), "Configuration file path must not be null or empty");

        if (string.IsNullOrEmpty(filePath))
            throw new ArgumentException("File path cannot be null or empty.", nameof(filePath));

        if (!System.IO.File.Exists(filePath))
            throw new FileNotFoundException($"Configuration file not found at: {filePath}");

        string content = System.IO.File.ReadAllText(filePath);
        Debug.Assert(content != null, "Configuration file content must not be null");

        string extension = System.IO.Path.GetExtension(filePath).ToLowerInvariant();

        IConfigParser parser = extension switch
        {
            ".toml" => new TomlConfigParser(),
            ".yaml" or ".yml" => new YamlConfigParser(),
            _ => throw new NotSupportedException($"Configuration file extension '{extension}' is not supported. Use .toml, .yaml, or .yml.")
        };

        return parser.Parse(content);
    }
}
