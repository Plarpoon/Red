mod bot;

use bot::events::handlers::Handler;
use bot::utils::{config::Config, logger};
use serenity::prelude::*;
use serenity::Client;

#[tokio::main]
async fn main() {
    // --- 1. Load config (async) ---
    logger::log_debug("Initializing logger and configuration.");

    let config = Config::load_or_create_and_validate_async()
        .await
        .expect("Failed to load or create config");

    // --- 2. Init tracing-based logger (async) ---
    // The returned guard ensures logs are flushed before exit
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
        logger::log_error(&format!("Client error: {:?}", why));
    }
}
