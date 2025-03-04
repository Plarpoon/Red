mod bot;
use bot::events::handlers::Handler;
use bot::utils::config::Config;
use bot::utils::logger;

use log::{error, info};
use serenity::{Client, prelude::GatewayIntents};
use std::process;

/* The main asynchronous entry point of the application */
#[tokio::main]
async fn main() {
    /* Run the application and exit on error */
    if let Err(err) = run().await {
        error!("Application error: {:?}", err);
        process::exit(1);
    }
}

/* Runs the bot and propagates errors using Result */
async fn run() -> Result<(), Box<dyn std::error::Error>> {
    /* Load and validate the configuration */
    let config = Config::load_or_create_and_validate_async().await?;

    /* Initialize the logger using the configuration */
    logger::init_logger_with_config(&config).await?;
    info!("Starting bot.");

    /* Retrieve the Discord bot token from the configuration */
    let token = &config.red.token;
    info!("Discord bot token retrieved.");

    /* Configure gateway intents */
    let intents = GatewayIntents::GUILD_MESSAGES
        | GatewayIntents::DIRECT_MESSAGES
        | GatewayIntents::MESSAGE_CONTENT;
    info!("Gateway intents configured.");

    /* Build the Discord client with the event handler */
    let mut client = Client::builder(token, intents)
        .event_handler(Handler)
        .await?;

    /* Start the client and handle connection-related errors */
    client.start().await.map_err(|e| {
        let err_msg = format!("{:?}", e);
        if err_msg.contains("connection refused")
            || err_msg.contains("network unreachable")
            || err_msg.contains("timed out")
        {
            error!("Discord servers appear to be offline or unreachable! Critical error.");
        }
        error!("Client error: {}", err_msg);
        e.into()
    })
}
