using Discord;
using Serilog;
using Serilog.Events;

namespace Red.Services;

internal class LoggingHandler
{
    internal static void Serilog()
    {
        Log.Logger = new LoggerConfiguration()
        .MinimumLevel.Verbose()
        .MinimumLevel.Override("Microsoft", LogEventLevel.Warning)
        .WriteTo.Console()
        .WriteTo.File("logs/log.txt",
            rollingInterval: RollingInterval.Minute,
            rollOnFileSizeLimit: true)
        .CreateLogger();

        Log.Information("Red starting!");
        Log.Information("Logging initialized!");
    }

    internal static Task LogAsync(LogMessage message)
    {
        if (message.Exception is Discord.Commands.CommandException cmdException)
        {
            switch (message.Severity)
            {
                case Discord.LogSeverity.Critical:
                    Log.Fatal($"[Command/{message.Severity}] {cmdException.Command.Aliases[0]}"
            + $" failed to execute in {cmdException.Context.Channel}.");
                    break;

                case Discord.LogSeverity.Debug:
                    Log.Debug($"[Command/{message.Severity}] {cmdException.Command.Aliases[0]}"
            + $" failed to execute in {cmdException.Context.Channel}.");
                    break;

                case Discord.LogSeverity.Error:
                    Log.Error($"[Command/{message.Severity}] {cmdException.Command.Aliases[0]}"
            + $" failed to execute in {cmdException.Context.Channel}.");
                    break;

                case Discord.LogSeverity.Info:
                    Log.Information($"[Command/{message.Severity}] {cmdException.Command.Aliases[0]}"
            + $" failed to execute in {cmdException.Context.Channel}.");
                    break;

                case Discord.LogSeverity.Verbose:
                    Log.Verbose($"[Command/{message.Severity}] {cmdException.Command.Aliases[0]}"
            + $" failed to execute in {cmdException.Context.Channel}.");
                    break;

                case Discord.LogSeverity.Warning:
                    Log.Warning($"[Command/{message.Severity}] {cmdException.Command.Aliases[0]}"
            + $" failed to execute in {cmdException.Context.Channel}.");
                    break;
            }
        }
        else
        {
            switch (message.Severity)
            {
                case Discord.LogSeverity.Critical:
                    Log.Fatal($"[General/{message.Severity}] {message}");
                    break;

                case Discord.LogSeverity.Debug:
                    Log.Debug($"[General/{message.Severity}] {message}");
                    break;

                case Discord.LogSeverity.Error:
                    Log.Error($"[General/{message.Severity}] {message}");
                    break;

                case Discord.LogSeverity.Info:
                    Log.Information($"[General/{message.Severity}] {message}");
                    break;

                case Discord.LogSeverity.Verbose:
                    Log.Verbose($"[General/{message.Severity}] {message}");
                    break;

                case Discord.LogSeverity.Warning:
                    Log.Warning($"[General/{message.Severity}] {message}");
                    break;
            }
        }

        Console.WriteLine(message);   /*DEBUGGING*/
        return Task.CompletedTask;
    }
}