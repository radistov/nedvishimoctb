use std::{fmt, str::FromStr};

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use teloxide::macros::BotCommands;
use uuid::Uuid;

#[derive(BotCommands, Clone, Debug)]
#[command(rename_rule = "lowercase")]
pub enum Command {
    #[command(description = "Запустить бота")]
    Start,

    #[command(description = "Показать профиль")]
    Profile,

    #[command(description = "Подобрать недвижимость")]
    Search,

    #[command(description = "Показать избранное")]
    Favorites,

    #[command(description = "Помощь")]
    Help,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct User {
    pub id: Uuid,
    pub telegram_id: i64,
    pub username: Option<String>,
    pub first_name: String,
    pub last_name: Option<String>,
    pub phone: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct UserProfile {
    pub id: Uuid,
    pub user_id: Uuid,
    pub city: Option<String>,
    pub district: Option<String>,
    pub budget: Option<i64>,
    pub rooms: Option<i32>,
    pub additional_requirements: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Property {
    pub id: Uuid,
    pub title: String,
    pub description: String,
    pub city: String,
    pub district: String,
    pub price: i64,
    pub rooms: i32,
    pub area: f32,
    pub photo_url: Option<String>,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Favorite {
    pub id: Uuid,
    pub user_id: Uuid,
    pub property_id: Uuid,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct ViewingRequest {
    pub id: Uuid,
    pub user_id: Uuid,
    pub property_id: Uuid,
    pub status: RequestStatus,
    pub comment: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum RequestStatus {
    New,
    InProgress,
    Approved,
    Rejected,
    Completed,
}

impl Default for RequestStatus {
    fn default() -> Self {
        Self::New
    }
}

impl fmt::Display for RequestStatus {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let value = match self {
            Self::New => "New",
            Self::InProgress => "InProgress",
            Self::Approved => "Approved",
            Self::Rejected => "Rejected",
            Self::Completed => "Completed",
        };

        formatter.write_str(value)
    }
}

impl FromStr for RequestStatus {
    type Err = String;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "New" => Ok(Self::New),
            "InProgress" => Ok(Self::InProgress),
            "Approved" => Ok(Self::Approved),
            "Rejected" => Ok(Self::Rejected),
            "Completed" => Ok(Self::Completed),
            _ => Err(format!("Неизвестный статус заявки: {value}")),
        }
    }
}

impl<'r> sqlx::Decode<'r, sqlx::Postgres> for RequestStatus {
    fn decode(
        value: sqlx::postgres::PgValueRef<'r>,
    ) -> Result<Self, sqlx::error::BoxDynError> {
        let value = <String as sqlx::Decode<sqlx::Postgres>>::decode(value)?;

        value
            .parse()
            .map_err(|error: String| error.into())
    }
}

impl sqlx::Type<sqlx::Postgres> for RequestStatus {
    fn type_info() -> sqlx::postgres::PgTypeInfo {
        <String as sqlx::Type<sqlx::Postgres>>::type_info()
    }

    fn compatible(ty: &sqlx::postgres::PgTypeInfo) -> bool {
        <String as sqlx::Type<sqlx::Postgres>>::compatible(ty)
    }
}

impl<'q> sqlx::Encode<'q, sqlx::Postgres> for RequestStatus {
    fn encode_by_ref(
        &self,
        buf: &mut Vec<u8>,
    ) -> sqlx::encode::IsNull {
        <String as sqlx::Encode<sqlx::Postgres>>::encode_by_ref(
            &self.to_string(),
            buf,
        )
    }

    fn produces(&self) -> Option<sqlx::postgres::PgTypeInfo> {
        Some(<String as sqlx::Type<sqlx::Postgres>>::type_info())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub enum RegistrationState {
    #[default]
    Start,
    WaitingPhone,
    WaitingCity,
    WaitingDistrict,
    WaitingBudget,
    WaitingRooms,
    WaitingAdditionalRequirements,
    Completed,
}