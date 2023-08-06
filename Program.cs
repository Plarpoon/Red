using System;
using System.IO;
using System.Linq;
using System.Reflection;
using System.Threading.Tasks;
using DSharpPlus;
using DSharpPlus.CommandsNext;
using DSharpPlus.SlashCommands;
using YamlDotNet.Serialization;

namespace EvilBunny
{
    public class Program
    {
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
                Intents = DiscordIntents.AllUnprivileged
            });

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
                .Where(t => t.IsSubclassOf(typeof(BaseCommandModule)) && t.Namespace == "EvilBunny.Command");
            foreach (var commandType in commandTypes)
                commands.RegisterCommands(commandType);

            // Load slash commands from the SlashCommand folder using reflection
            var slashCommandTypes = commandsAssembly.GetTypes()
                .Where(t => t.IsSubclassOf(typeof(ApplicationCommandModule)) && t.Namespace == "EvilBunny.SlashCommands");
            foreach (var slashCommandType in slashCommandTypes)
                slash.RegisterCommands(slashCommandType);

            // Connect to Discord and start the bot
            await discord.ConnectAsync();
            await Task.Delay(-1);
        }
    }

    public class Config
    {
        public string Token { get; set; }
    }
}
