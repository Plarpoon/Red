use reqwest::Client;
use songbird;

/* The parent music command. */
/* Use `/music` to see available subcommands. */
#[poise::command(slash_command, prefix_command, hide_in_help, subcommands("play"))]
pub async fn music(
    ctx: poise::Context<'_, (), Box<dyn std::error::Error + Send + Sync>>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    /* Extract the HTTP handle from the serenity context. */
    let http = ctx.serenity_context().http.clone();
    /* Respond using the HTTP handle and channel ID. */
    ctx.channel_id()
        .say(&http, "Available subcommands: play")
        .await?;
    Ok(())
}

/* Play sub-command. */
/* Usage: `/music play <url>` */
#[poise::command(
    slash_command,
    description_localized("en-US", "Play audio from a YouTube URL in your voice channel")
)]
pub async fn play(
    ctx: poise::Context<'_, (), Box<dyn std::error::Error + Send + Sync>>,
    #[description = "The YouTube URL of the video to play"] url: String,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    /* Extract and drop non-Send references as early as possible. */
    let author_id = ctx.author().id;
    let author_tag = ctx.author().tag();
    let (guild_id, channel_id) = {
        let guild = ctx
            .guild()
            .ok_or("This command can only be used in a guild")?;
        let guild_id = guild.id;
        let channel_id = guild
            .voice_states
            .get(&author_id)
            .and_then(|vs| vs.channel_id)
            .ok_or("You must be in a voice channel to use this command")?;
        (guild_id, channel_id)
    };
    let url_for_source = url.clone();

    /* Clone the serenity context to get owned, Send–compatible data. */
    let serenity_ctx = ctx.serenity_context().clone();
    let http = serenity_ctx.http.clone();

    /* Now perform async operations without holding non-Send references. */
    let manager = songbird::get(&serenity_ctx)
        .await
        .expect("Songbird voice client should be initialized")
        .clone();
    let handler_lock = manager.join(guild_id, channel_id).await?;

    let client = Client::new();
    let source = songbird::input::YoutubeDl::new_ytdl_like("yt-dlp", client, url_for_source).into();
    {
        /* Lock the handler and play the audio source. */
        let mut handler = handler_lock.lock().await;
        handler.play_input(source);
    }

    /* Send a confirmation message using the cloned HTTP handle. */
    channel_id
        .say(&http, "Now playing audio from the provided YouTube URL.")
        .await?;

    /* Log command execution details. */
    log::info!(
        "Play command invoked by {} in voice channel {} with URL {}",
        author_tag,
        channel_id,
        url
    );

    Ok(())
}
