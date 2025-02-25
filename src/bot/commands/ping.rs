use log::{error, info};
use serenity::model::channel::Message;
use serenity::prelude::*;

/* Handles the `!ping` command. */
pub async fn handle_ping(ctx: &Context, msg: &Message) {
    if msg.content != "!ping" {
        return;
    }

    match msg.channel_id.say(&ctx.http, "Pong!").await {
        Ok(_) => info!("Successfully sent 'Pong!' in response to '!ping' command."),
        Err(why) => error!("Error sending message: {:?}", why),
    }
}
