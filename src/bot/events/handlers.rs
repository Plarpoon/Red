use crate::bot::commands::ping;
use crate::bot::utils::logger;
use serenity::async_trait;
use serenity::model::channel::Message;
use serenity::model::gateway::Ready;
use serenity::prelude::*;

pub struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn ready(&self, _ctx: Context, ready: Ready) {
        logger::log_info(&format!("{} is connected!", ready.user.name));
    }

    async fn message(&self, ctx: Context, msg: Message) {
        // Delegate the `!ping` command handling to the appropriate function
        ping::handle_ping(&ctx, &msg).await;
    }
}
