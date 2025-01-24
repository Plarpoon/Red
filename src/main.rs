mod bot;

use bot::events::handlers::Handler;
use bot::utils::{config::Config, logger};
use serenity::prelude::*;
use serenity::Client;

#[tokio::main]
async fn main() {
    logger::log_trace("Starting bot.");

    // --- 1. Load config ---
    logger::log_debug("Initializing logger and configuration.");
    let config = Config::load_or_create_and_validate_async()
        .await
        .expect("Failed to load or create config");

    // --- 2. Init tracing-based logger ---
    let _guard = logger::init_logger_with_config(&config)
        .await
        .expect("Failed to init async logger");
    logger::log_debug("Logger initialized successfully.");

    // --- 3. Set up client ---
    let token = &config.red.token;
    logger::log_debug("Discord bot token retrieved.");

    let intents = GatewayIntents::GUILD_MESSAGES
        | GatewayIntents::DIRECT_MESSAGES
        | GatewayIntents::MESSAGE_CONTENT;
    logger::log_debug("Gateway intents configured.");

    let mut client = Client::builder(token, intents)
        .event_handler(Handler)
        .await
        .expect("Error creating client");
    logger::log_debug("Client successfully created.");

    // --- 4. Start the bot ---
    if let Err(why) = client.start().await {
        // Convert the error to a String for easy inspection
        let error_msg = format!("{:?}", why);

        // Check for possible network/offline indicators
        if error_msg.contains("connection refused")
            || error_msg.contains("network unreachable")
            || error_msg.contains("timed out")
        {
            logger::log_critical(
                "Discord servers appear to be offline or unreachable! Critical error.",
            );
        }

        logger::log_error(&format!("Client error: {error_msg}"));
    }
}
