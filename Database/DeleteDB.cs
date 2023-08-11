using System.Data.SQLite;
using DSharpPlus;
using Microsoft.Extensions.Logging;

namespace EvilBunny
{
    public static class DeleteDB
    {
        public static void DeleteData(DiscordClient discord, SQLiteConnection connection)
        {
            // Delete all data from the guilds, channels, roles, users, global_settings, channel_settings, and user_settings tables
            discord.Logger.LogInformation("Deleting all data from the guilds, channels, roles, users, global_settings, channel_settings, and user_settings tables...");
            var deleteGuildsCommand = new SQLiteCommand("DELETE FROM guilds", connection);
            var rowsAffected = deleteGuildsCommand.ExecuteNonQuery();
            discord.Logger.LogInformation("Deleted {RowsAffected} rows from the guilds table.", rowsAffected);
            var deleteChannelsCommand = new SQLiteCommand("DELETE FROM channels", connection);
            rowsAffected = deleteChannelsCommand.ExecuteNonQuery();
            discord.Logger.LogInformation("Deleted {RowsAffected} rows from the channels table.", rowsAffected);
            var deleteRolesCommand = new SQLiteCommand("DELETE FROM roles", connection);
            rowsAffected = deleteRolesCommand.ExecuteNonQuery();
            discord.Logger.LogInformation("Deleted {RowsAffected} rows from the roles table.", rowsAffected);
            var deleteUsersCommand = new SQLiteCommand("DELETE FROM users", connection);
            rowsAffected = deleteUsersCommand.ExecuteNonQuery();
            discord.Logger.LogInformation("Deleted {RowsAffected} rows from the users table.", rowsAffected);
            var deleteGlobalSettingsCommand = new SQLiteCommand("DELETE FROM global_settings", connection);
            rowsAffected = deleteGlobalSettingsCommand.ExecuteNonQuery();
            discord.Logger.LogInformation("Deleted {RowsAffected} rows from the global_settings table.", rowsAffected);
            var deleteChannelSettingsCommand = new SQLiteCommand("DELETE FROM channel_settings", connection);
            rowsAffected = deleteChannelSettingsCommand.ExecuteNonQuery();
            discord.Logger.LogInformation("Deleted {RowsAffected} rows from the channel_settings table.", rowsAffected);
            var deleteUserSettingsCommand = new SQLiteCommand("DELETE FROM user_settings", connection);
            rowsAffected = deleteUserSettingsCommand.ExecuteNonQuery();
            discord.Logger.LogInformation("Deleted {RowsAffected} rows from the user_settings table.", rowsAffected);
        }
    }
}
