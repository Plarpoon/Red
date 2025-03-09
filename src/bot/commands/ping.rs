/* Define a type alias for errors using a boxed error trait object */
pub type Error = Box<dyn std::error::Error + Send + Sync>;

/* Create an alias for the Poise command context with unit type as shared data */
pub type CommandContext<'a> = poise::Context<'a, (), Error>;

/* Define the ping command as both a prefix and a slash command */
#[poise::command(prefix_command, slash_command)]
pub async fn ping(ctx: CommandContext<'_>) -> Result<(), Error> {
    /* Respond with "Pong!" asynchronously */
    ctx.say("Pong!").await?;
    /* Log the successful execution of the ping command */
    log::info!("Successfully responded with 'Pong!'");
    Ok(())
}
