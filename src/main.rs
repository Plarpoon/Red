mod bot;

use bot::events::handlers::Handler;
use red::config::Config;
use serenity::prelude::*;
use serenity::Client;
use tokio;

#[tokio::main]
async fn main() {
    // Load the configuration file
    let config = Config::load_or_create("placeholder_token".to_string(), 1)
        .expect("Failed to load or create config");

    let token = if config.red.token == "placeholder_token" {
        eprintln!(
            "No valid token found! Please update 'config.toml' and replace 'placeholder_token' with your actual Discord token."
        );
        std::process::exit(1);
    } else {
        config.red.token
    };

    let intents = GatewayIntents::GUILD_MESSAGES
        | GatewayIntents::DIRECT_MESSAGES
        | GatewayIntents::MESSAGE_CONTENT;

    let mut client = Client::builder(&token, intents)
        .event_handler(Handler)
        .await
        .expect("Err creating client");

    if let Err(why) = client.start().await {
        println!("Client error: {:?}", why);
    }
}
