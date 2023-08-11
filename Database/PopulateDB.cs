using System.Data.SQLite;
using DSharpPlus;
using Microsoft.Extensions.Logging;

namespace EvilBunny
{
    public static class PopulateDB
    {
        public static async Task Populate(DiscordClient discord)
        {
            // Create a new SQLite connection
            discord.Logger.LogInformation("Creating SQLite connection...");
            var connectionString = "Data Source=Database/EvilDB.sqlite;Version=3;";
            using var connection = new SQLiteConnection(connectionString);
            await connection.OpenAsync();
            discord.Logger.LogInformation("SQLite connection created successfully.");

            // Call the DeleteData method to delete all data from the tables
            DeleteDB.DeleteData(discord, connection);

            // Check all Discord guilds that have invited the bot
            discord.Logger.LogInformation("Checking all Discord guilds that have invited the bot...");

            foreach (var guild in discord.Guilds.Values)
            {
                // Insert the guild into the database
                discord.Logger.LogInformation("Inserting guild {GuildName} into database...", guild.Name);
                var insertGuildCommand = new SQLiteCommand("INSERT INTO guilds (guild_id, guild_name) VALUES (@guild_id, @guild_name)", connection);
                insertGuildCommand.Parameters.AddWithValue("@guild_id", guild.Id.ToString());
                insertGuildCommand.Parameters.AddWithValue("@guild_name", guild.Name);
                await insertGuildCommand.ExecuteNonQueryAsync();
                discord.Logger.LogInformation("Inserted 1 row into the guilds table for guild {GuildName}.", guild.Name);

                // Populate the database with all of the settings from that guild
                int rowsAffected;
                foreach (var channel in guild.Channels.Values)
                {
                    var insertChannelCommand = new SQLiteCommand("INSERT INTO channels (guild_id, channel_id, channel_name) VALUES (@guild_id, @channel_id, @channel_name)", connection);
                    insertChannelCommand.Parameters.AddWithValue("@guild_id", guild.Id.ToString());
                    insertChannelCommand.Parameters.AddWithValue("@channel_id", channel.Id.ToString());
                    insertChannelCommand.Parameters.AddWithValue("@channel_name", channel.Name);
                    rowsAffected = await insertChannelCommand.ExecuteNonQueryAsync();
                    discord.Logger.LogInformation("Inserted {RowsAffected} row(s) into the channels table for channel {ChannelName} in guild {GuildName}.", rowsAffected, channel.Name, guild.Name);
                }

                foreach (var role in guild.Roles.Values)
                {
                    var insertRoleCommand = new SQLiteCommand("INSERT INTO roles (guild_id, role_id, role_name) VALUES (@guild_id, @role_id, @role_name)", connection);
                    insertRoleCommand.Parameters.AddWithValue("@guild_id", guild.Id.ToString());
                    insertRoleCommand.Parameters.AddWithValue("@role_id", role.Id.ToString());
                    insertRoleCommand.Parameters.AddWithValue("@role_name", role.Name);
                    rowsAffected = await insertRoleCommand.ExecuteNonQueryAsync();
                    discord.Logger.LogInformation("Inserted {RowsAffected} row(s) into the roles table for role {RoleName} in guild {GuildName}.", rowsAffected, role.Name, guild.Name);
                }
                discord.Logger.LogInformation("All Discord guilds checked successfully.");
            }
        }
    }
}
