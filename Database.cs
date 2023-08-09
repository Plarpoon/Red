using System.Data.SQLite;

namespace EvilBunny
{
    /// <summary>
    /// The database class.
    /// </summary>
    public class Database
    {
        /// <summary>
        /// Initializes the database.
        /// </summary>
        public static void Initialize()
        {
            // Create a new SQLite connection
            var connectionString = "Data Source=EvilDB.sqlite;Version=3;";
            using var connection = new SQLiteConnection(connectionString);
            connection.Open();

            // Create the users table
            var createUsersTableCommand = new SQLiteCommand("CREATE TABLE IF NOT EXISTS users (id INTEGER PRIMARY KEY AUTOINCREMENT, username TEXT NOT NULL, discriminator TEXT NOT NULL)", connection);
            createUsersTableCommand.ExecuteNonQuery();

            // Create the settings table
            var createSettingsTableCommand = new SQLiteCommand("CREATE TABLE IF NOT EXISTS settings (id INTEGER PRIMARY KEY AUTOINCREMENT, key TEXT NOT NULL, value TEXT NOT NULL)", connection);
            createSettingsTableCommand.ExecuteNonQuery();
        }
    }
}
