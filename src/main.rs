mod bot;

use bot::events::handlers::Handler;
use bot::utils::{config::Config, logger};
use serenity::prelude::*;
use serenity::Client;
use tokio;

#[tokio::main]
async fn main() {
    // Initialize logger
    logger::log_debug("Loading logger.");
    let config = Config::load_or_create_and_validate().expect("Failed to load or create config");
    logger::init_logger_with_config(&config);
    logger::log_debug("Logger initialized.");

    // Load the configuration file and ensure it is valid
    logger::log_debug("Loading configuration file.");
    let config =
        Config::load_or_create_and_validate().expect("Failed to load, create, or validate config");
    logger::log_debug("Configuration file loaded successfully.");

    let token = &config.red.token;
    logger::log_debug("Using the Discord bot token.");

    let intents = GatewayIntents::GUILD_MESSAGES
        | GatewayIntents::DIRECT_MESSAGES
        | GatewayIntents::MESSAGE_CONTENT;
    logger::log_debug("Gateway intents set.");

    let mut client = Client::builder(token, intents)
        .event_handler(Handler)
        .await
        .expect("Error creating client");
    logger::log_debug("Client created successfully.");

    if let Err(why) = client.start().await {
        logger::log_error(&format!("Client error: {:?}", why));
    }
}
