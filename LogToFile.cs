using Serilog;

namespace EvilBunny
{
    public static class LogToFile
    {
        public static void Configure()
        {
            // Configure Serilog to write log messages to the console and a file
            Log.Logger = new LoggerConfiguration()
                .WriteTo.Console()
                .WriteTo.File(
                    path: "Logs/EvilLog-" + DateTime.Now.ToString("hhmm-") + ".log",
                    rollingInterval: RollingInterval.Day,
                    outputTemplate: "{Timestamp:dd-MM-yyyy HH:mm:ss.fff zzz} [{Level:u3}] {Message:lj}{NewLine}{Exception}",
                    retainedFileCountLimit: 3,  // Keep only the last 3 files
                    shared: true)
                .CreateLogger();
        }
    }
}
