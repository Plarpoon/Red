/* Define a type alias for errors using a boxed error trait object */
pub type Error = Box<dyn std::error::Error + Send + Sync>;

/* Create an alias for the Poise command context with unit type as shared data */
pub type CommandContext<'a> = poise::Context<'a, (), Error>;

/* Define the ping command as both a prefix and a slash command */
#[poise::command(prefix_command, slash_command)]
pub async fn ping(ctx: CommandContext<'_>) -> Result<(), Error> {
    /* Record the current time before sending the message */
    let start_time = std::time::Instant::now();

    /* Send an initial message which will later be edited with the latency embed */
    let sent_message = ctx.say("Calculating latency...").await?;

    /* Calculate the elapsed time in milliseconds */
    let latency_ms = start_time.elapsed().as_millis();

    /* Edit the message to display an embed containing the latency and its explanation, with the :ping_pong: emoji added to the title */
    sent_message
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

    /* Log the successful execution of the ping command with the latency value */
    log::info!("Ping command responded with {}ms", latency_ms);

    Ok(())
}
