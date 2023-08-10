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
            var connectionString = "Data Source=Database/EvilDB.sqlite;Version=3;";
            using var connection = new SQLiteConnection(connectionString);
            connection.Open();

            // Delete all data from the guilds, channels, and users tables
            var deleteGuildsCommand = new SQLiteCommand("DELETE FROM guilds", connection);
            deleteGuildsCommand.ExecuteNonQuery();
            var deleteChannelsCommand = new SQLiteCommand("DELETE FROM channels", connection);
            deleteChannelsCommand.ExecuteNonQuery();
            var deleteUsersCommand = new SQLiteCommand("DELETE FROM users", connection);
            deleteUsersCommand.ExecuteNonQuery();

            // Check all Discord guilds that have invited the bot
            discord.Logger.LogInformation("Checking all Discord guilds that have invited the bot...");
            foreach (var guild in discord.Guilds.Values)
            {
                // Insert the guild into the database
                discord.Logger.LogInformation($"Inserting guild {guild.Name} into database...");
                var insertGuildCommand = new SQLiteCommand("INSERT INTO guilds (guild_id) VALUES (@guild_id)", connection);
                insertGuildCommand.Parameters.AddWithValue("@guild_id", guild.Id.ToString());
                insertGuildCommand.ExecuteNonQuery();
                discord.Logger.LogInformation($"Guild {guild.Name} inserted into database successfully.");

                // Populate the database with all of the settings from that guild
                foreach (var channel in guild.Channels.Values)
                {
                    var permissions = channel.PermissionsFor(guild.CurrentMember);
                    var insertChannelCommand = new SQLiteCommand("INSERT INTO channels (channel_id, channel_name, permissions) VALUES (@channel_id, @channel_name, @permissions)", connection);
                    insertChannelCommand.Parameters.AddWithValue("@channel_id", channel.Id.ToString());
                    insertChannelCommand.Parameters.AddWithValue("@channel_name", channel.Name);
                    insertChannelCommand.Parameters.AddWithValue("@permissions", permissions.ToString());
                    insertChannelCommand.ExecuteNonQuery();
                }

                foreach (var member in guild.Members.Values)
                {
                    var insertUserCommand = new SQLiteCommand("INSERT INTO users (user_id, username, discriminator, roles) VALUES (@user_id, @username, @discriminator, @roles)", connection);
                    insertUserCommand.Parameters.AddWithValue("@user_id", member.Id.ToString());
                    insertUserCommand.Parameters.AddWithValue("@username", member.Username);
                    insertUserCommand.Parameters.AddWithValue("@discriminator", member.Discriminator);
                    insertUserCommand.Parameters.AddWithValue("@roles", string.Join(",", member.Roles));
                    insertUserCommand.ExecuteNonQuery();
                }
            }
            discord.Logger.LogInformation("All Discord guilds checked successfully.");

            // Return a completed Task
            return Task.CompletedTask;
        }
    }
}