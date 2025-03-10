#[poise::command(
    slash_command,
    description_localized("en-US", "Ping the bot to calculate latency to Discord's API.")
)]

pub async fn ping(
    ctx: poise::Context<'_, (), Box<dyn std::error::Error + Send + Sync>>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // Record the current time before sending the message
    let start_time = std::time::Instant::now();

    /* Send an initial message which will later be edited with the latency embed */
    let message = ctx.say("Calculating latency...").await?;

    /* Calculate the elapsed time in milliseconds */
    let latency_ms = start_time.elapsed().as_millis();

    /* Edit the message to display an embed containing the latency and its explanation */
    message
        .edit(
            ctx,
            poise::CreateReply::default().embed(
                poise::serenity_prelude::CreateEmbed::default()
                    .title("Pong! :ping_pong:")
                    .description(format!("Latency: {}ms", latency_ms))
                    .field(
                        "Explanation",
                        "Latency is measured as the round-trip time to Discord's API.",
                        false,
                    ),
            ),
        )
        .await?;

    /* Retrieve the invoking user's tag */
    let username = ctx.author().tag();

    /* Retrieve guild channel once to reduce repetition */
    let guild_channel = ctx.guild_channel().await;

    /* Get the channel name or "DM" if not in a guild */
    let channel_name = guild_channel
        .as_ref()
        .map(|gc| gc.name.clone())
        .unwrap_or_else(|| "DM".to_string());

    /* Get the guild name or "DM" if not in a guild channel */
    let guild_name = if let Some(gc) = guild_channel {
        gc.guild_id
            .to_partial_guild(&ctx.serenity_context().http)
            .await
            .map(|g| g.name)
            .unwrap_or_else(|_| "Unknown".to_string())
    } else {
        "DM".to_string()
    };

    /* Log the execution details including the username, channel, and guild */
    log::info!(
        "Ping command by {} in channel '{}' of guild '{}' responded with {}ms",
        username,
        channel_name,
        guild_name,
        latency_ms
    );

    Ok(())
}
