mod bot;

use bot::events::handlers::Handler;
use bot::utils::config::Config;
use bot::utils::logger;

use log::{debug, error, info};
use serenity::prelude::*;
use serenity::Client;
use std::process;

#[tokio::main]
async fn main() {
    /* Asynchronously load and validate the configuration */
    let config = match Config::load_or_create_and_validate_async().await {
        Ok(cfg) => {
            debug!("Configuration loaded successfully.");
            cfg
        }
        Err(e) => {
            eprintln!("Error loading configuration: {:?}", e);
            process::exit(1);
        }
    };

    /* Initialize the logger using the configuration; exit if initialization fails */
    if let Err(e) = logger::init_logger(&config) {
        eprintln!("Failed to initialize logger: {:?}", e);
        process::exit(1);
    }

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
    let mut client = match Client::builder(token, intents).event_handler(Handler).await {
        Ok(client) => {
            info!("Client successfully created.");
            client
        }
        Err(e) => {
            error!("Error creating client: {:?}", e);
            process::exit(1);
        }
    };

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
