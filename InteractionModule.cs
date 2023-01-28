using Discord.Interactions;

namespace Red
{
    public class InteractionModule : InteractionModuleBase<SocketInteractionContext>
    {
        [SlashCommand("test", "test if bot is alive")]
        public async Task HandlePingCommand()
        {
            await RespondAsync("Bot is alive!");
        }
    }
}