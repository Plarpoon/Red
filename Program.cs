using Discord;
using Discord.Interactions;
using Discord.WebSocket;
using Microsoft.Extensions.Configuration;
using Microsoft.Extensions.DependencyInjection;
using Red.Services;

namespace Red;

internal static class Program
{
    private static void Main()
    {
        LoggingHandler.Serilog();

        IConfiguration config = new ConfigurationBuilder()
            .AddJsonFile("secrets.json", true)
            .AddJsonFile("app-settings.json", true)
            .Build();

        RunAsync(config).GetAwaiter().GetResult();
    }

    private static async Task RunAsync(IConfiguration configuration)
    {
        await using var services = ConfigureServices(configuration);

        var client = services.GetRequiredService<DiscordSocketClient>();
        var commands = services.GetRequiredService<InteractionService>();

        client.Log += LoggingHandler.LogAsync;
        commands.Log += LoggingHandler.LogAsync;

        // Slash Commands and Context Commands can be automatically registered, but this process needs to happen after the client enters the READY state.
        // Since Global Commands take around 1 hour to register, we should use a test guild to instantly update and test our commands.
        client.Ready += async () =>
        {
            if (IsDebug())
                await commands.RegisterCommandsToGuildAsync(
                    configuration.GetValue<ulong>("698934302720786503")); // Add here ID of testing guild.
            else
                await commands.RegisterCommandsGloballyAsync();
        };

        // Here we can initialize the service that will register and execute our commands
        await services.GetRequiredService<CommandHandler>().InitializeAsync();

        // Bot token can be provided from the Configuration object we set up earlier
        await client.LoginAsync(TokenType.Bot, configuration["token"]);
        await client.StartAsync();

        await Task.Delay(Timeout.Infinite);
    }

    private static ServiceProvider ConfigureServices(IConfiguration configuration)
    {
        return new ServiceCollection()
            .AddSingleton(configuration)
            .AddSingleton<DiscordSocketClient>()
            .AddSingleton(x => new InteractionService(x.GetRequiredService<DiscordSocketClient>()))
            .AddSingleton<CommandHandler>()
            .BuildServiceProvider();
    }

    private static bool IsDebug()
    {
#if DEBUG
        return true;
#else
                return false;
#endif
    }
}