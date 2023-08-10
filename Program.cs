using System;
using System.IO;
using System.Linq;
using System.Reflection;
using System.Threading;
using System.Threading.Tasks;
using DSharpPlus;
using DSharpPlus.CommandsNext;
using DSharpPlus.SlashCommands;
using DSharpPlus.Interactivity.Extensions;
using YamlDotNet.Serialization;
using Microsoft.Extensions.Logging;

namespace EvilBunny
{
    /// <summary>
    /// The main program class.
    /// </summary>
    public class Program
    {
        /// <summary>
        /// The main entry point of the program.
        /// </summary>
        /// <param name="args">The command line arguments.</param>
        public static async Task Main(string[] args)
        {
            // Read the bot token from the YAML file
            var deserializer = new DeserializerBuilder().Build();
            var config = deserializer.Deserialize<Config>(File.ReadAllText("config.yaml"));
            var token = config.Token;

            // Create the Discord client
            var discord = new DiscordClient(new DiscordConfiguration
            {
                Token = token,
                TokenType = TokenType.Bot,
                Intents = DiscordIntents.AllUnprivileged,
                MinimumLogLevel = LogLevel.Information,
                LogTimestampFormat = "dd MMM yyyy - hh:mm:ss tt"
            });

            // Initialize the database
            Database.Initialize(discord);

            // Subscribe to the GuildCreated event
            // When invited into a new Guild trigger below code
            discord.GuildCreated += async (s, e) => await PopulateDB.Populate(discord);

            // Enable Interactivity
            var interactivity = discord.UseInteractivity();

            // Register the CommandsNext module
            var commands = discord.UseCommandsNext(new CommandsNextConfiguration
            {
                StringPrefixes = new[] { "!" }
            });

            // Register the SlashCommands module
            var slash = discord.UseSlashCommands();

            // Load standard commands from the Commands folder using reflection
            var commandsAssembly = Assembly.GetExecutingAssembly();
            var commandTypes = commandsAssembly.GetTypes()
                .Where(t => t.IsSubclassOf(typeof(BaseCommandModule)) && t.Namespace == "EvilBunny.Commands");
            foreach (var commandType in commandTypes)
                commands.RegisterCommands(commandType);

            // Load slash commands from the SlashCommand folder using reflection
            var slashCommandTypes = commandsAssembly.GetTypes()
                .Where(t => t.IsSubclassOf(typeof(ApplicationCommandModule)) && t.Namespace == "EvilBunny.SlashCommands");
            foreach (var slashCommandType in slashCommandTypes)
                slash.RegisterCommands(slashCommandType);

            // Connect to Discord and start the bot
            await discord.ConnectAsync();

            // Wait for a cancellation token to be triggered by CTRL-C on the console
            var cts = new CancellationTokenSource();
            Console.CancelKeyPress += (sender, e) =>
            {
                e.Cancel = true;
                cts.Cancel();
            };

            try
            {
                await Task.Delay(-1, cts.Token);
            }
            catch (TaskCanceledException)
            {
                Console.WriteLine();
            }

            // Disconnect from Discord and stop the bot gracefully
            await discord.DisconnectAsync();
        }
    }

    /// <summary>
    /// The configuration class.
    /// </summary>
    public class Config
    {
        /// <summary>
        /// Gets or sets the bot token.
        /// </summary>
        public string Token { get; set; } = "";
    }
}
