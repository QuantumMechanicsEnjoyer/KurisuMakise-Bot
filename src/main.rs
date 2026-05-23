use serenity::model::gateway::GatewayIntents;

mod commands;
mod types;
mod utilities;

struct Data {}
type Error = Box<dyn std::error::Error + Send + Sync>;

#[tokio::main]
async fn main() {
    if std::path::Path::new(".env").exists() {
        dotenv::dotenv().ok();
    }
    let token = std::env::var("BOT_TOKEN").expect("Expected a token in the environment");
    let intents = GatewayIntents::all();

    let framework = poise::Framework::<Data, Error>::builder()
        .options(poise::FrameworkOptions {
            commands: vec![
                commands::test(),
                commands::server_info(),
                // commands::upload(),
                commands::latex(),
            ],
            ..Default::default()
        })
        .setup(|ctx, _ready, framework| {
            Box::pin(async move {
                poise::builtins::register_globally(ctx, &framework.options().commands).await?;
                Ok(Data {})
            })
        })
        .build();

    let mut client = serenity::Client::builder(token, intents)
        .framework(framework)
        .await
        .expect("Error creating client");

    if let Err(why) = client.start().await {
        println!("Client error: {:?}", why);
    }
}
