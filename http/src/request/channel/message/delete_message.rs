use crate::request::prelude::*;
use dawn_model::id::{ChannelId, MessageId};

pub struct DeleteMessage<'a> {
    channel_id: ChannelId,
    fut: Option<Pending<'a, ()>>,
    http: &'a Client,
    message_id: MessageId,
}

impl<'a> DeleteMessage<'a> {
    pub(crate) fn new(http: &'a Client, channel_id: ChannelId, message_id: MessageId) -> Self {
        Self {
            channel_id,
            fut: None,
            http,
            message_id,
        }
    }

    fn start(&mut self) -> Result<()> {
        self.fut.replace(Box::pin(self.http.request(Request::from(
            Route::DeleteMessage {
                channel_id: self.channel_id.0,
                message_id: self.message_id.0,
            },
        ))));

        Ok(())
    }
}

poll_req!(DeleteMessage<'_>, ());
