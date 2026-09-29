using System.ComponentModel;
using System.Threading.Tasks;
using ClrinfCS.Mcp;
using Spectre.Console.Cli;

namespace ClrinfCS.Commands.Mcp;

public class McpCliCommand : AsyncCommand<McpCliCommand.Settings>
{
    public class Settings : CommandSettings
    {
    }

    public override async Task<int> ExecuteAsync(CommandContext context, Settings settings)
    {
        await McpServer.RunAsync();
        return 0;
    }
}
