use crate::bot::commands::ping;

use log::info;
use serenity::{
    async_trait,
    model::{channel::Message, gateway::Ready},
    prelude::*,
};

pub struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn ready(&self, _ctx: Context, ready: Ready) {
        /* Log that the bot is connected */
        info!("{} is connected!", ready.user.name);
    }

    async fn message(&self, ctx: Context, msg: Message) {
        /* Delegate the `!ping` command handling to the appropriate function */
        ping::handle_ping(&ctx, &msg).await;
    }
}
