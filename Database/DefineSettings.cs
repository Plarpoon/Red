using System.Data.SQLite;
using DSharpPlus;

namespace EvilBunny
{
    public static class DefineSettings
    {
        public static void Initialize(DiscordClient discord, SQLiteConnection connection)
        {
            #region global_settings

            // Create a key for each guild_id that the bot has been invited into
            foreach (var guild in discord.Guilds.Values)
            {
                var insertGlobalSettingCommand = new SQLiteCommand("INSERT INTO global_settings (guild_id, key, value) VALUES (@guild_id, @key, @value)", connection);
                insertGlobalSettingCommand.Parameters.AddWithValue("@guild_id", guild.Id.ToString());
                insertGlobalSettingCommand.Parameters.AddWithValue("@key", "GuildLoggingEnabled");
                insertGlobalSettingCommand.Parameters.AddWithValue("@value", "false");
                insertGlobalSettingCommand.ExecuteNonQuery();
            }

            #endregion

            #region channel_settings

            //...

            #endregion

            #region user_settings

            //...

            #endregion
        }
    }
}
