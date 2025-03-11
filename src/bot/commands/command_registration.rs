use crate::bot::commands::commands_list;
use crate::bot::utils::config::Config;
use log::{info, warn};
use poise::serenity_prelude as serenity;

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

async fn register_debug_commands(
    http: &serenity::Http,
    guild_id: u64,
) -> Result<(), Box<dyn std::error::Error>> {
    /* Debug mode: Purge global commands and then re-register guild commands. */
    info!("Debug mode enabled.");
    warn!("Purging all global commands.");
    let global_commands = http.get_global_commands().await?;
    for command in global_commands {
        if let Err(e) = http.delete_global_command(command.id).await {
            warn!("Failed to delete global command {}: {:?}", command.name, e);
        } else {
            info!("Deleted global command: {}", command.name);
        }
    }

    warn!("Purging existing guild commands for guild: {}", guild_id);
    let guild = serenity::GuildId::new(guild_id);
    let existing_commands = guild.get_commands(http).await?;
    for command in existing_commands {
        if let Err(e) = guild.delete_command(http, command.id).await {
            warn!("Failed to delete command {}: {:?}", command.name, e);
        } else {
            warn!("Deleted command: {}", command.name);
        }
    }

    let commands = commands_list::get_commands().await;
    let command_data = poise::builtins::create_application_commands(&commands);
    guild.set_commands(http, command_data).await?;
    info!("Re-registered updated commands for guild: {}", guild_id);
    Ok(())
}

async fn register_global_commands(http: &serenity::Http) -> Result<(), Box<dyn std::error::Error>> {
    /* Production mode: Register global commands. */
    info!("Production mode enabled.");
    warn!("Registering global commands.");
    let commands = commands_list::get_commands().await;
    let command_data = poise::builtins::create_application_commands(&commands);
    http.create_global_command(&command_data).await?;
    info!("Registered global commands.");
    Ok(())
}
