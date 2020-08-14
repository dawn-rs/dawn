use futures::StreamExt;
use std::{env, error::Error};
use twilight_gateway::{shard::ShardConfig, Shard};
use twilight_model::gateway::GatewayIntents;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error + Send + Sync>> {
    // Initialize the tracing subscriber.
    tracing_subscriber::fmt::init();

    let config = ShardConfig::builder(env::var("DISCORD_TOKEN")?)
        .intents(Some(
            GatewayIntents::GUILD_MESSAGES | GatewayIntents::DIRECT_MESSAGES,
        ))
        .build();

    let mut shard = Shard::new(config);
    shard.start().await?;
    println!("Created shard");

    let mut events = shard.events();

    while let Some(event) = events.next().await {
        println!("Event: {:?}", event);
    }

    Ok(())
}
