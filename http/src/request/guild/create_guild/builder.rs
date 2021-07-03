use super::{CategoryFields, GuildChannelFields, RoleFields, TextFields, VoiceFields};
use std::{
    error::Error,
    fmt::{Display, Formatter, Result as FmtResult},
};
use twilight_model::{
    channel::{permission_overwrite::PermissionOverwrite, ChannelType},
    guild::Permissions,
    id::{ChannelId, RoleId},
};

/// Error building role fields.
#[derive(Debug)]
pub struct RoleFieldsError {
    kind: RoleFieldsErrorType,
}

impl RoleFieldsError {
    /// Immutable reference to the type of error that occurred.
    #[must_use = "retrieving the type has no effect if left unused"]
    pub const fn kind(&self) -> &RoleFieldsErrorType {
        &self.kind
    }

    /// Consume the error, returning the source error if there is any.
    #[allow(clippy::unused_self)]
    #[must_use = "consuming the error and retrieving the source has no effect if left unused"]
    pub fn into_source(self) -> Option<Box<dyn Error + Send + Sync>> {
        None
    }

    /// Consume the error, returning the owned error type and the source error.
    #[must_use = "consuming the error into its parts has no effect if left unused"]
    pub fn into_parts(self) -> (RoleFieldsErrorType, Option<Box<dyn Error + Send + Sync>>) {
        (self.kind, None)
    }
}

impl Display for RoleFieldsError {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        match &self.kind {
            RoleFieldsErrorType::ColorNotRgb { color } => {
                f.write_str("the color ")?;
                Display::fmt(color, f)?;

                f.write_str(" is invalid")
            }
            RoleFieldsErrorType::IdInvalid => {
                f.write_str("the given id value is 1, which is not acceptable")
            }
        }
    }
}

impl Error for RoleFieldsError {}

/// Type of [`RoleFieldsError`] that occurred.
#[derive(Debug)]
#[non_exhaustive]
pub enum RoleFieldsErrorType {
    /// Color was larger than a valid RGB hexadecimal value.
    ColorNotRgb {
        /// Provided color hex value.
        color: u32,
    },
    /// Invalid id for builders.
    IdInvalid,
}

/// A builder for role fields.
#[derive(Clone, Debug, Eq, PartialEq)]
#[must_use = "must be built into a role"]
pub struct RoleFieldsBuilder(RoleFields);

impl RoleFieldsBuilder {
    /// The maximumn accepted color value.
    ///
    /// This is used by [`color`].
    ///
    /// [`color`]: Self::color
    pub const COLOR_MAXIMUM: u32 = 0xff_ff_ff;

    const ROLE_ID: RoleId = RoleId(1);

    /// Create a new default role field builder.
    pub const fn new(name: String) -> Self {
        Self(RoleFields {
            color: None,
            hoist: None,
            id: Self::ROLE_ID,
            mentionable: None,
            name,
            permissions: None,
            position: None,
        })
    }

    /// Build the role fields.
    #[allow(clippy::missing_const_for_fn)]
    pub fn build(self) -> RoleFields {
        self.0
    }

    /// Set the role color.
    ///
    /// This must be a valid hexadecimal RGB value.
    ///
    /// # Errors
    ///
    /// Returns a [`RoleFieldsErrorType::ColorNotRgb`] error type if the color
    /// is not valid RGB.
    pub fn color(mut self, color: u32) -> Result<Self, RoleFieldsError> {
        if color > Self::COLOR_MAXIMUM {
            return Err(RoleFieldsError {
                kind: RoleFieldsErrorType::ColorNotRgb { color },
            });
        }

        self.0.color.replace(color);

        Ok(self)
    }

    /// Show the role above other roles in the user list.
    pub const fn hoist(mut self) -> Self {
        self.0.hoist = Some(true);

        self
    }

    /// Set the id of the role.
    ///
    /// # Errors
    ///
    /// Returns a [`RoleFieldsErrorType::IdInvalid`] error type if the ID is set
    /// to 1.
    pub fn id(mut self, id: RoleId) -> Result<Self, RoleFieldsError> {
        if id == Self::ROLE_ID {
            return Err(RoleFieldsError {
                kind: RoleFieldsErrorType::IdInvalid,
            });
        }

        self.0.id = id;

        Ok(self)
    }

    /// Allow the role to be @mentioned.
    pub const fn mentionable(mut self) -> Self {
        self.0.mentionable = Some(true);

        self
    }

    /// Set the permissions of the role.
    pub const fn permissions(mut self, permissions: Permissions) -> Self {
        self.0.permissions = Some(permissions);

        self
    }

    /// Set the position of the role.
    pub const fn position(mut self, position: i64) -> Self {
        self.0.position = Some(position);

        self
    }
}

/// Error building text fields.
#[derive(Debug)]
pub struct TextFieldsError {
    kind: TextFieldsErrorType,
}

impl TextFieldsError {
    /// Immutable reference to the type of error that occurred.
    #[must_use = "retrieving the type has no effect if left unused"]
    pub const fn kind(&self) -> &TextFieldsErrorType {
        &self.kind
    }

    /// Consume the error, returning the source error if there is any.
    #[allow(clippy::unused_self)]
    #[must_use = "consuming the error and retrieving the source has no effect if left unused"]
    pub fn into_source(self) -> Option<Box<dyn Error + Send + Sync>> {
        None
    }

    /// Consume the error, returning the owned error type and the source error.
    #[must_use = "consuming the error into its parts has no effect if left unused"]
    pub fn into_parts(self) -> (TextFieldsErrorType, Option<Box<dyn Error + Send + Sync>>) {
        (self.kind, None)
    }
}

impl Display for TextFieldsError {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        match &self.kind {
            TextFieldsErrorType::NameTooShort { name } => {
                f.write_str("the name is too short: ")?;

                Display::fmt(&name.len(), f)
            }
            TextFieldsErrorType::NameTooLong { name } => {
                f.write_str("the name is too long: ")?;

                Display::fmt(&name.len(), f)
            }
            TextFieldsErrorType::RateLimitInvalid { limit } => {
                f.write_str("the rate limit ")?;
                Display::fmt(limit, f)?;

                f.write_str(" is invalid")
            }
            TextFieldsErrorType::TopicTooLong { topic } => {
                f.write_str("the topic is too long: ")?;

                Display::fmt(&topic.len(), f)
            }
        }
    }
}

impl Error for TextFieldsError {}

/// Type of [`TextFieldsError`] that occurred.
#[derive(Debug)]
#[non_exhaustive]
pub enum TextFieldsErrorType {
    /// The name is too short.
    NameTooShort {
        /// The invalid name.
        name: String,
    },
    /// The name is too long.
    NameTooLong {
        /// The invalid name.
        name: String,
    },
    /// The rate limit is invalid.
    RateLimitInvalid {
        /// The incorrect rate limit.
        limit: u64,
    },
    /// The topic is too long.
    TopicTooLong {
        /// The incorrect topic.
        topic: String,
    },
}

/// A builder for text fields.
#[derive(Clone, Debug, Eq, PartialEq)]
#[must_use = "must be built into a text channel"]
pub struct TextFieldsBuilder(TextFields);

impl TextFieldsBuilder {
    /// The minimum number of UTF-16 code points that can be in a channel name.
    ///
    /// This is used by [`new`].
    ///
    /// [`new`]: Self::new
    pub const MIN_NAME_LENGTH: usize = 2;

    /// The maximum number of UTF-16 code points that can be in a channel name.
    ///
    /// This is used by [`new`].
    ///
    /// [`new`]: Self::new
    pub const MAX_NAME_LENGTH: usize = 100;

    /// The maximum length of a rate limit.
    ///
    /// This is used by [`rate_limit_per_user`].
    ///
    /// [`rate_limit_per_user`]: Self::rate_limit_per_user
    pub const MAX_RATE_LIMIT: u64 = 21600;

    /// The maximum number of UTF-16 code points that can be in a channel topic.
    ///
    /// This is used by [`topic`].
    ///
    /// [`topic`]: Self::topic
    pub const MAX_TOPIC_LENGTH: usize = 1024;

    /// Create a new text fields builder.
    ///
    /// # Errors
    ///
    /// Returns a [`TextFieldsErrorType::NameTooShort`] error type if the name
    /// is too short.
    ///
    /// Returns a [`TextFieldsErrorType::NameTooLong`] error type if the name is
    /// too long.
    pub fn new(name: String) -> Result<Self, TextFieldsError> {
        if name.len() < Self::MIN_NAME_LENGTH {
            return Err(TextFieldsError {
                kind: TextFieldsErrorType::NameTooShort { name },
            });
        }

        if name.len() > Self::MAX_NAME_LENGTH {
            return Err(TextFieldsError {
                kind: TextFieldsErrorType::NameTooLong { name },
            });
        }

        Ok(Self(TextFields {
            id: ChannelId(1),
            kind: ChannelType::GuildText,
            name,
            nsfw: None,
            permission_overwrites: None,
            parent_id: None,
            rate_limit_per_user: None,
            topic: None,
        }))
    }

    /// Build the text fields.
    #[allow(clippy::missing_const_for_fn)]
    pub fn build(self) -> TextFields {
        self.0
    }

    /// Make the channel NSFW.
    pub const fn nsfw(mut self) -> Self {
        self.0.nsfw = Some(true);

        self
    }

    /// Set the channel's permission overwrites.
    #[allow(clippy::missing_const_for_fn)]
    pub fn permission_overwrites(mut self, overwrites: Vec<PermissionOverwrite>) -> Self {
        self.0.permission_overwrites = Some(overwrites);

        self
    }

    /// Set the channel's rate limit per user.
    ///
    /// # Errors
    ///
    /// Returns a [`TextFieldsErrorType::RateLimitInvalid`] error type if the
    /// rate limit is invalid.
    #[allow(clippy::missing_const_for_fn)]
    pub fn rate_limit_per_user(mut self, limit: u64) -> Result<Self, TextFieldsError> {
        if limit > Self::MAX_RATE_LIMIT {
            return Err(TextFieldsError {
                kind: TextFieldsErrorType::RateLimitInvalid { limit },
            });
        }

        self.0.rate_limit_per_user = Some(limit);

        Ok(self)
    }

    /// Set the channel's topic.
    ///
    /// # Errors
    ///
    /// Returns a [`TextFieldsErrorType::TopicTooLong`] error type if the topic
    /// is too long.
    pub fn topic(mut self, topic: String) -> Result<Self, TextFieldsError> {
        if topic.len() > Self::MAX_TOPIC_LENGTH {
            return Err(TextFieldsError {
                kind: TextFieldsErrorType::TopicTooLong { topic },
            });
        }

        self.0.topic.replace(topic);

        Ok(self)
    }
}

/// Error building voice fields.
#[derive(Debug)]
pub struct VoiceFieldsError {
    kind: VoiceFieldsErrorType,
}

impl VoiceFieldsError {
    /// Immutable reference to the type of error that occurred.
    #[must_use = "retrieving the type has no effect if left unused"]
    pub const fn kind(&self) -> &VoiceFieldsErrorType {
        &self.kind
    }

    /// Consume the error, returning the source error if there is any.
    #[allow(clippy::unused_self)]
    #[must_use = "consuming the error and retrieving the source has no effect if left unused"]
    pub fn into_source(self) -> Option<Box<dyn Error + Send + Sync>> {
        None
    }

    /// Consume the error, returning the owned error type and the source error.
    #[must_use = "consuming the error into its parts has no effect if left unused"]
    pub fn into_parts(self) -> (VoiceFieldsErrorType, Option<Box<dyn Error + Send + Sync>>) {
        (self.kind, None)
    }
}

impl Display for VoiceFieldsError {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        match &self.kind {
            VoiceFieldsErrorType::NameTooShort { name } => {
                f.write_str("the name is too short: ")?;

                Display::fmt(&name.len(), f)
            }
            VoiceFieldsErrorType::NameTooLong { name } => {
                f.write_str("the name is too long: ")?;

                Display::fmt(&name.len(), f)
            }
        }
    }
}

impl Error for VoiceFieldsError {}

/// Type of [`VoiceFieldsError`] that occurred.
#[derive(Debug)]
#[non_exhaustive]
pub enum VoiceFieldsErrorType {
    /// The name is too short.
    NameTooShort {
        /// The invalid name.
        name: String,
    },
    /// THe name is too long.
    NameTooLong {
        /// The invalid name.
        name: String,
    },
}

/// A builder for voice fields.
#[derive(Clone, Debug, Eq, PartialEq)]
#[must_use = "must be built into a voice channel"]
pub struct VoiceFieldsBuilder(VoiceFields);

impl VoiceFieldsBuilder {
    /// The minimum number of UTF-16 code points that can be in a channel name.
    ///
    /// This is used by [`new`].
    ///
    /// [`new`]: Self::new
    pub const MIN_NAME_LENGTH: usize = 2;

    /// The maximum number of UTF-16 code points that can be in a channel name.
    ///
    /// This is used by [`new`].
    ///
    /// [`new`]: Self::new
    pub const MAX_NAME_LENGTH: usize = 100;

    /// Create a new voice fields builder.
    ///
    /// # Errors
    ///
    /// Returns a [`VoiceFieldsErrorType::NameTooShort`] error type if the name
    /// is too short.
    ///
    /// Returns a [`VoiceFieldsErrorType::NameTooLong`] error type if the name
    /// is too long.
    pub fn new(name: String) -> Result<Self, VoiceFieldsError> {
        if name.len() < Self::MIN_NAME_LENGTH {
            return Err(VoiceFieldsError {
                kind: VoiceFieldsErrorType::NameTooShort { name },
            });
        }

        if name.len() > Self::MAX_NAME_LENGTH {
            return Err(VoiceFieldsError {
                kind: VoiceFieldsErrorType::NameTooLong { name },
            });
        }

        Ok(Self(VoiceFields {
            bitrate: None,
            id: ChannelId(1),
            kind: ChannelType::GuildVoice,
            name,
            permission_overwrites: None,
            parent_id: None,
            user_limit: None,
        }))
    }

    /// Build the voice fields.
    #[allow(clippy::missing_const_for_fn)]
    pub fn build(self) -> VoiceFields {
        self.0
    }

    /// Set the voice channel's bitrate.
    pub const fn bitrate(mut self, bitrate: u64) -> Self {
        self.0.bitrate = Some(bitrate);

        self
    }

    /// Set the channel's permission overwrites.
    pub fn permission_overwrites(mut self, overwrites: Vec<PermissionOverwrite>) -> Self {
        self.0.permission_overwrites.replace(overwrites);

        self
    }

    /// Set the voice channel's user limit.
    pub const fn user_limit(mut self, limit: u64) -> Self {
        self.0.user_limit = Some(limit);

        self
    }
}

/// Error creating category fields.
#[derive(Debug)]
pub struct CategoryFieldsError {
    kind: CategoryFieldsErrorType,
}

impl CategoryFieldsError {
    /// Immutable reference to the type of error that occurred.
    #[must_use = "retrieving the type has no effect if left unused"]
    pub const fn kind(&self) -> &CategoryFieldsErrorType {
        &self.kind
    }

    /// Consume the error, returning the source error if there is any.
    #[allow(clippy::unused_self)]
    #[must_use = "consuming the error and retrieving the source has no effect if left unused"]
    pub fn into_source(self) -> Option<Box<dyn Error + Send + Sync>> {
        None
    }

    /// Consume the error, returning the owned error type and the source error.
    #[must_use = "consuming the error into its parts has no effect if left unused"]
    pub fn into_parts(
        self,
    ) -> (
        CategoryFieldsErrorType,
        Option<Box<dyn Error + Send + Sync>>,
    ) {
        (self.kind, None)
    }
}

impl Display for CategoryFieldsError {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        match &self.kind {
            CategoryFieldsErrorType::NameTooShort { name } => {
                f.write_str("the name is too short: ")?;

                Display::fmt(&name.len(), f)
            }
            CategoryFieldsErrorType::NameTooLong { name } => {
                f.write_str("the name is too long: ")?;

                Display::fmt(&name.len(), f)
            }
        }
    }
}

impl Error for CategoryFieldsError {}

/// Type of [`CategoryFieldsError`] that occurred.
#[derive(Debug)]
#[non_exhaustive]
pub enum CategoryFieldsErrorType {
    /// The name is too short.
    NameTooShort {
        /// The invalid name.
        name: String,
    },
    /// THe name is too long.
    NameTooLong {
        /// The invalid name.
        name: String,
    },
}

/// A builder for a category channel, and its children.
#[derive(Clone, Debug, Eq, PartialEq)]
#[must_use = "must be built into a category channel"]
pub struct CategoryFieldsBuilder {
    fields: CategoryFields,
    channels: Vec<GuildChannelFields>,
}

impl CategoryFieldsBuilder {
    /// The minimum number of UTF-16 code points that can be in a channel name.
    ///
    /// This is used by [`new`].
    ///
    /// [`new`]: Self::new
    pub const MIN_NAME_LENGTH: usize = 2;

    /// The maximum number of UTF-16 code points that can be in a channel name.
    ///
    /// This is used by [`new`].
    ///
    /// [`new`]: Self::new
    pub const MAX_NAME_LENGTH: usize = 100;

    /// Create a new category fields builder.
    ///
    /// # Errors
    ///
    /// Returns a [`CategoryFieldsErrorType::NameTooShort`] error type if the
    /// name is too short.
    ///
    /// Returns a [`CategoryFieldsErrorType::NameTooLong`] error type if the
    /// name is too long.
    pub fn new(name: String) -> Result<Self, CategoryFieldsError> {
        if name.len() < Self::MIN_NAME_LENGTH {
            return Err(CategoryFieldsError {
                kind: CategoryFieldsErrorType::NameTooShort { name },
            });
        }

        if name.len() > Self::MAX_NAME_LENGTH {
            return Err(CategoryFieldsError {
                kind: CategoryFieldsErrorType::NameTooLong { name },
            });
        }

        Ok(Self {
            fields: CategoryFields {
                id: ChannelId(1),
                name,
                kind: ChannelType::GuildCategory,
                permission_overwrites: None,
            },
            channels: Vec::new(),
        })
    }

    pub(super) fn build(mut self, id: ChannelId) -> Vec<GuildChannelFields> {
        for channel in &mut self.channels {
            match channel {
                GuildChannelFields::Text(t) => t.parent_id.replace(id),
                GuildChannelFields::Voice(v) => v.parent_id.replace(id),
                GuildChannelFields::Category(_) => None,
            };
        }

        self.channels.insert(
            0,
            GuildChannelFields::Category(CategoryFields { id, ..self.fields }),
        );

        self.channels
    }

    /// Add a child text channel.
    pub fn add_text(mut self, channel: TextFields) -> Self {
        self.channels.push(GuildChannelFields::Text(channel));

        self
    }

    /// add a child voice channel.
    pub fn add_voice(mut self, channel: VoiceFields) -> Self {
        self.channels.push(GuildChannelFields::Voice(channel));

        self
    }
}

/// A builder for a list of channels.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
#[must_use = "must be built into a list of channnels"]
pub struct GuildChannelFieldsBuilder(Vec<GuildChannelFields>);

impl GuildChannelFieldsBuilder {
    /// Create a new channels builder.
    pub const fn new() -> Self {
        Self(Vec::new())
    }

    /// Build the list of channels.
    #[allow(clippy::missing_const_for_fn)]
    pub fn build(self) -> Vec<GuildChannelFields> {
        self.0
    }

    /// Add a text channel to the builder.
    pub fn add_text(mut self, channel: TextFields) -> Self {
        self.0.push(GuildChannelFields::Text(channel));

        self
    }

    /// Add a voice channel to the builder.
    pub fn add_voice(mut self, channel: VoiceFields) -> Self {
        self.0.push(GuildChannelFields::Voice(channel));

        self
    }

    /// Add a category channel builder, and all its children to the builder.
    pub fn add_category_builder(mut self, channel: CategoryFieldsBuilder) -> Self {
        let last_id = self
            .0
            .iter()
            .rev()
            .find(|c| matches!(c, GuildChannelFields::Category(_)))
            .map_or(ChannelId(1), GuildChannelFields::id);

        let mut channels = channel.build(ChannelId(last_id.0 + 1));

        self.0.append(&mut channels);

        self
    }
}

#[cfg(test)]
mod tests {
    use super::{
        super::{CategoryFields, GuildChannelFields, RoleFields, TextFields, VoiceFields},
        CategoryFieldsBuilder, CategoryFieldsErrorType, GuildChannelFieldsBuilder,
        RoleFieldsBuilder, RoleFieldsErrorType, TextFieldsBuilder, TextFieldsErrorType,
        VoiceFieldsBuilder, VoiceFieldsErrorType,
    };
    use twilight_model::{
        channel::{
            permission_overwrite::{PermissionOverwrite, PermissionOverwriteType},
            ChannelType,
        },
        guild::Permissions,
        id::{ChannelId, RoleId},
    };

    fn perms() -> Permissions {
        Permissions::CONNECT | Permissions::SPEAK | Permissions::SEND_TTS_MESSAGES
    }

    fn overwrite() -> PermissionOverwrite {
        PermissionOverwrite {
            allow: perms(),
            deny: Permissions::empty(),
            kind: PermissionOverwriteType::Role(RoleId(2)),
        }
    }

    fn voice() -> VoiceFields {
        VoiceFieldsBuilder::new("voicename".to_owned())
            .unwrap()
            .bitrate(96_000)
            .permission_overwrites(vec![overwrite()])
            .user_limit(40)
            .build()
    }

    #[test]
    fn test_role_fields() {
        assert!(matches!(
            RoleFieldsBuilder::new("role".to_owned())
                .color(123_123_123)
                .unwrap_err()
                .kind(),
            RoleFieldsErrorType::ColorNotRgb { color: 123_123_123 },
        ));

        let fields = RoleFieldsBuilder::new("rolename".to_owned())
            .color(0x12_34_56)
            .unwrap()
            .hoist()
            .id(RoleId(2))
            .unwrap()
            .mentionable()
            .permissions(Permissions::empty())
            .position(1);

        assert_eq!(
            fields.build(),
            RoleFields {
                color: Some(0x12_34_56),
                hoist: Some(true),
                id: RoleId(2),
                mentionable: Some(true),
                name: String::from("rolename"),
                permissions: Some(Permissions::empty()),
                position: Some(1),
            }
        );
    }

    #[test]
    fn test_voice_fields() {
        assert!(matches!(
            VoiceFieldsBuilder::new("c".to_owned()).unwrap_err().kind(),
            VoiceFieldsErrorType::NameTooShort { name }
            if name == "c"
        ));

        assert_eq!(
            voice(),
            VoiceFields {
                bitrate: Some(96_000),
                id: ChannelId(1),
                kind: ChannelType::GuildVoice,
                name: String::from("voicename"),
                permission_overwrites: Some(vec![PermissionOverwrite {
                    allow: perms(),
                    deny: Permissions::empty(),
                    kind: PermissionOverwriteType::Role(RoleId(2)),
                }]),
                parent_id: None,
                user_limit: Some(40),
            }
        );
    }

    fn text() -> TextFields {
        TextFieldsBuilder::new("textname".to_owned())
            .unwrap()
            .nsfw()
            .permission_overwrites(vec![overwrite()])
            .rate_limit_per_user(4_000)
            .unwrap()
            .topic("a topic".to_owned())
            .unwrap()
            .build()
    }

    #[test]
    fn test_text_fields() {
        assert!(matches!(
            TextFieldsBuilder::new("b".to_owned()).unwrap_err().kind(),
            TextFieldsErrorType::NameTooShort { name }
            if name == "b"
        ));

        assert_eq!(
            text(),
            TextFields {
                id: ChannelId(1),
                kind: ChannelType::GuildText,
                name: String::from("textname"),
                nsfw: Some(true),
                permission_overwrites: Some(vec![PermissionOverwrite {
                    allow: perms(),
                    deny: Permissions::empty(),
                    kind: PermissionOverwriteType::Role(RoleId(2)),
                }]),
                parent_id: None,
                rate_limit_per_user: Some(4_000),
                topic: Some(String::from("a topic")),
            }
        );
    }

    fn category() -> CategoryFieldsBuilder {
        CategoryFieldsBuilder::new("category".to_owned())
            .unwrap()
            .add_text(text())
            .add_voice(voice())
    }

    #[test]
    fn test_category_fields() {
        assert!(matches!(
            CategoryFieldsBuilder::new("a".to_owned()).unwrap_err().kind(),
            CategoryFieldsErrorType::NameTooShort { name }
            if name == "a"
        ));

        let fields = category();
        let channels = GuildChannelFieldsBuilder::new().add_category_builder(fields);

        assert_eq!(
            channels.build(),
            vec![
                GuildChannelFields::Category(CategoryFields {
                    id: ChannelId(2),
                    kind: ChannelType::GuildCategory,
                    name: String::from("category"),
                    permission_overwrites: None,
                }),
                GuildChannelFields::Text(TextFields {
                    id: ChannelId(1),
                    kind: ChannelType::GuildText,
                    name: String::from("textname"),
                    nsfw: Some(true),
                    permission_overwrites: Some(vec![PermissionOverwrite {
                        allow: Permissions::CONNECT
                            | Permissions::SPEAK
                            | Permissions::SEND_TTS_MESSAGES,
                        deny: Permissions::empty(),
                        kind: PermissionOverwriteType::Role(RoleId(2)),
                    }]),
                    parent_id: Some(ChannelId(2)),
                    rate_limit_per_user: Some(4_000),
                    topic: Some(String::from("a topic")),
                }),
                GuildChannelFields::Voice(VoiceFields {
                    bitrate: Some(96_000),
                    id: ChannelId(1),
                    kind: ChannelType::GuildVoice,
                    name: String::from("voicename"),
                    permission_overwrites: Some(vec![PermissionOverwrite {
                        allow: Permissions::CONNECT
                            | Permissions::SPEAK
                            | Permissions::SEND_TTS_MESSAGES,
                        deny: Permissions::empty(),
                        kind: PermissionOverwriteType::Role(RoleId(2)),
                    }]),
                    parent_id: Some(ChannelId(2)),
                    user_limit: Some(40),
                }),
            ]
        );
    }

    #[test]
    fn test_channels() {
        let channels = GuildChannelFieldsBuilder::new()
            .add_text(text())
            .add_voice(voice());

        assert_eq!(
            channels.build(),
            vec![
                GuildChannelFields::Text(TextFields {
                    id: ChannelId(1),
                    kind: ChannelType::GuildText,
                    name: String::from("textname"),
                    nsfw: Some(true),
                    permission_overwrites: Some(vec![PermissionOverwrite {
                        allow: Permissions::CONNECT
                            | Permissions::SPEAK
                            | Permissions::SEND_TTS_MESSAGES,
                        deny: Permissions::empty(),
                        kind: PermissionOverwriteType::Role(RoleId(2)),
                    }]),
                    parent_id: None,
                    rate_limit_per_user: Some(4_000),
                    topic: Some(String::from("a topic")),
                }),
                GuildChannelFields::Voice(VoiceFields {
                    bitrate: Some(96_000),
                    id: ChannelId(1),
                    kind: ChannelType::GuildVoice,
                    name: String::from("voicename"),
                    permission_overwrites: Some(vec![PermissionOverwrite {
                        allow: Permissions::CONNECT
                            | Permissions::SPEAK
                            | Permissions::SEND_TTS_MESSAGES,
                        deny: Permissions::empty(),
                        kind: PermissionOverwriteType::Role(RoleId(2)),
                    }]),
                    parent_id: None,
                    user_limit: Some(40),
                }),
            ]
        );
    }
}
