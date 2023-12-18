using DSharpPlus.CommandsNext;
using DSharpPlus.CommandsNext.Attributes;

namespace EvilBunny.Commands
{
    public class Commands : BaseCommandModule
    {
        [Command("hello")]
        [Description("Responds with a greeting")]
        public async Task Hello(CommandContext ctx)
        {
            await ctx.RespondAsync("Hello there!");
        }
    }
}
