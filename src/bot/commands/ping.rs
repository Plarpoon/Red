use log::{error, info};
use serenity::{model::channel::Message, prelude::*};

/* Handles the `!ping` command. */
pub async fn handle_ping(ctx: &Context, msg: &Message) {
    if msg.content != "!ping" {
        return;
    }

    if let Err(why) = msg.channel_id.say(&ctx.http, "Pong!").await {
        error!("Error sending message: {:?}", why);
    } else {
        info!("Successfully sent 'Pong!' in response to '!ping' command.");
    }
}
