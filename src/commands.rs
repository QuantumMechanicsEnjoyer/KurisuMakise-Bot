use poise::{CreateReply, command};
use serenity::{builder::{CreateAttachment, CreateEmbed}, model::Color};

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
    let reply = CreateReply::default()
        .attachment(attachment)
        .embed(embed);

    ctx.send(reply).await?;

    Ok(())
}
