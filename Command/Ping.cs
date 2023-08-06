using System.Threading.Tasks;
using DSharpPlus.CommandsNext;
using DSharpPlus.CommandsNext.Attributes;

namespace EvilBunny.Command
{
    public class ExampleCommand : BaseCommandModule
    {
        [Command("hello")]
        [Description("Responds with a greeting")]
        public async Task Hello(CommandContext ctx)
        {
            await ctx.RespondAsync("Hello there!");
        }
    }
}
