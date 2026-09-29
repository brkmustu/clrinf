using Domain.Configuration.Parsers;
using Domain.Configuration.Models;
using YamlDotNet.Serialization;
using YamlDotNet.Serialization.NamingConventions;

namespace Application.Common.Configuration.Parsers;

public class YamlConfigParser : IConfigParser
{
    public CodeGenConfig Parse(string content)
    {
        var deserializer = new DeserializerBuilder()
            .WithNamingConvention(UnderscoredNamingConvention.Instance)
            .IgnoreUnmatchedProperties()
            .Build();
        return deserializer.Deserialize<CodeGenConfig>(content);
    }
}
