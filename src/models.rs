// src/models.rs

/*!
    Модели предметной области.

    Данный модуль содержит все основные структуры проекта.

    Здесь описываются:

    • Пользователи
    • Профили
    • Объекты недвижимости
    • Избранное
    • Заявки на просмотр
    • Статусы заявок
    • Состояния регистрации (FSM)

    Все структуры совместимы с SQLx и Serde.
*/

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use teloxide::macros::BotCommands;
use uuid::Uuid;

////////////////////////////////////////////////////////////////////////////////
// Telegram-команды
////////////////////////////////////////////////////////////////////////////////

/// Основные команды Telegram-бота.
#[derive(BotCommands, Clone)]
#[command(rename_rule = "lowercase")]
pub enum Command {
    #[command(description = "запустить бота")]
    Start,

    #[command(description = "мой профиль")]
    Profile,

    #[command(description = "начать подбор недвижимости")]
    Search,

    #[command(description = "избранное")]
    Favorites,

    #[command(description = "помощь")]
    Help,
}

////////////////////////////////////////////////////////////////////////////////
// Пользователь
////////////////////////////////////////////////////////////////////////////////

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct User {
    pub id: Uuid,

    /// Telegram ID пользователя.
    pub telegram_id: i64,

    pub username: Option<String>,

    pub first_name: String,

    pub last_name: Option<String>,

    pub phone: Option<String>,

    pub created_at: DateTime<Utc>,
}

////////////////////////////////////////////////////////////////////////////////
// Профиль пользователя
////////////////////////////////////////////////////////////////////////////////

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

////////////////////////////////////////////////////////////////////////////////
// Объект недвижимости
////////////////////////////////////////////////////////////////////////////////

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Property {
    pub id: Uuid,

    /// Заголовок объявления.
    pub title: String,

    /// Полное описание.
    pub description: String,

    pub city: String,

    pub district: String,

    pub price: i64,

    pub rooms: i32,

    pub area: f32,

    /// Ссылка на фотографию.
    ///
    /// В дальнейшем можно заменить на Telegram file_id.
    pub photo_url: Option<String>,

    pub is_active: bool,

    pub created_at: DateTime<Utc>,
}

////////////////////////////////////////////////////////////////////////////////
// Избранное
////////////////////////////////////////////////////////////////////////////////

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Favorite {
    pub id: Uuid,

    pub user_id: Uuid,

    pub property_id: Uuid,

    pub created_at: DateTime<Utc>,
}

////////////////////////////////////////////////////////////////////////////////
// Заявка на просмотр
////////////////////////////////////////////////////////////////////////////////

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct ViewingRequest {
    pub id: Uuid,

    pub user_id: Uuid,

    pub property_id: Uuid,

    pub status: RequestStatus,

    pub comment: Option<String>,

    pub created_at: DateTime<Utc>,
}

////////////////////////////////////////////////////////////////////////////////
// Статусы заявки
////////////////////////////////////////////////////////////////////////////////

#[derive(
    Debug,
    Clone,
    Copy,
    Serialize,
    Deserialize,
    sqlx::Type,
    PartialEq,
    Eq,
)]
#[sqlx(type_name = "TEXT")]
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

////////////////////////////////////////////////////////////////////////////////
// FSM регистрации
////////////////////////////////////////////////////////////////////////////////

/// Этапы заполнения анкеты пользователя.
///
/// Позже эти состояния будут использоваться
/// в teloxide::dispatching::dialogue.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub enum RegistrationState {
    /// Пользователь ещё не начал регистрацию.
    #[default]
    Start,

    /// Ввод телефона.
    WaitingPhone,

    /// Выбор города.
    WaitingCity,

    /// Выбор района.
    WaitingDistrict,

    /// Ввод бюджета.
    WaitingBudget,

    /// Выбор количества комнат.
    WaitingRooms,

    /// Дополнительные требования.
    WaitingAdditionalRequirements,

    /// Регистрация завершена.
    Completed,
}