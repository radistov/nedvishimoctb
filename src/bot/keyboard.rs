use teloxide::types::{
    InlineKeyboardButton, InlineKeyboardMarkup, KeyboardButton, KeyboardMarkup,
};

use uuid::Uuid;

pub fn main_menu() -> KeyboardMarkup {
    KeyboardMarkup::new(vec![
        vec![KeyboardButton::new("🏠 Подобрать недвижимость")],
        vec![
            KeyboardButton::new("⭐ Избранное"),
            KeyboardButton::new("👤 Профиль"),
        ],
        vec![KeyboardButton::new("ℹ️ Помощь")],
    ])
    .resize_keyboard()
}

pub fn request_phone() -> KeyboardMarkup {
    KeyboardMarkup::new(vec![vec![
        KeyboardButton::new("📱 Отправить номер телефона").request_contact(true),
    )])
    .resize_keyboard()
    .one_time_keyboard()
}

pub fn city_keyboard() -> InlineKeyboardMarkup {
    InlineKeyboardMarkup::new(vec![
        vec![
            InlineKeyboardButton::callback("Москва", "city:Москва"),
            InlineKeyboardButton::callback("Санкт-Петербург", "city:Санкт-Петербург"),
        ],
        vec![
            InlineKeyboardButton::callback("Казань", "city:Казань"),
            InlineKeyboardButton::callback("Екатеринбург", "city:Екатеринбург"),
        ],
        vec![
            InlineKeyboardButton::callback("Новосибирск", "city:Новосибирск"),
        ],
    ])
}

pub fn district_keyboard() -> InlineKeyboardMarkup {
    InlineKeyboardMarkup::new(vec![
        vec![
            InlineKeyboardButton::callback("Центральный", "district:Центральный"),
            InlineKeyboardButton::callback("Северный", "district:Северный"),
        ],
        vec![
            InlineKeyboardButton::callback("Южный", "district:Южный"),
            InlineKeyboardButton::callback("Западный", "district:Западный"),
        ],
        vec![
            InlineKeyboardButton::callback("Восточный", "district:Восточный"),
        ],
    ])
}

pub fn budget_keyboard() -> InlineKeyboardMarkup {
    InlineKeyboardMarkup::new(vec![
        vec![
            InlineKeyboardButton::callback("До 5 млн ₽", "budget:5000000"),
            InlineKeyboardButton::callback("До 8 млн ₽", "budget:8000000"),
        ],
        vec![
            InlineKeyboardButton::callback("До 10 млн ₽", "budget:10000000"),
            InlineKeyboardButton::callback("До 15 млн ₽", "budget:15000000"),
        ],
        vec![
            InlineKeyboardButton::callback("До 20 млн ₽", "budget:20000000"),
            InlineKeyboardButton::callback("До 30 млн ₽", "budget:30000000"),
        ],
    ])
}

pub fn rooms_keyboard() -> InlineKeyboardMarkup {
    InlineKeyboardMarkup::new(vec![
        vec![
            InlineKeyboardButton::callback("1 комната", "rooms:1"),
            InlineKeyboardButton::callback("2 комнаты", "rooms:2"),
        ],
        vec![
            InlineKeyboardButton::callback("3 комнаты", "rooms:3"),
            InlineKeyboardButton::callback("4 комнаты", "rooms:4"),
        ],
        vec![InlineKeyboardButton::callback("5+ комнат", "rooms:5")],
    ])
}

pub fn property_keyboard(property_id: Uuid) -> InlineKeyboardMarkup {
    let id = property_id.to_string();

    InlineKeyboardMarkup::new(vec![
        vec![
            InlineKeyboardButton::callback(
                "⭐ В избранное",
                format!("favorite:{id}"),
            ),
            InlineKeyboardButton::callback(
                "📅 Записаться",
                format!("view:{id}"),
            ),
        ],
        vec![InlineKeyboardButton::callback(
            "➡️ Следующий вариант",
            format!("next:{id}"),
        )],
    ])
}

pub fn favorites_keyboard(property_id: Uuid) -> InlineKeyboardMarkup {
    let id = property_id.to_string();

    InlineKeyboardMarkup::new(vec![
        vec![
            InlineKeyboardButton::callback(
                "❌ Удалить",
                format!("favorite_remove:{id}"),
            ),
            InlineKeyboardButton::callback(
                "📅 Записаться",
                format!("view:{id}"),
            ),
        ],
    ])
}

pub fn admin_menu() -> KeyboardMarkup {
    KeyboardMarkup::new(vec![
        vec![KeyboardButton::new("➕ Добавить объект")],
        vec![
            KeyboardButton::new("🏠 Список объектов"),
            KeyboardButton::new("📋 Заявки"),
        ],
        vec![KeyboardButton::new("📊 Статистика")],
    ])
    .resize_keyboard()
}