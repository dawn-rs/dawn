use crate::{
    client::Client,
    request::Request,
    response::{marker::ListBody, ResponseFuture},
    routing::Route,
};
use twilight_model::{
    guild::Role,
    id::{GuildId, RoleId},
};

/// Modify the position of the roles.
///
/// The minimum amount of roles to modify, is a swap between two roles.
pub struct UpdateRolePositions<'a> {
    guild_id: GuildId,
    http: &'a Client,
    roles: Vec<(RoleId, u64)>,
}

impl<'a> UpdateRolePositions<'a> {
    pub(crate) fn new(
        http: &'a Client,
        guild_id: GuildId,
        roles: impl Iterator<Item = (RoleId, u64)>,
    ) -> Self {
        Self {
            guild_id,
            http,
            roles: roles.collect(),
        }
    }

    /// Execute the request, returning a future resolving to a [`Response`].
    ///
    /// [`Response`]: crate::response::Response
    pub fn exec(self) -> ResponseFuture<ListBody<Role>> {
        let mut request = Request::builder(Route::UpdateRolePositions {
            guild_id: self.guild_id.0,
        });

        request = match request.json(&self.roles) {
            Ok(request) => request,
            Err(source) => return ResponseFuture::error(source),
        };

        self.http.request(request.build())
    }
}
