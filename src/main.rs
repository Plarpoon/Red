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
    /* Run the bot and handle errors by logging and exiting */
    run().await.unwrap_or_else(|err| {
        error!("Application error: {:?}", err);
        process::exit(1)
    })
}

/* Asynchronously runs the bot and propagates any errors */
async fn run() -> Result<(), Box<dyn std::error::Error>> {
    /* Load and validate the configuration */
    let config = Config::load_or_create_and_validate_async().await?;

    /* Initialize the logger using the loaded configuration */
    logger::init_logger_with_config(&config).await?;
    info!("Starting bot.");

    /* Retrieve the Discord bot token from the configuration */
    let token = &config.red.token;
    info!("Discord bot token retrieved.");

    /* Configure the gateway intents required by the bot */
    let intents = serenity::GatewayIntents::GUILD_MESSAGES
        | serenity::GatewayIntents::DIRECT_MESSAGES
        | serenity::GatewayIntents::MESSAGE_CONTENT;
    info!("Gateway intents configured.");

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

    /* Start the client and map any startup errors */
    client.start().await.map_err(|e| {
        let err_msg = format!("{:?}", e);
        error!("Client error: {}", err_msg);
        e.into()
    })
}
