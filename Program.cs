using Discord;
using Discord.Interactions;
using Discord.WebSocket;
using Microsoft.Extensions.Configuration;
using Microsoft.Extensions.DependencyInjection;

namespace Red;

internal static class Program
{
    private static void Main()
    {
        IConfiguration config = new ConfigurationBuilder()
            .AddEnvironmentVariables("DC_")
            .AddJsonFile("app-settings.json", true)
            .Build();

        RunAsync(config).GetAwaiter().GetResult();
    }

    private static async Task RunAsync(IConfiguration configuration)
    {
        // Dependency injection is a key part of the Interactions framework but it needs to be disposed at the end of the app's lifetime.
        await using var services = ConfigureServices(configuration);

        var client = services.GetRequiredService<DiscordSocketClient>();
        var commands = services.GetRequiredService<InteractionService>();

        client.Log += LogAsync;
        commands.Log += LogAsync;

        // Slash Commands and Context Commands are can be automatically registered, but this process needs to happen after the client enters the READY state.
        // Since Global Commands take around 1 hour to register, we should use a test guild to instantly update and test our commands. To determine the method we should
        // register the commands with, we can check whether we are in a DEBUG environment and if we are, we can register the commands to a predetermined test guild.
        client.Ready += async () =>
        {
            if (IsDebug())
                // Id of the test guild can be provided from the Configuration object
                await commands.RegisterCommandsToGuildAsync(configuration.GetValue<ulong>("testGuild"));
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

    private static Task LogAsync(LogMessage message)
    {
        Console.WriteLine(message.ToString());
        return Task.CompletedTask;
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