use serde::{Deserialize, Deserializer, Serialize, Serializer};
use sqlx::{Decode, Encode, Postgres, Type};
use sqlx::database::{HasArguments, HasValueRef};
use sqlx::encode::IsNull;
use sqlx::error::BoxDynError;

#[derive(Debug, Clone, Hash)]
pub enum UserRoles {
    Admin,
    User
}

impl Serialize for UserRoles {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error> where S: Serializer {
        match self {
            UserRoles::User => serializer.serialize_str("user"),
            UserRoles::Admin => serializer.serialize_str("admin"),
        }
    }
}

impl<'q> Encode<'q, Postgres> for UserRoles {
    fn encode_by_ref(&self, buf: &mut <Postgres as HasArguments<'q>>::ArgumentBuffer) -> IsNull {

        let text: String = match self {
            UserRoles::Admin => "admin".to_string(),
            UserRoles::User => "user".to_string()
        };

        buf.extend(text.as_bytes().iter());

        IsNull::No
    }
}

impl<'r> Decode<'r, Postgres> for UserRoles {
    fn decode(value: <Postgres as HasValueRef<'r>>::ValueRef) -> Result<Self, BoxDynError> {
        let role = value.as_str().unwrap();

        match role {
            "admin" => Ok(UserRoles::Admin),
            "user" => Ok(UserRoles::User),
            _ => Err(BoxDynError::from(format!("Invalid UserRoles decoded: {}", role)))
        }
    }
}

impl Type<Postgres> for UserRoles {
    fn type_info() -> <Postgres as sqlx::Database>::TypeInfo {
        <Postgres as sqlx::Database>::TypeInfo::with_name("varchar")
    }
}

impl<'d> Deserialize<'d> for UserRoles {
    fn deserialize<D>(deserializer: D) -> Result<UserRoles, D::Error> where D: Deserializer<'d> {
        match Deserialize::deserialize(deserializer) {
            Ok(value) => match value {
                "admin" => Ok(UserRoles::Admin),
                "user" => Ok(UserRoles::User),
                _ => Err(serde::de::Error::custom("Error"))
            },
            _ => Err(serde::de::Error::missing_field("role"))
        }
    }
}