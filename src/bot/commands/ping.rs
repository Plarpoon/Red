use log::{error, info};
use serenity::model::channel::Message;
use serenity::prelude::*;

/// Handles the `!ping` command.
pub async fn handle_ping(ctx: &Context, msg: &Message) {
    if msg.content == "!ping" {
        if let Err(why) = msg.channel_id.say(&ctx.http, "Pong!").await {
            /* Log an error if sending the message fails */
            error!("Error sending message: {:?}", why);
        } else {
            /* Log a success message if 'Pong!' was sent */
            info!("Successfully sent 'Pong!' in response to '!ping' command.");
        }
    }
}
