using System.Data.SQLite;
using DSharpPlus;
using DSharpPlus.Entities;
using DSharpPlus.Interactivity.Extensions;
using Microsoft.Extensions.Logging;

namespace EvilBunny
{
    public static class PopulateDB
    {
        public static Task Populate(DiscordClient discord)
        {
            // Create a new SQLite connection
            discord.Logger.LogInformation("Creating SQLite connection...");
            var connectionString = "Data Source=Database/EvilDB.sqlite;Version=3;";
            using var connection = new SQLiteConnection(connectionString);
            connection.Open();
            discord.Logger.LogInformation("SQLite connection created successfully.");

            // Delete all data from the guilds, channels, and users tables
            discord.Logger.LogInformation("Deleting all data from the guilds, channels, and users tables...");
            var deleteGuildsCommand = new SQLiteCommand("DELETE FROM guilds", connection);
            var rowsAffected = deleteGuildsCommand.ExecuteNonQuery();
            discord.Logger.LogInformation($"Deleted {rowsAffected} rows from the guilds table.");
            var deleteChannelsCommand = new SQLiteCommand("DELETE FROM channels", connection);
            rowsAffected = deleteChannelsCommand.ExecuteNonQuery();
            discord.Logger.LogInformation($"Deleted {rowsAffected} rows from the channels table.");
            var deleteUsersCommand = new SQLiteCommand("DELETE FROM users", connection);
            rowsAffected = deleteUsersCommand.ExecuteNonQuery();
            discord.Logger.LogInformation($"Deleted {rowsAffected} rows from the users table.");

            // Check all Discord guilds that have invited the bot
            discord.Logger.LogInformation("Checking all Discord guilds that have invited the bot...");
            foreach (var guild in discord.Guilds.Values)
            {
                // Insert the guild into the database
                discord.Logger.LogInformation($"Inserting guild {guild.Name} into database...");
                var admins = string.Join(",", guild.Members.Values.Where(m => m.PermissionsIn(guild.GetDefaultChannel()).HasPermission(Permissions.Administrator)).Select(m => m.Id.ToString()));
                var insertGuildCommand = new SQLiteCommand("INSERT INTO guilds (guild_id, admins) VALUES (@guild_id, @admins)", connection);
                insertGuildCommand.Parameters.AddWithValue("@guild_id", guild.Id.ToString());
                insertGuildCommand.Parameters.AddWithValue("@admins", admins);
                insertGuildCommand.ExecuteNonQuery();
                discord.Logger.LogInformation($"Inserted {rowsAffected} row(s) into the guilds table for guild {guild.Name}.");

                // Populate the database with all of the settings from that guild
                foreach (var channel in guild.Channels.Values)
                {
                    var permissions = channel.PermissionsFor(guild.CurrentMember);
                    var insertChannelCommand = new SQLiteCommand("INSERT INTO channels (channel_id, channel_name, permissions) VALUES (@channel_id, @channel_name, @permissions)", connection);
                    insertChannelCommand.Parameters.AddWithValue("@channel_id", channel.Id.ToString());
                    insertChannelCommand.Parameters.AddWithValue("@channel_name", channel.Name);
                    insertChannelCommand.Parameters.AddWithValue("@permissions", permissions.ToString());
                    rowsAffected = insertChannelCommand.ExecuteNonQuery();
                    discord.Logger.LogInformation($"Inserted {rowsAffected} row(s) into the channels table for channel {channel.Name} in guild {guild.Name}.");
                }

                foreach (var member in guild.Members.Values)
                {
                    var insertUserCommand = new SQLiteCommand("INSERT INTO users (user_id, username, discriminator, guild_id, guild_name) VALUES (@user_id, @username, @discriminator, @guild_id, @guild_name)", connection);
                    insertUserCommand.Parameters.AddWithValue("@user_id", member.Id.ToString());
                    insertUserCommand.Parameters.AddWithValue("@username", member.Username);
                    insertUserCommand.Parameters.AddWithValue("@discriminator", member.Discriminator);
                    insertUserCommand.Parameters.AddWithValue("@guild_id", guild.Id.ToString());
                    insertUserCommand.Parameters.AddWithValue("@guild_name", guild.Name);
                    insertUserCommand.ExecuteNonQuery();
                }

                foreach (var role in guild.Roles.Values)
                {
                    var insertRoleCommand = new SQLiteCommand("INSERT INTO roles (role_id, role_name, guild_id) VALUES (@role_id, @role_name, @guild_id)", connection);
                    insertRoleCommand.Parameters.AddWithValue("@role_id", role.Id.ToString());
                    insertRoleCommand.Parameters.AddWithValue("@role_name", role.Name);
                    insertRoleCommand.Parameters.AddWithValue("@guild_id", guild.Id.ToString());
                    insertRoleCommand.ExecuteNonQuery();
                }

                var insertSettingCommand = new SQLiteCommand("INSERT INTO settings (key, value, guild_id) VALUES (@key, @value, @guild_id)", connection);
                insertSettingCommand.Parameters.AddWithValue("@key", "LoggingEnabled");
                insertSettingCommand.Parameters.AddWithValue("@value", "OFF");
                insertSettingCommand.Parameters.AddWithValue("@guild_id", guild.Id.ToString());
                insertSettingCommand.ExecuteNonQuery();
            }
            discord.Logger.LogInformation("All Discord guilds checked successfully.");

            // Return a completed Task
            return Task.CompletedTask;
        }
    }
}
