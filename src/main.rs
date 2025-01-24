mod bot;

use bot::events::handlers::Handler;
use bot::utils::{config::Config, logger};
use serenity::prelude::*;
use serenity::Client;
use tokio;

/// Initializes the logger and configuration.
async fn initialize() -> Config {
    logger::log_debug("Initializing logger and configuration.");

    let config = Config::load_or_create_and_validate().expect("Failed to load or create config");

    logger::init_logger_with_config(&config);
    logger::log_debug("Logger initialized successfully.");

    config
}

/// Creates and returns a Serenity client.
async fn create_client(token: &str, intents: GatewayIntents) -> Client {
    logger::log_debug("Creating Serenity client.");

    Client::builder(token, intents)
        .event_handler(Handler)
        .await
        .expect("Error creating client")
}

#[tokio::main]
async fn main() {
    let config = initialize().await;

    let token = &config.red.token;
    logger::log_debug("Discord bot token retrieved.");

    let intents = GatewayIntents::GUILD_MESSAGES
        | GatewayIntents::DIRECT_MESSAGES
        | GatewayIntents::MESSAGE_CONTENT;
    logger::log_debug("Gateway intents configured.");

    let mut client = create_client(token, intents).await;
    logger::log_debug("Client successfully created.");

    if let Err(why) = client.start().await {
        logger::log_error(&format!("Client error: {:?}", why));
    }
}
