use crate::{
    guild::Permissions,
    id::{RoleId, UserId},
};
use serde::{
    de::{Deserializer, Error as DeError},
    ser::SerializeStruct,
    Deserialize, Serialize, Serializer,
};

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct PermissionOverwrite {
    pub allow: Permissions,
    pub deny: Permissions,
    pub kind: PermissionOverwriteType,
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub enum PermissionOverwriteType {
    Member(UserId),
    Role(RoleId),
}

#[derive(Deserialize)]
struct PermissionOverwriteData {
    allow: Permissions,
    deny: Permissions,
    id: String,
    #[serde(rename = "type")]
    kind: PermissionOverwriteTypeName,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
enum PermissionOverwriteTypeName {
    Member,
    Role,
}

impl<'de> Deserialize<'de> for PermissionOverwrite {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let data = PermissionOverwriteData::deserialize(deserializer)?;

        let span = tracing::trace_span!("deserializing permission overwrite");
        let _span_enter = span.enter();

        let kind = match data.kind {
            PermissionOverwriteTypeName::Member => {
                let id = UserId(data.id.parse().map_err(DeError::custom)?);
                tracing::trace!(id = %id.0, kind = ?data.kind);

                PermissionOverwriteType::Member(id)
            }
            PermissionOverwriteTypeName::Role => {
                let id = RoleId(data.id.parse().map_err(DeError::custom)?);
                tracing::trace!(id = %id.0, kind = ?data.kind);

                PermissionOverwriteType::Role(id)
            }
        };

        Ok(Self {
            allow: data.allow,
            deny: data.deny,
            kind,
        })
    }
}

impl Serialize for PermissionOverwrite {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut state = serializer.serialize_struct("PermissionOverwrite", 4)?;

        state.serialize_field("allow", &self.allow)?;
        state.serialize_field("deny", &self.deny)?;

        match &self.kind {
            PermissionOverwriteType::Member(id) => {
                state.serialize_field("id", &id)?;
                state.serialize_field("type", "member")?;
            }
            PermissionOverwriteType::Role(id) => {
                state.serialize_field("id", &id)?;
                state.serialize_field("type", "role")?;
            }
        }

        state.end()
    }
}

#[cfg(test)]
mod tests {
    use super::{PermissionOverwrite, PermissionOverwriteType, Permissions};
    use crate::id::UserId;

    #[test]
    fn test_overwrite() {
        let overwrite = PermissionOverwrite {
            allow: Permissions::CREATE_INVITE,
            deny: Permissions::KICK_MEMBERS,
            kind: PermissionOverwriteType::Member(UserId(12_345_678)),
        };

        // We can't use serde_test because it doesn't support 128 bit integers.
        //
        // <https://github.com/serde-rs/serde/issues/1281>
        let input = r#"{
  "allow": "1",
  "deny": "2",
  "id": "12345678",
  "type": "member"
}"#;

        assert_eq!(
            serde_json::from_str::<PermissionOverwrite>(input).unwrap(),
            overwrite
        );
        assert_eq!(serde_json::to_string_pretty(&overwrite).unwrap(), input);
    }
}
