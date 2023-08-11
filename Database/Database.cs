using System.Data.SQLite;
using DSharpPlus;
using Microsoft.Extensions.Logging;

namespace EvilBunny
{
    public static class Database
    {
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
            var createGuildsTableCommand = new SQLiteCommand("CREATE TABLE IF NOT EXISTS guilds (guild_id TEXT PRIMARY KEY, guild_name TEXT)", connection);
            createGuildsTableCommand.ExecuteNonQuery();
            discord.Logger.LogInformation("Guilds table created successfully.");

            // Create the channels table
            discord.Logger.LogInformation("Creating channels table...");
            var createChannelsTableCommand = new SQLiteCommand("CREATE TABLE IF NOT EXISTS channels (guild_id TEXT, channel_id TEXT, channel_name TEXT, PRIMARY KEY (guild_id, channel_id))", connection);
            createChannelsTableCommand.ExecuteNonQuery();
            discord.Logger.LogInformation("Channels table created successfully.");

            // Create the roles table
            discord.Logger.LogInformation("Creating roles table...");
            var createRolesTableCommand = new SQLiteCommand("CREATE TABLE IF NOT EXISTS roles (guild_id TEXT, role_id TEXT, role_name TEXT, PRIMARY KEY (guild_id, role_id))", connection);
            createRolesTableCommand.ExecuteNonQuery();
            discord.Logger.LogInformation("Roles table created successfully.");

            // Create the users table
            discord.Logger.LogInformation("Creating users table...");
            var createUsersTableCommand = new SQLiteCommand("CREATE TABLE IF NOT EXISTS users (guild_id TEXT, user_id TEXT, username TEXT, PRIMARY KEY (guild_id, user_id))", connection);
            createUsersTableCommand.ExecuteNonQuery();
            discord.Logger.LogInformation("Users table created successfully.");

            // Create the global_settings table
            discord.Logger.LogInformation("Creating global_settings table...");
            var createGlobalSettingsTableCommand = new SQLiteCommand("CREATE TABLE IF NOT EXISTS global_settings (guild_id TEXT, key TEXT, value TEXT, PRIMARY KEY (guild_id, key))", connection);
            createGlobalSettingsTableCommand.ExecuteNonQuery();
            discord.Logger.LogInformation("Global_settings table created successfully.");

            // Create the channel_settings table
            discord.Logger.LogInformation("Creating channel_settings table...");
            var createChannelSettingsTableCommand = new SQLiteCommand("CREATE TABLE IF NOT EXISTS channel_settings (guild_id TEXT, key TEXT, value TEXT, channel_id TEXT, PRIMARY KEY (guild_id, key, channel_id))", connection);
            createChannelSettingsTableCommand.ExecuteNonQuery();
            discord.Logger.LogInformation("Channel_settings table created successfully.");

            // Create the user_settings table
            discord.Logger.LogInformation("Creating user_settings table...");
            var createUserSettingsTableCommand = new SQLiteCommand("CREATE TABLE IF NOT EXISTS user_settings (guild_id TEXT, key TEXT, value TEXT, user_id TEXT, PRIMARY KEY (guild_id, key, user_id))", connection);
            createUserSettingsTableCommand.ExecuteNonQuery();
            discord.Logger.LogInformation("User_settings table created successfully.");
        }
    }
}
