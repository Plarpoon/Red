using Discord.Commands;
using Discord;

namespace Red
{
    public class PrefixModule : ModuleBase<SocketCommandContext>
    {
        [Command("test")]
        public async Task HandlePingCommand()
        {
            await Context.Message.ReplyAsync("Bot is alive!");
        }
    }
}