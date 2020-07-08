<!-- cargo-sync-readme start -->

# twilight-standby

Standby is a utility to wait for an event to happen based on a predicate
check. For example, you may have a command that has a reaction menu of ✅ and
❌. If you want to handle a reaction to these, using something like an
application-level state or event stream may not suit your use case. It may
be cleaner to wait for a reaction inline to your function. This is where
Twilight Standby comes in.

Standby allows you to wait for things like an event in a certain guild
([`Standby::wait_for`]), a new message in a channel
([`Standby::wait_for_message`]), a new reaction on a message
([`Standby::wait_for_reaction`]), and any event that might not take place in
a guild, such as a new `Ready` event ([`Standby::wait_for_event`]). Each
method also has a stream variant.

To use Standby, you must process events with it in your main event loop.
Check out the [`Standby::process`] method.

## When to use futures and streams

`Standby` has two variants of each method: a future variant and a stream
variant. An example is [`Standby::wait_for_message`], which also has a
[`Standby::wait_for_message_stream`] variant. The future variant is useful
when you want to oneshot an event that you need to wait for. This means that
if you only need to wait for one message in a channel to come in, you'd use
the future variant. If you need to wait for multiple messages, such as maybe
all of the messages within a minute's timespan, you'd use the
[`Standby::wait_for_message_stream`] method.

The difference is that if you use the futures variant in a loop then you may
miss some events while processing a received event. By using a stream, you
won't miss any events.

## Examples

### At a glance

Wait for a message in channel 123 by user 456 with the content "test":

```rust,no_run
use twilight_model::{gateway::payload::MessageCreate, id::{ChannelId, UserId}};
use twilight_standby::Standby;

let standby = Standby::new();

let message = standby.wait_for_message(ChannelId(123), |event: &MessageCreate| {
    event.author.id == UserId(456) && event.content == "test"
}).await?;
```

### A full example

A full sample bot connecting to the gateway, processing events, and
including a handler to wait for reactions:

```rust,no_run
use futures_util::StreamExt;
use std::{env, error::Error};
use twilight_gateway::{Event, Shard};
use twilight_model::{
    channel::Message,
    gateway::payload::ReactionAdd,
    id::{ChannelId, UserId},
};
use twilight_standby::Standby;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // Start a shard connected to the gateway to receive events.
    let mut shard = Shard::new(env::var("DISCORD_TOKEN")?);
    let mut events = shard.events().await;
    shard.start().await?;

    let standby = Standby::new();

    while let Some(event) = events.next().await {
        // Have standby process the event, which will fulfill any futures that
        // are waiting for an event.
        standby.process(&event);

        match event {
            Event::MessageCreate(msg) if msg.content == "!react" => {
                tokio::spawn(react(msg.0, standby.clone()));
            },
            _ => {},
        }
    }

    Ok(())
}

// Wait for a reaction from the user who sent the message, and then print it
// once they react.
async fn react(msg: Message, standby: Standby) -> Result<(), Box<dyn Error + Send + Sync + 'static>> {
    let author_id = msg.author.id;

    let reaction = standby.wait_for_reaction(msg.id, move |event: &ReactionAdd| {
        event.user_id == author_id
    }).await?;

    println!("user reacted with {:?}", reaction.emoji);

    Ok(())
}
```

For more examples, check out each of the methods on [`Standby`].

[`Standby`]: struct.Standby.html
[`Standby::process`]: struct.Standby.html#method.process
[`Standby::wait_for`]: struct.Standby.html#method.wait_for
[`Standby::wait_for_event`]: struct.Standby.html#method.wait_for_event
[`Standby::wait_for_message`]: struct.Standby.html#method.wait_for_message
[`Standby::wait_for_message_stream`]: struct.Standby.html#method.wait_for_message_stream
[`Standby::wait_for_reaction`]: struct.Standby.html#method.wait_for_reaction

<!-- cargo-sync-readme end -->
