using Domain.Configuration.Models;

namespace Domain.Configuration.Parsers;

public interface IConfigParser
{
    CodeGenConfig Parse(string content);
}
