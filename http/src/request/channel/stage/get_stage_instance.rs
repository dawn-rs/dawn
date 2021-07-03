use crate::{client::Client, request::Request, response::ResponseFuture, routing::Route};
use twilight_model::{channel::StageInstance, id::ChannelId};

/// Gets the stage instance associated with a stage channel, if it exists.
pub struct GetStageInstance<'a> {
    channel_id: ChannelId,
    http: &'a Client,
}

impl<'a> GetStageInstance<'a> {
    pub(crate) const fn new(http: &'a Client, channel_id: ChannelId) -> Self {
        Self { channel_id, http }
    }

    /// Execute the request, returning a future resolving to a [`Response`].
    ///
    /// [`Response`]: crate::response::Response
    pub fn exec(self) -> ResponseFuture<StageInstance> {
        let request = Request::from_route(Route::GetStageInstance {
            channel_id: self.channel_id.0,
        });

        self.http.request(request)
    }
}
