use poise::serenity_prelude as serenity;

/* Retrieve the application ID from Discord using Serenity's HTTP client */
pub async fn get_application_id(
    http: &serenity::Http,
) -> Result<serenity::ApplicationId, Box<dyn std::error::Error>> {
    let app_info = http.get_current_application_info().await?;
    Ok(app_info.id)
}
