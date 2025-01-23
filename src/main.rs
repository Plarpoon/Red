mod bot;

use bot::events::handlers::Handler;
use red::config::Config;
use serenity::prelude::*;
use serenity::Client;
use tokio;

#[tokio::main]
async fn main() {
    // Load the configuration file and ensure it is valid.
    let config =
        Config::load_or_create_and_validate().expect("Failed to load, create, or validate config");

    let token = config.red.token;

    let intents = GatewayIntents::GUILD_MESSAGES
        | GatewayIntents::DIRECT_MESSAGES
        | GatewayIntents::MESSAGE_CONTENT;

    let mut client = Client::builder(&token, intents)
        .event_handler(Handler)
        .await
        .expect("Error creating client");

    if let Err(why) = client.start().await {
        println!("Client error: {:?}", why);
    }
}
