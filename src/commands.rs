use futures::Stream;
use poise::{CreateReply, command};
use serenity::{
    builder::{CreateAttachment, CreateEmbed},
    model::Color,
};

use crate::types::{Context, Error};

#[command(
    slash_command,
    description_localized("en-US", "A simple test command that greets the user")
)]
pub async fn test(ctx: Context<'_>) -> Result<(), Error> {
    let response = format!("Hello, {}!", ctx.author().name);
    ctx.say(response).await?;
    Ok(())
}

#[command(
    slash_command,
    description_localized("en-US", "Get information about the server")
)]
pub async fn server_info(ctx: Context<'_>) -> Result<(), Error> {
    let system = sysinfo::System::new_all();
    let embed = serenity::builder::CreateEmbed::default()
        .title("Server Information")
        .field(
            "Total Memory",
            crate::utilities::format_memory(system.total_memory()),
            true,
        )
        .field(
            "Used Memory",
            format!(
                "{} ``({:.2}%)``",
                crate::utilities::format_memory(system.used_memory()),
                system.used_memory() as f64 / system.total_memory() as f64 * 100.
            ),
            true,
        )
        .field(
            "Used Swap",
            format!("{:.2}%", system.used_swap() / system.free_swap() * 100),
            true,
        )
        .field(
            "Processor Name",
            format!("``{}``", system.cpus()[0].brand()),
            true,
        )
        .field(
            "CPU Usage",
            format!("{:.2}%", system.global_cpu_usage()),
            true,
        )
        .color(Color::from_rgb(0x5f, 0x92, 0xfd));

    let message = CreateReply::default().embed(embed);

    ctx.send(message).await?;
    Ok(())
}

#[command(
    slash_command,
    description_localized("en-US", "Upload an attachment and save it to the server")
)]
pub async fn upload(
    ctx: Context<'_>,
    attachment: poise::serenity_prelude::Attachment,
) -> Result<(), Error> {
    ctx.defer_ephemeral().await?;
    let content = attachment.download().await?;
    let write_result = crate::utilities::write(&attachment.filename, &content).await;

    match write_result {
        Ok(_) => {
            let reply = CreateReply::default()
                .content(format!(
                    "File `{}` saved successfully!",
                    attachment.filename
                ))
                .ephemeral(true);
            ctx.send(reply).await?;
            Ok(())
        }
        Err(e) => {
            let reply = CreateReply::default()
                .content(format!("Failed to save file: {}", e))
                .ephemeral(true);
            ctx.send(reply).await?;
            return Ok(());
        }
    }
}

#[command(
    slash_command,
    description_localized("en-US", "Render LaTeX code and return it as an image")
)]
pub async fn latex(ctx: Context<'_>, code: String) -> Result<(), Error> {
    ctx.defer().await?;

    let latex_image = crate::utilities::generate_latex_image(&code).await?;
    let attachment = CreateAttachment::bytes(latex_image, "latex.png");
    let embed = CreateEmbed::default()
        .image("attachment://latex.png")
        .color(Color::from_rgb(0x5f, 0x92, 0xfd));
    let reply = CreateReply::default().attachment(attachment).embed(embed);

    ctx.send(reply).await?;

    Ok(())
}

#[command(
    slash_command,
    description_localized("en-US", "Save a URL to the database")
)]
pub async fn save_url(
    ctx: Context<'_>,
    url: String,
    description: Option<String>,
) -> Result<(), Error> {
    ctx.defer_ephemeral().await?;
    let data = ctx.data();
    let database = &data.database;
    let result = sqlx::query!(
        "INSERT INTO urls (discord_id, url, description, created_at) VALUES (?, ?, ?, ?)",
        ctx.author().id.get() as i64,
        url,
        description.unwrap_or("None".to_string()),
        chrono::Utc::now().timestamp()
    )
    .execute(database.get_connection())
    .await?;
    let reply = match result.rows_affected() {
        1 => CreateReply::default()
            .content("URL saved successfully!")
            .ephemeral(true),
        _ => CreateReply::default()
            .content("Failed to save URL.")
            .ephemeral(true),
    };
    ctx.send(reply).await?;
    Ok(())
}

#[command(
    slash_command,
    description_localized("en-US", "List all saved URLs for the user")
)]
pub async fn url_list(ctx: Context<'_>) -> Result<(), Error> {
    ctx.defer_ephemeral().await?;
    let data = ctx.data();
    let database = &data.database;
    let rows = sqlx::query!(
        "SELECT id, url, description FROM urls WHERE discord_id = ?",
        ctx.author().id.get() as i64
    )
    .fetch_all(database.get_connection())
    .await?;

    if rows.is_empty() {
        let reply = CreateReply::default()
            .content("You have no saved URLs.")
            .ephemeral(true);
        ctx.send(reply).await?;
        return Ok(());
    }

    let mut embed = CreateEmbed::default()
        .title("Your Saved URLs")
        .color(Color::from_rgb(0x5f, 0x92, 0xfd));

    for row in rows {
        embed = embed.field(
            "",
            format!(
                "``{}`` {} - ``{}``",
                row.id,
                row.url,
                row.description.unwrap_or_default()
            ),
            false,
        );
    }

    let reply = CreateReply::default().embed(embed).ephemeral(true);
    ctx.send(reply).await?;
    Ok(())
}

#[derive(poise::ChoiceParameter)]
pub enum ManageUrlAction {
    #[name = "delete"]
    Delete,
    #[name = "update"]
    Update,
}

async fn autocomplete_url_id(ctx: Context<'_>, partial: &str) -> impl Stream<Item = String> {
    let data = ctx.data();
    let database = &data.database;
    let rows = sqlx::query!(
        "SELECT id FROM urls WHERE discord_id = ? AND CAST(id AS TEXT) LIKE ? LIMIT 25",
        ctx.author().id.get() as i64,
        format!("%{}%", partial)
    )
    .fetch_all(database.get_connection())
    .await
    .unwrap_or_default();
    // println!("Autocomplete query for partial '{}', found {} matches", partial, rows.len());
    futures::stream::iter(rows.into_iter().map(|row| row.id.to_string()))
}

#[command(slash_command, description_localized("en-US", "Manage your saved URL"))]
pub async fn manage_url(
    ctx: Context<'_>,
    #[autocomplete = "autocomplete_url_id"] id: String,
    action: Option<ManageUrlAction>,
    new_description: Option<String>,
) -> Result<(), Error> {
    ctx.defer_ephemeral().await?;
    let data = ctx.data();
    let database = &data.database;

    match action {
        Some(ManageUrlAction::Delete) => {
            let result = sqlx::query!(
                "DELETE FROM urls WHERE id = ? AND discord_id = ?",
                id,
                ctx.author().id.get() as i64
            )
            .execute(database.get_connection())
            .await?;
            let reply = if result.rows_affected() == 1 {
                CreateReply::default()
                    .content("URL deleted successfully!")
                    .ephemeral(true)
            } else {
                CreateReply::default()
                    .content(
                        "Failed to delete URL. Make sure the ID is correct and belongs to you.",
                    )
                    .ephemeral(true)
            };
            ctx.send(reply).await?;
        }
        Some(ManageUrlAction::Update) => {
            if let Some(desc) = new_description {
                let result = sqlx::query!(
                    "UPDATE urls SET description = ? WHERE id = ? AND discord_id = ?",
                    desc,
                    id,
                    ctx.author().id.get() as i64
                )
                .execute(database.get_connection())
                .await?;
                let reply = if result.rows_affected() == 1 {
                    CreateReply::default()
                        .content("URL description updated successfully!")
                        .ephemeral(true)
                } else {
                    CreateReply::default()
                        .content(
                            "Failed to update URL. Make sure the ID is correct and belongs to you.",
                        )
                        .ephemeral(true)
                };
                ctx.send(reply).await?;
            } else {
                let reply = CreateReply::default()
                    .content("Please provide a new description for the update action.")
                    .ephemeral(true);
                ctx.send(reply).await?;
            }
        }
        None => {
            let reply = CreateReply::default()
                .content("Please specify an action (delete or update).")
                .ephemeral(true);
            ctx.send(reply).await?;
        }
    }

    Ok(())
}
