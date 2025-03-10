mod bot;

use bot::commands::commands_list;
use bot::utils::config::Config;
use bot::utils::log::logger;

use log::{error, info};
use poise::Framework;
use poise::serenity_prelude as serenity;
use std::process;

#[tokio::main]
async fn main() {
    /* Run the bot and exit on error */
    if let Err(err) = run().await {
        error!("Application error: {:?}", err);
        process::exit(1);
    }
}

/* Asynchronously runs the bot and propagates any errors */
async fn run() -> Result<(), Box<dyn std::error::Error>> {
    /* Load and validate configuration, then initialize logger */
    let config = Config::load_or_create_and_validate_async().await?;
    logger::init_logger_with_config(&config).await?;
    info!("Starting bot.");

    /* Retrieve token and configure gateway intents */
    let token = &config.red.token;
    info!("Discord bot token retrieved.");
    let intents = serenity::GatewayIntents::GUILD_MESSAGES
        | serenity::GatewayIntents::DIRECT_MESSAGES
        | serenity::GatewayIntents::MESSAGE_CONTENT;
    info!("Gateway intents configured.");

    /* Determine debug guild ID if debug mode is enabled */
    let guild_id = config.debug.enable_debug.then(|| {
        config
            .debug
            .debug_server_id
            .parse::<u64>()
            .expect("Invalid debug_server_id")
    });
    if let Some(id) = guild_id {
        info!("Using debug guild ID: {}", id);
    }

    /* Build the Poise framework with registered commands */
    let framework = Framework::builder()
        .options(
            poise::FrameworkOptions::<(), Box<dyn std::error::Error + Send + Sync>> {
                commands: commands_list::get_commands().await,
                ..Default::default()
            },
        )
        .setup(|_ctx, _ready, _framework| Box::pin(async { Ok(()) }))
        .build();

    /* Create the Serenity client with the attached Poise framework */
    let mut client = serenity::ClientBuilder::new(token, intents)
        .framework(framework)
        .await?;

    /* Recover and set the application ID from Discord */
    let http = client.http.clone();
    let app_info = http.get_current_application_info().await?;
    info!("Recovered application ID: {}", app_info.id);
    http.set_application_id(app_info.id);

    /* If in debug mode, manually register guild commands for faster updates */
    if let Some(guild_id) = guild_id {
        info!(
            "Manually registering guild commands for guild: {}",
            guild_id
        );
        let http = client.http.clone();
        let commands = commands_list::get_commands().await;
        let command_data = poise::builtins::create_application_commands(&commands);
        http.create_guild_commands(serenity::GuildId::new(guild_id), &command_data)
            .await?;
    }

    /* Start the client and propagate any startup errors */
    client.start().await.map_err(|e| {
        error!("Client error: {:?}", e);
        e.into()
    })
}
