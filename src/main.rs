mod bot;

use bot::commands::{command_registration, commands_list};
use bot::utils::{application_id, config::Config, log::logger};
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
    /* Load configuration and initialize logger */
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
        info!("Using debug guild ID: {}", config.debug.debug_server_id);
        config.debug.debug_server_id
    });

    /* Build the Poise framework with registered commands */
    let commands = commands_list::get_commands().await;
    let options = poise::FrameworkOptions::<(), Box<dyn std::error::Error + Send + Sync>> {
        commands,
        ..Default::default()
    };
    let framework = Framework::builder()
        .options(options)
        .setup(|_ctx, _ready, _framework| Box::pin(async { Ok(()) }))
        .build();

    /* Create the Serenity client with the attached Poise framework */
    let mut client = serenity::ClientBuilder::new(token, intents)
        .framework(framework)
        .await?;

    /* Get the HTTP client from Serenity */
    let http = client.http.clone();

    /* Retrieve the application ID using */
    let application_id = application_id::get_application_id(&http).await?;
    info!("Application ID: {}", application_id);
    http.set_application_id(application_id);

    /* Delegate command registration to the command_registration module */
    command_registration::register_commands(&http, &config, guild_id).await?;

    /* Start the client and propagate any startup errors */
    client.start().await.map_err(|e| {
        error!("Client error: {:?}", e);
        e.into()
    })
}
