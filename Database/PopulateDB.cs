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

            // Check all Discord guilds that have invited the bot
            discord.Logger.LogInformation("Checking all Discord guilds that have invited the bot...");

            foreach (var guild in discord.Guilds.Values)
            {
                // Begin a transaction
                using var transaction = connection.BeginTransaction();

                // Insert the guild into the database if it doesn't already exist
                discord.Logger.LogInformation("Inserting guild {GuildName} into database if it doesn't already exist...", guild.Name);
                var insertGuildCommand = new SQLiteCommand("INSERT OR IGNORE INTO guilds (guild_id, guild_name) VALUES (@guild_id, @guild_name)", connection);
                insertGuildCommand.Parameters.AddWithValue("@guild_id", guild.Id.ToString());
                insertGuildCommand.Parameters.AddWithValue("@guild_name", guild.Name);
                await insertGuildCommand.ExecuteNonQueryAsync();
                discord.Logger.LogInformation("Inserted 1 row into the guilds table for guild {GuildName} if it didn't already exist.", guild.Name);

                // Populate the database with all of the settings from that guild
                int rowsAffected;
                int channelCount = 0;
                foreach (var channel in guild.Channels.Values)
                {
                    var insertChannelCommand = new SQLiteCommand("INSERT OR IGNORE INTO channels (guild_id, channel_id, channel_name) VALUES (@guild_id, @channel_id, @channel_name)", connection);
                    insertChannelCommand.Parameters.AddWithValue("@guild_id", guild.Id.ToString());
                    insertChannelCommand.Parameters.AddWithValue("@channel_id", channel.Id.ToString());
                    insertChannelCommand.Parameters.AddWithValue("@channel_name", channel.Name);
                    rowsAffected = await insertChannelCommand.ExecuteNonQueryAsync();
                    channelCount += rowsAffected;
                }
                discord.Logger.LogInformation("Inserted {ChannelCount} row(s) into the channels table for guild {GuildName}.", channelCount, guild.Name);

                int roleCount = 0;
                foreach (var role in guild.Roles.Values)
                {
                    var insertRoleCommand = new SQLiteCommand("INSERT OR IGNORE INTO roles (guild_id, role_id, role_name) VALUES (@guild_id, @role_id, @role_name)", connection);
                    insertRoleCommand.Parameters.AddWithValue("@guild_id", guild.Id.ToString());
                    insertRoleCommand.Parameters.AddWithValue("@role_id", role.Id.ToString());
                    insertRoleCommand.Parameters.AddWithValue("@role_name", role.Name);
                    rowsAffected = await insertRoleCommand.ExecuteNonQueryAsync();
                    roleCount += rowsAffected;
                }
                discord.Logger.LogInformation("Inserted {RoleCount} row(s) into the roles table for guild {GuildName}.", roleCount, guild.Name);

                int userCount = 0;
                foreach (var member in await guild.GetAllMembersAsync())
                {
                    var insertUserCommand = new SQLiteCommand("INSERT OR IGNORE INTO users (guild_id, user_id, username) VALUES (@guild_id, @user_id, @username)", connection);
                    insertUserCommand.Parameters.AddWithValue("@guild_id", guild.Id.ToString());
                    insertUserCommand.Parameters.AddWithValue("@user_id", member.Id.ToString());
                    insertUserCommand.Parameters.AddWithValue("@username", member.Username);
                    rowsAffected = await insertUserCommand.ExecuteNonQueryAsync();
                    userCount += rowsAffected;
                }
                discord.Logger.LogInformation("Inserted {UserCount} row(s) into the users table for guild {GuildName}.", userCount, guild.Name);

                // Commit the transaction
                transaction.Commit();
            }

            discord.Logger.LogInformation("All Discord guilds checked successfully.");
        }
    }
}
