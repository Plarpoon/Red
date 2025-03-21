use async_trait::async_trait;
use log::warn;
use reqwest;
use songbird::events::{Event, EventContext, EventHandler as VoiceEventHandler, TrackEvent};
use songbird::{self, input::YoutubeDl};
use std::error::Error;

struct TrackErrorNotifier;

#[async_trait]
impl VoiceEventHandler for TrackErrorNotifier {
    async fn act(&self, ctx: &EventContext<'_>) -> Option<Event> {
        if let EventContext::Track(track_list) = ctx {
            for (state, handle) in *track_list {
                /* Log the error using the log crate instead of printing */
                warn!(
                    "Track {:?} encountered an error: {:?}",
                    handle.uuid(),
                    state.playing
                );
            }
        }
        None
    }
}

#[poise::command(
    slash_command,
    subcommands("join", "leave", "play", "mute", "unmute", "deafen", "undeafen"),
    description_localized("en-US", "Music related commands")
)]
pub async fn music(
    ctx: poise::Context<'_, (), Box<dyn Error + Send + Sync>>,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    /* Inform the user to use a subcommand */
    ctx.say("Please use a subcommand: join, leave, play, mute, unmute, deafen, or undeafen.")
        .await?;
    Ok(())
}

/* Join the voice channel the user is currently in */
#[poise::command(
    slash_command,
    guild_only,
    description_localized("en-US", "Join the voice channel you're currently in.")
)]
pub async fn join(
    ctx: poise::Context<'_, (), Box<dyn Error + Send + Sync>>,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    /* Retrieve guild ID or return an error message */
    let guild_id = ctx
        .guild_id()
        .ok_or("This command can only be used in a guild")?;

    /* Attempt to extract the user's voice channel from the cache */
    let cache = ctx.serenity_context().cache.clone();
    let channel_id = if let Some(id) = cache
        .guild(guild_id)
        .and_then(|guild| guild.voice_states.get(&ctx.author().id).cloned())
        .and_then(|vs| vs.channel_id)
    {
        id
    } else {
        ctx.say("You are not in a voice channel.").await?;
        return Ok(());
    };

    /* Get the Songbird manager; if unavailable, inform the user */
    let manager = if let Some(m) = songbird::get(ctx.serenity_context()).await {
        m.clone()
    } else {
        ctx.say("Songbird Voice client is not available.").await?;
        return Ok(());
    };

    /* Attempt to join the voice channel */
    match manager.join(guild_id, channel_id).await {
        Ok(handler_lock) => {
            let mut handler = handler_lock.lock().await;
            handler.add_global_event(TrackEvent::Error.into(), TrackErrorNotifier);
            ctx.say(&format!("Joined voice channel: {:?}", channel_id))
                .await?;
        }
        Err(e) => {
            ctx.say(&format!("Failed to join voice channel: {:?}", e))
                .await?;
        }
    }
    Ok(())
}

/* Leave the current voice channel */
#[poise::command(
    slash_command,
    guild_only,
    description_localized("en-US", "Leave the current voice channel.")
)]
pub async fn leave(
    ctx: poise::Context<'_, (), Box<dyn Error + Send + Sync>>,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    /* Retrieve guild ID or return an error message */
    let guild_id = ctx
        .guild_id()
        .ok_or("This command can only be used in a guild")?;

    /* Get the Songbird manager; if unavailable, inform the user */
    let manager = if let Some(m) = songbird::get(ctx.serenity_context()).await {
        m.clone()
    } else {
        ctx.say("Songbird Voice client is not available.").await?;
        return Ok(());
    };

    if manager.get(guild_id).is_none() {
        ctx.say("Not in a voice channel.").await?;
        return Ok(());
    }

    /* Attempt to leave the voice channel */
    match manager.remove(guild_id).await {
        Ok(_) => {
            ctx.say("Left the voice channel.").await?;
        }
        Err(e) => {
            ctx.say(&format!("Failed to leave voice channel: {:?}", e))
                .await?;
        }
    }
    Ok(())
}

/* Play a song from a URL or search query */
#[poise::command(
    slash_command,
    guild_only,
    description_localized("en-US", "Play a song from a provided URL or search query.")
)]
pub async fn play(
    ctx: poise::Context<'_, (), Box<dyn Error + Send + Sync>>,
    #[description_localized("en-US", "URL or search query")] url: String,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    /* Retrieve guild ID or return an error message */
    let guild_id = ctx
        .guild_id()
        .ok_or("This command can only be used in a guild")?;

    /* Determine if the URL should be treated as a search query */
    let do_search = !url.starts_with("http");
    let http_client = reqwest::Client::new();

    /* Get the Songbird manager; if unavailable, inform the user */
    let manager = if let Some(m) = songbird::get(ctx.serenity_context()).await {
        m.clone()
    } else {
        ctx.say("Songbird Voice client is not available.").await?;
        return Ok(());
    };

    /* Retrieve the voice channel handler */
    let handler_lock = if let Some(lock) = manager.get(guild_id) {
        lock
    } else {
        ctx.say("Not in a voice channel to play in.").await?;
        return Ok(());
    };

    let mut handler = handler_lock.lock().await;
    let src = if do_search {
        YoutubeDl::new_search(http_client, url)
    } else {
        YoutubeDl::new(http_client, url)
    };
    handler.play_input(src.into());
    ctx.say("Playing song.").await?;
    Ok(())
}

/* Mute the bot in the current voice channel */
#[poise::command(
    slash_command,
    guild_only,
    description_localized("en-US", "Mute the bot in the current voice channel.")
)]
pub async fn mute(
    ctx: poise::Context<'_, (), Box<dyn Error + Send + Sync>>,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    /* Retrieve guild ID or return an error message */
    let guild_id = ctx
        .guild_id()
        .ok_or("This command can only be used in a guild")?;

    /* Get the Songbird manager; if unavailable, inform the user */
    let manager = if let Some(m) = songbird::get(ctx.serenity_context()).await {
        m.clone()
    } else {
        ctx.say("Songbird Voice client is not available.").await?;
        return Ok(());
    };

    /* Retrieve the voice channel handler */
    let handler_lock = if let Some(lock) = manager.get(guild_id) {
        lock
    } else {
        ctx.say("Not in a voice channel.").await?;
        return Ok(());
    };

    let mut handler = handler_lock.lock().await;
    if handler.is_mute() {
        ctx.say("Already muted.").await?;
    } else if let Err(e) = handler.mute(true).await {
        ctx.say(&format!("Failed to mute: {:?}", e)).await?;
    } else {
        ctx.say("Muted.").await?;
    }
    Ok(())
}

/* Unmute the bot in the current voice channel */
#[poise::command(
    slash_command,
    guild_only,
    description_localized("en-US", "Unmute the bot in the current voice channel.")
)]
pub async fn unmute(
    ctx: poise::Context<'_, (), Box<dyn Error + Send + Sync>>,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    /* Retrieve guild ID or return an error message */
    let guild_id = ctx
        .guild_id()
        .ok_or("This command can only be used in a guild")?;

    /* Get the Songbird manager; if unavailable, inform the user */
    let manager = if let Some(m) = songbird::get(ctx.serenity_context()).await {
        m.clone()
    } else {
        ctx.say("Songbird Voice client is not available.").await?;
        return Ok(());
    };

    /* Retrieve the voice channel handler */
    let handler_lock = if let Some(lock) = manager.get(guild_id) {
        lock
    } else {
        ctx.say("Not in a voice channel.").await?;
        return Ok(());
    };

    let mut handler = handler_lock.lock().await;
    if !handler.is_mute() {
        ctx.say("Not muted.").await?;
    } else if let Err(e) = handler.mute(false).await {
        ctx.say(&format!("Failed to unmute: {:?}", e)).await?;
    } else {
        ctx.say("Unmuted.").await?;
    }
    Ok(())
}

/* Deafen the bot in the current voice channel */
#[poise::command(
    slash_command,
    guild_only,
    description_localized("en-US", "Deafen the bot in the current voice channel.")
)]
pub async fn deafen(
    ctx: poise::Context<'_, (), Box<dyn Error + Send + Sync>>,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    /* Retrieve guild ID or return an error message */
    let guild_id = ctx
        .guild_id()
        .ok_or("This command can only be used in a guild")?;

    /* Get the Songbird manager; if unavailable, inform the user */
    let manager = if let Some(m) = songbird::get(ctx.serenity_context()).await {
        m.clone()
    } else {
        ctx.say("Songbird Voice client is not available.").await?;
        return Ok(());
    };

    /* Retrieve the voice channel handler */
    let handler_lock = if let Some(lock) = manager.get(guild_id) {
        lock
    } else {
        ctx.say("Not in a voice channel.").await?;
        return Ok(());
    };

    let mut handler = handler_lock.lock().await;
    if handler.is_deaf() {
        ctx.say("Already deafened.").await?;
    } else if let Err(e) = handler.deafen(true).await {
        ctx.say(&format!("Failed to deafen: {:?}", e)).await?;
    } else {
        ctx.say("Deafened.").await?;
    }
    Ok(())
}

/* Undeafen the bot in the current voice channel */
#[poise::command(
    slash_command,
    guild_only,
    description_localized("en-US", "Undeafen the bot in the current voice channel.")
)]
pub async fn undeafen(
    ctx: poise::Context<'_, (), Box<dyn Error + Send + Sync>>,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    /* Retrieve guild ID or return an error message */
    let guild_id = ctx
        .guild_id()
        .ok_or("This command can only be used in a guild")?;

    /* Get the Songbird manager; if unavailable, inform the user */
    let manager = if let Some(m) = songbird::get(ctx.serenity_context()).await {
        m.clone()
    } else {
        ctx.say("Songbird Voice client is not available.").await?;
        return Ok(());
    };

    /* Retrieve the voice channel handler */
    let handler_lock = if let Some(lock) = manager.get(guild_id) {
        lock
    } else {
        ctx.say("Not in a voice channel.").await?;
        return Ok(());
    };

    let mut handler = handler_lock.lock().await;
    if let Err(e) = handler.deafen(false).await {
        ctx.say(&format!("Failed to undeafen: {:?}", e)).await?;
    } else {
        ctx.say("Undeafened.").await?;
    }
    Ok(())
}
