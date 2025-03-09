use log::info;
use poise::serenity_prelude::{
    Context as SerenityContext, EventHandler, Message, Ready, async_trait,
};

pub struct Handler;

#[async_trait]
impl EventHandler for Handler {
    /* Called when the bot connects successfully */
    async fn ready(&self, _ctx: SerenityContext, ready: Ready) {
        info!("{} is connected!", ready.user.name);
    }

    /* Called when a new message is received */
    async fn message(&self, _ctx: SerenityContext, _msg: Message) {
        /* Poise automatically handles commands. */
    }
}
