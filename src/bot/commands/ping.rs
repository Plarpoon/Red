use crate::bot::utils::logger;
use serenity::model::channel::Message;
use serenity::prelude::*;

/// Handles the `!ping` command.
pub async fn handle_ping(ctx: &Context, msg: &Message) {
    if msg.content == "!ping" {
        if let Err(why) = msg.channel_id.say(&ctx.http, "Pong!").await {
            logger::log_error(&format!("Error sending message: {why:?}"));
        } else {
            logger::log_info("Successfully sent 'Pong!' in response to '!ping' command.");
        }
    }
}
