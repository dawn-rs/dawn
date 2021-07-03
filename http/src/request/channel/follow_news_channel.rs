use crate::{client::Client, request::Request, response::ResponseFuture, routing::Route};
use serde::Serialize;
use twilight_model::{channel::FollowedChannel, id::ChannelId};

#[derive(Default, Serialize)]
struct FollowNewsChannelFields {
    webhook_channel_id: ChannelId,
}

/// Follow a news channel by [`ChannelId`]s.
pub struct FollowNewsChannel<'a> {
    channel_id: ChannelId,
    fields: FollowNewsChannelFields,
    http: &'a Client,
}

impl<'a> FollowNewsChannel<'a> {
    pub(crate) const fn new(
        http: &'a Client,
        channel_id: ChannelId,
        webhook_channel_id: ChannelId,
    ) -> Self {
        Self {
            channel_id,
            http,
            fields: FollowNewsChannelFields { webhook_channel_id },
        }
    }

    /// Execute the request, returning a future resolving to a [`Response`].
    ///
    /// [`Response`]: crate::response::Response
    pub fn exec(self) -> ResponseFuture<FollowedChannel> {
        let mut request = Request::builder(Route::FollowNewsChannel {
            channel_id: self.channel_id.0,
        });

        request = match request.json(&self.fields) {
            Ok(request) => request,
            Err(source) => return ResponseFuture::error(source),
        };

        self.http.request(request.build())
    }
}
