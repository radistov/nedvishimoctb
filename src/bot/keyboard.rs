// src/bot/keyboards.rs

use teloxide::types::{
    InlineKeyboardButton,
    InlineKeyboardMarkup,
    KeyboardButton,
    KeyboardMarkup,
};

/// Главное меню пользователя.
pub fn main_menu() -> KeyboardMarkup {
    KeyboardMarkup::new(vec![
        vec![KeyboardButton::new("🏠 Подобрать недвижимость")],
        vec![
            KeyboardButton::new("⭐ Избранное"),
            KeyboardButton::new("👤 Профиль"),
        ],
        vec![KeyboardButton::new("ℹ Помощь")],
    ])
    .resize_keyboard()
}

/// Кнопка запроса номера телефона.
pub fn request_phone() -> KeyboardMarkup {
    KeyboardMarkup::new(vec![vec![
        KeyboardButton::new("📱 Отправить номер телефона")
            .request_contact(true),
    ]])
    .resize_keyboard()
    .one_time_keyboard()
}

/// Выбор города.
///
/// Позже список будет загружаться из базы данных.
pub fn city_keyboard() -> InlineKeyboardMarkup {
    InlineKeyboardMarkup::new(vec![
        vec![
            InlineKeyboardButton::callback(
                "Москва",
                "city:Москва",
            ),
            InlineKeyboardButton::callback(
                "Санкт-Петербург",
                "city:Санкт-Петербург",
            ),
        ],
        vec![
            InlineKeyboardButton::callback(
                "Казань",
                "city:Казань",
            ),
            InlineKeyboardButton::callback(
                "Екатеринбург",
                "city:Екатеринбург",
            ),
        ],
        vec![
            InlineKeyboardButton::callback(
                "Новосибирск",
                "city:Новосибирск",
            ),
        ],
    ])
}

/// Районы.
///
/// Пока статический список.
/// Позже будет зависеть от выбранного города.
pub fn district_keyboard() -> InlineKeyboardMarkup {
    InlineKeyboardMarkup::new(vec![
        vec![
            InlineKeyboardButton::callback(
                "Центральный",
                "district:Центральный",
            ),
        ],
        vec![
            InlineKeyboardButton::callback(
                "Северный",
                "district:Северный",
            ),
            InlineKeyboardButton::callback(
                "Южный",
                "district:Южный",
            ),
        ],
        vec![
            InlineKeyboardButton::callback(
                "Восточный",
                "district:Восточный",
            ),
            InlineKeyboardButton::callback(
                "Западный",
                "district:Западный",
            ),
        ],
    ])
}

/// Бюджет.
pub fn budget_keyboard() -> InlineKeyboardMarkup {
    InlineKeyboardMarkup::new(vec![
        vec![
            InlineKeyboardButton::callback(
                "До 3 млн",
                "budget:3000000",
            ),
        ],
        vec![
            InlineKeyboardButton::callback(
                "До 5 млн",
                "budget:5000000",
            ),
        ],
        vec![
            InlineKeyboardButton::callback(
                "До 8 млн",
                "budget:8000000",
            ),
        ],
        vec![
            InlineKeyboardButton::callback(
                "До 10 млн",
                "budget:10000000",
            ),
        ],
        vec![
            InlineKeyboardButton::callback(
                "Без ограничений",
                "budget:999999999",
            ),
        ],
    ])
}

/// Количество комнат.
pub fn rooms_keyboard() -> InlineKeyboardMarkup {
    InlineKeyboardMarkup::new(vec![
        vec![
            InlineKeyboardButton::callback("1", "rooms:1"),
            InlineKeyboardButton::callback("2", "rooms:2"),
        ],
        vec![
            InlineKeyboardButton::callback("3", "rooms:3"),
            InlineKeyboardButton::callback("4", "rooms:4"),
        ],
        vec![
            InlineKeyboardButton::callback("5+", "rooms:5"),
        ],
    ])
}

/// Кнопки карточки объекта.
pub fn property_keyboard(property_id: &str) -> InlineKeyboardMarkup {
    InlineKeyboardMarkup::new(vec![
        vec![
            InlineKeyboardButton::callback(
                "⭐ В избранное",
                format!("favorite:{property_id}"),
            ),
        ],
        vec![
            InlineKeyboardButton::callback(
                "📅 Записаться на просмотр",
                format!("view:{property_id}"),
            ),
        ],
        vec![
            InlineKeyboardButton::callback(
                "➡ Следующий",
                format!("next:{property_id}"),
            ),
        ],
    ])
}

/// Меню избранного.
pub fn favorites_keyboard(property_id: &str) -> InlineKeyboardMarkup {
    InlineKeyboardMarkup::new(vec![
        vec![
            InlineKeyboardButton::callback(
                "🗑 Удалить",
                format!("favorite_remove:{property_id}"),
            ),
        ],
        vec![
            InlineKeyboardButton::callback(
                "📅 Просмотр",
                format!("view:{property_id}"),
            ),
        ],
    ])
}

/// Административное меню.
pub fn admin_menu() -> KeyboardMarkup {
    KeyboardMarkup::new(vec![
        vec![
            KeyboardButton::new("➕ Добавить объект"),
            KeyboardButton::new("📋 Объекты"),
        ],
        vec![
            KeyboardButton::new("📨 Заявки"),
            KeyboardButton::new("📊 Статистика"),
        ],
    ])
    .resize_keyboard()
}