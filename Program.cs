using Microsoft.Extensions.Configuration;

namespace Red;

internal static class Program
{
    private static void Main()
    {
        IConfiguration config = new ConfigurationBuilder()
            .AddJsonFile("app-settings.json", true)
            .Build();

        RunAsync(config).GetAwaiter().GetResult();
    }
}