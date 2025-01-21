use serenity::model::channel::Message;
use serenity::prelude::*;

/// Handles the `!ping` command.
pub async fn handle_ping(ctx: &Context, msg: &Message) {
    if msg.content == "!ping" {
        if let Err(why) = msg.channel_id.say(&ctx.http, "Pong!").await {
            println!("Error sending message: {why:?}");
        }
    }
}
