use std::{env, error::Error};
use twilight_http::Client;
use twilight_model::{
    channel::message::allowed_mentions::{AllowedMentions, AllowedMentionsBuilder},
    id::{ChannelId, UserId},
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error + Send + Sync>> {
    // Initialize the tracing subscriber.
    tracing_subscriber::fmt::init();

    //if we want to set the default for allowed mentions we need to use the builder, keep in mind these calls can't be chained!
    let client = Client::builder()
        .token(env::var("DISCORD_TOKEN")?)
        //add an empty allowed mentions, this will prevent any and all pings
        .default_allowed_mentions(AllowedMentions::default())
        .build();
    let channel_id = ChannelId(381_926_291_785_383_946);
    let user_id = UserId(77_469_400_222_932_992);

    //here we want to warn a user about trying to ping everyone so we override to allow pinging them
    //but since we did not allow @everyone pings it will not ping everyone
    client
        .create_message(channel_id)
        .content(format!(
            "<@{}> you are not allowed to ping @everyone!",
            user_id.0
        ))?
        .allowed_mentions(
            AllowedMentionsBuilder::new()
                .user_ids(vec![user_id])
                .build(),
        )
        .exec()
        .await?;

    Ok(())
}
