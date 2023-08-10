using System.Data.SQLite;
using DSharpPlus;
using DSharpPlus.Entities;
using YamlDotNet.Serialization;
using DSharpPlus.Interactivity.Extensions;
using Microsoft.Extensions.Logging;

namespace EvilBunny
{
    public static class Database
    {
        /// <summary>
        /// Initializes the database.
        /// </summary>

        public static void Initialize(DiscordClient discord)
        {
            // Create a new SQLite connection
            discord.Logger.LogInformation("Creating SQLite connection...");
            var connectionString = "Data Source=Database/EvilDB.sqlite;Version=3;";
            using var connection = new SQLiteConnection(connectionString);
            connection.Open();
            discord.Logger.LogInformation("SQLite connection created successfully.");

            // Create the guilds table
            discord.Logger.LogInformation("Creating guilds table...");
            var createGuildsTableCommand = new SQLiteCommand("CREATE TABLE IF NOT EXISTS guilds (id INTEGER PRIMARY KEY AUTOINCREMENT, guild_id TEXT NOT NULL)", connection);
            createGuildsTableCommand.ExecuteNonQuery();
            discord.Logger.LogInformation("Guilds table created successfully.");

            // Create the channels table
            discord.Logger.LogInformation("Creating channels table...");
            var createChannelsTableCommand = new SQLiteCommand("CREATE TABLE IF NOT EXISTS channels (id INTEGER PRIMARY KEY AUTOINCREMENT, channel_id TEXT NOT NULL, channel_name TEXT NOT NULL, permissions TEXT NOT NULL)", connection);
            createChannelsTableCommand.ExecuteNonQuery();
            discord.Logger.LogInformation("Channels table created successfully.");

            // Create the users table
            discord.Logger.LogInformation("Creating users table...");
            var createUsersTableCommand = new SQLiteCommand("CREATE TABLE IF NOT EXISTS users (id INTEGER PRIMARY KEY AUTOINCREMENT, user_id TEXT NOT NULL, username TEXT NOT NULL, discriminator TEXT NOT NULL, roles TEXT NOT NULL)", connection);
            createUsersTableCommand.ExecuteNonQuery();
            discord.Logger.LogInformation("Users table created successfully.");

            // Create the settings table
            discord.Logger.LogInformation("Creating settings table...");
            var createSettingsTableCommand = new SQLiteCommand("CREATE TABLE IF NOT EXISTS settings (id INTEGER PRIMARY KEY AUTOINCREMENT, key TEXT NOT NULL, value TEXT NOT NULL)", connection);
            createSettingsTableCommand.ExecuteNonQuery();
            discord.Logger.LogInformation("Settings table created successfully.");

            // Call the Populate method of the PopulateDB class
            PopulateDB.Populate(discord);
        }
    }
}
