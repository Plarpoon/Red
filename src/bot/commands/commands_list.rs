use crate::bot::commands::{music, ping};

/* Returns a vector of commands to register */
pub async fn get_commands() -> Vec<poise::Command<(), Box<dyn std::error::Error + Send + Sync>>> {
    vec![ping::ping(), music::music()]
}
