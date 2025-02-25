mod bot;

use bot::events::handlers::Handler;
use bot::utils::config::Config;
use bot::utils::logger;

use log::{error, info, warn};
use serenity::prelude::*;
use serenity::Client;
use std::process;

#[tokio::main]
async fn main() {
    /* Load and validate the configuration or exit if an error occurs */
    let config = Config::load_or_create_and_validate_async()
        .await
        .unwrap_or_else(|e| {
            warn!("Failed to load configuration: {:?}", e);
            eprintln!("{:?}", e);
            process::exit(1);
        });

    /* Initialize the logger using the configuration */
    logger::init_logger(&config).unwrap_or_else(|e| {
        eprintln!("Failed to initialize logger: {:?}", e);
        process::exit(1);
    });

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
        .await
        .unwrap_or_else(|e| {
            error!("Error creating client: {:?}", e);
            process::exit(1);
        });

    /* Start the client and log any errors that occur */
    if let Err(why) = client.start().await {
        let error_msg = format!("{:?}", why);
        if error_msg.contains("connection refused")
            || error_msg.contains("network unreachable")
            || error_msg.contains("timed out")
        {
            error!("Discord servers appear to be offline or unreachable! Critical error.");
        }
        error!("Client error: {}", error_msg);
    }
}
