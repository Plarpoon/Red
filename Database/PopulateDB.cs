using System.Data.SQLite;
using DSharpPlus;
using DSharpPlus.Entities;
using DSharpPlus.Interactivity.Extensions;
using Microsoft.Extensions.Logging;

namespace EvilBunny
{
    public static class PopulateDB
    {
        public static void Populate(DiscordClient discord)
        {
            // Create a new SQLite connection
            var connectionString = "Data Source=Database/EvilDB.sqlite;Version=3;";
            using var connection = new SQLiteConnection(connectionString);
            connection.Open();

            // Check all Discord guilds that have invited the bot
            discord.Logger.LogInformation("Checking all Discord guilds that have invited the bot...");
            foreach (var guild in discord.Guilds.Values)
            {
                // Check if the guild exists in the database
                var selectGuildCommand = new SQLiteCommand("SELECT * FROM guilds WHERE guild_id = @guild_id", connection);
                selectGuildCommand.Parameters.AddWithValue("@guild_id", guild.Id.ToString());
                using var reader = selectGuildCommand.ExecuteReader();
                if (!reader.HasRows)
                {
                    // Guild does not exist in the database, so insert it
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
            }
            discord.Logger.LogInformation("All Discord guilds checked successfully.");
        }
    }
}
