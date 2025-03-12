use crate::bot::commands::commands_list;
use crate::bot::utils::config::Config;
use log::{info, warn};
use poise::serenity_prelude as serenity;

/* Registers commands based on the current configuration */
pub async fn register_commands(
    http: &serenity::Http,
    config: &Config,
    guild_id: Option<u64>,
) -> Result<(), Box<dyn std::error::Error>> {
    if config.debug.enable_debug {
        /* In debug mode, register commands for the debug guild */
        let gid = guild_id.expect("Debug guild_id should be available in debug mode");
        return register_debug_commands(http, gid).await;
    }
    /* In production mode, register commands globally */
    register_global_commands(http).await
}

/* Registers commands in debug mode by purging global and guild commands, then re-registering guild commands */
async fn register_debug_commands(
    http: &serenity::Http,
    guild_id: u64,
) -> Result<(), Box<dyn std::error::Error>> {
    /* Debug mode: Purge global commands */
    info!("Debug mode enabled.");
    warn!("Purging all global commands.");
    let global_commands = http.get_global_commands().await?;
    for command in global_commands {
        match http.delete_global_command(command.id).await {
            Ok(_) => info!("Deleted global command: {}", command.name),
            Err(e) => warn!("Failed to delete global command {}: {:?}", command.name, e),
        }
    }

    /* Purge guild commands */
    warn!("Purging existing guild commands for guild: {}", guild_id);
    let guild = serenity::GuildId::new(guild_id);
    let existing_commands = guild.get_commands(http).await?;
    for command in existing_commands {
        match guild.delete_command(http, command.id).await {
            Ok(_) => warn!("Deleted command: {}", command.name),
            Err(e) => warn!("Failed to delete command {}: {:?}", command.name, e),
        }
    }

    /* Register updated commands for the guild */
    let commands = commands_list::get_commands().await;
    let command_data = poise::builtins::create_application_commands(&commands);
    guild.set_commands(http, command_data).await?;
    info!("Re-registered updated commands for guild: {}", guild_id);
    Ok(())
}

/* Registers commands globally in production mode */
async fn register_global_commands(http: &serenity::Http) -> Result<(), Box<dyn std::error::Error>> {
    info!("Production mode enabled.");
    warn!("Registering global commands.");
    let commands = commands_list::get_commands().await;
    let command_data = poise::builtins::create_application_commands(&commands);
    http.create_global_command(&command_data).await?;
    info!("Registered global commands.");
    Ok(())
}
