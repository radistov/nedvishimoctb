// src/bot/handlers.rs

use std::sync::Arc;

use anyhow::{Context, Result};
use chrono::Utc;
use teloxide::{
    dispatching::dialogue::GetChatId,
    prelude::*,
    types::{
        CallbackQuery, ChatId, InputFile, KeyboardRemove, Message,
    },
};

use uuid::Uuid;

use crate::{
    bot::{
        keyboards::{
            budget_keyboard,
            city_keyboard,
            district_keyboard,
            favorites_keyboard,
            main_menu,
            property_keyboard,
            request_phone,
            rooms_keyboard,
        },
        HandlerResult,
        MyDialogue,
    },
    db::operations,
    models::{
        Command,
        Favorite,
        RegistrationState,
        RequestStatus,
        User,
        UserProfile,
        ViewingRequest,
    },
    AppState,
};

/// Обрабатывает команды Telegram.
pub async fn command_handler(
    bot: Bot,
    dialogue: MyDialogue,
    message: Message,
    command: Command,
    state: Arc<AppState>,
) -> HandlerResult {
    let user = message
        .from
        .as_ref()
        .context("У сообщения отсутствует пользователь")?;

    match command {
        Command::Start => {
            start(
                bot,
                dialogue,
                message,
                user.id.0 as i64,
                state,
            )
            .await?;
        }

        Command::Profile => {
            show_profile(
                bot,
                message,
                user.id.0 as i64,
                state,
            )
            .await?;
        }

        Command::Search => {
            start_search(
                bot,
                dialogue,
                message,
                user.id.0 as i64,
                state,
            )
            .await?;
        }

        Command::Favorites => {
            show_favorites(
                bot,
                message,
                user.id.0 as i64,
                state,
            )
            .await?;
        }

        Command::Help => {
            show_help(bot, message).await?;
        }
    }

    Ok(())
}

/// Обрабатывает обычные текстовые сообщения.
pub async fn message_handler(
    bot: Bot,
    dialogue: MyDialogue,
    message: Message,
    state: Arc<AppState>,
) -> HandlerResult {
    let text = match message.text() {
        Some(text) => text.trim(),
        None => return Ok(()),
    };

    let telegram_id = message
        .from
        .as_ref()
        .map(|user| user.id.0 as i64)
        .context("Не удалось определить Telegram ID")?;

    match text {
        "🏠 Подобрать недвижимость" => {
            start_search(
                bot,
                dialogue,
                message,
                telegram_id,
                state,
            )
            .await?;
        }

        "⭐ Избранное" => {
            show_favorites(
                bot,
                message,
                telegram_id,
                state,
            )
            .await?;
        }

        "👤 Профиль" => {
            show_profile(
                bot,
                message,
                telegram_id,
                state,
            )
            .await?;
        }

        "ℹ Помощь" => {
            show_help(bot, message).await?;
        }

        _ => {
            handle_dialogue_message(
                bot,
                dialogue,
                message,
                state,
            )
            .await?;
        }
    }

    Ok(())
}

/// Обрабатывает callback-кнопки.
pub async fn callback_handler(
    bot: Bot,
    dialogue: MyDialogue,
    query: CallbackQuery,
    state: Arc<AppState>,
) -> HandlerResult {
    let data = match query.data.as_deref() {
        Some(data) => data.to_owned(),
        None => return Ok(()),
    };

    let user_id = query.from.id.0 as i64;

    bot.answer_callback_query(query.id.clone())
        .await
        .ok();

    if let Some(value) = data.strip_prefix("city:") {
        dialogue
            .update(RegistrationState::WaitingDistrict)
            .await?;

        if let Some(profile) = get_or_create_profile(
            &state,
            user_id,
            query.from.first_name.clone(),
            query.from.username.clone(),
        )
        .await?
        {
            let mut profile = profile;
            profile.city = Some(value.to_owned());

            operations::update_profile(
                &state.db,
                &profile,
            )
            .await?;

            if let Some(message) = query.message.as_ref() {
                bot.edit_message_text(
                    message.chat().id,
                    message.id(),
                    "📍 Отлично. Теперь выберите район:",
                )
                .reply_markup(district_keyboard())
                .await?;
            }
        }

        return Ok(());
    }

    if let Some(value) = data.strip_prefix("district:") {
        dialogue
            .update(RegistrationState::WaitingBudget)
            .await?;

        let profile = get_profile_for_user(
            &state,
            user_id,
        )
        .await?;

        let mut profile = profile
            .context("Профиль пользователя не найден")?;

        profile.district = Some(value.to_owned());

        operations::update_profile(
            &state.db,
            &profile,
        )
        .await?;

        if let Some(message) = query.message.as_ref() {
            bot.edit_message_text(
                message.chat().id,
                message.id(),
                "💰 Какой максимальный бюджет?",
            )
            .reply_markup(budget_keyboard())
            .await?;
        }

        return Ok(());
    }

    if let Some(value) = data.strip_prefix("budget:") {
        let budget = value
            .parse::<i64>()
            .context("Некорректное значение бюджета")?;

        dialogue
            .update(RegistrationState::WaitingRooms)
            .await?;

        let profile = get_profile_for_user(
            &state,
            user_id,
        )
        .await?;

        let mut profile = profile
            .context("Профиль пользователя не найден")?;

        profile.budget = Some(budget);

        operations::update_profile(
            &state.db,
            &profile,
        )
        .await?;

        if let Some(message) = query.message.as_ref() {
            bot.edit_message_text(
                message.chat().id,
                message.id(),
                "🛏 Сколько комнат вам нужно?",
            )
            .reply_markup(rooms_keyboard())
            .await?;
        }

        return Ok(());
    }

    if let Some(value) = data.strip_prefix("rooms:") {
        let rooms = value
            .parse::<i32>()
            .context("Некорректное количество комнат")?;

        dialogue
            .update(RegistrationState::WaitingAdditionalRequirements)
            .await?;

        let profile = get_profile_for_user(
            &state,
            user_id,
        )
        .await?;

        let mut profile = profile
            .context("Профиль пользователя не найден")?;

        profile.rooms = Some(rooms);

        operations::update_profile(
            &state.db,
            &profile,
        )
        .await?;

        if let Some(message) = query.message.as_ref() {
            bot.edit_message_text(
                message.chat().id,
                message.id(),
                "📝 Напишите дополнительные требования.\n\n\
                 Например: метро рядом, балкон, ремонт.\n\n\
                 Если дополнительных требований нет, напишите «нет».",
            )
            .await?;
        }

        return Ok(());
    }

    if let Some(property_id) = data.strip_prefix("favorite:") {
        let property_id = Uuid::parse_str(property_id)
            .context("Некорректный ID объекта")?;

        let user = get_user(
            &state,
            user_id,
        )
        .await?
        .context("Пользователь не найден")?;

        let favorite = Favorite {
            id: Uuid::new_v4(),
            user_id: user.id,
            property_id,
            created_at: Utc::now(),
        };

        operations::add_to_favorites(
            &state.db,
            &favorite,
        )
        .await?;

        if let Some(message) = query.message.as_ref() {
            bot.answer_callback_query(query.id)
                .text("⭐ Объект добавлен в избранное")
                .show_alert(false)
                .await
                .ok();

            bot.edit_message_reply_markup(
                message.chat().id,
                message.id(),
            )
            .reply_markup(property_keyboard(&property_id.to_string()))
            .await
            .ok();
        }

        return Ok(());
    }

    if let Some(property_id) =
        data.strip_prefix("favorite_remove:")
    {
        let property_id = Uuid::parse_str(property_id)
            .context("Некорректный ID объекта")?;

        let user = get_user(
            &state,
            user_id,
        )
        .await?
        .context("Пользователь не найден")?;

        remove_favorite(
            &state,
            user.id,
            property_id,
        )
        .await?;

        if let Some(message) = query.message.as_ref() {
            bot.edit_message_text(
                message.chat().id,
                message.id(),
                "🗑 Объект удалён из избранного.",
            )
            .await?;
        }

        return Ok(());
    }

    if let Some(property_id) = data.strip_prefix("view:") {
        let property_id = Uuid::parse_str(property_id)
            .context("Некорректный ID объекта")?;

        create_viewing_request(
            bot,
            query.message.as_ref(),
            user_id,
            property_id,
            state,
        )
        .await?;

        return Ok(());
    }

    if let Some(property_id) = data.strip_prefix("next:") {
        let property_id = Uuid::parse_str(property_id)
            .context("Некорректный ID объекта")?;

        show_next_property(
            bot,
            query.message.as_ref(),
            user_id,
            property_id,
            state,
        )
        .await?;

        return Ok(());
    }

    Ok(())
}

/// Обработка состояний FSM.
async fn handle_dialogue_message(
    bot: Bot,
    dialogue: MyDialogue,
    message: Message,
    state: Arc<AppState>,
) -> HandlerResult {
    let current_state = dialogue.get().await?;

    match current_state {
        Some(RegistrationState::WaitingPhone) => {
            handle_phone(
                bot,
                dialogue,
                message,
                state,
            )
            .await?;
        }

        Some(RegistrationState::WaitingAdditionalRequirements) => {
            handle_additional_requirements(
                bot,
                dialogue,
                message,
                state,
            )
            .await?;
        }

        Some(RegistrationState::Completed) => {
            bot.send_message(
                message.chat.id,
                "Анкета уже заполнена. Используйте /search для нового поиска.",
            )
            .reply_markup(main_menu())
            .await?;
        }

        _ => {
            bot.send_message(
                message.chat.id,
                "Используйте меню или команду /start.",
            )
            .reply_markup(main_menu())
            .await?;
        }
    }

    Ok(())
}

/// Запускает регистрацию пользователя.
async fn start(
    bot: Bot,
    dialogue: MyDialogue,
    message: Message,
    telegram_id: i64,
    state: Arc<AppState>,
) -> HandlerResult {
    let tg_user = message
        .from
        .as_ref()
        .context("Пользователь отсутствует")?;

    let existing_user =
        operations::get_user_by_telegram_id(
            &state.db,
            telegram_id,
        )
        .await?;

    if existing_user.is_none() {
        let user = User {
            id: Uuid::new_v4(),
            telegram_id,
            username: tg_user.username.clone(),
            first_name: tg_user.first_name.clone(),
            last_name: tg_user.last_name.clone(),
            phone: None,
            created_at: Utc::now(),
        };

        operations::create_user(
            &state.db,
            &user,
        )
        .await?;

        let profile = UserProfile {
            id: Uuid::new_v4(),
            user_id: user.id,
            city: None,
            district: None,
            budget: None,
            rooms: None,
            additional_requirements: None,
        };

        operations::create_profile(
            &state.db,
            &profile,
        )
        .await?;

        dialogue
            .update(RegistrationState::WaitingPhone)
            .await?;

        bot.send_message(
            message.chat.id,
            format!(
                "👋 Привет, {}!\n\n\
                 Добро пожаловать в сервис подбора недвижимости.\n\n\
                 Для начала отправьте свой номер телефона.",
                tg_user.first_name
            ),
        )
        .reply_markup(request_phone())
        .await?;

        return Ok(());
    }

    dialogue
        .update(RegistrationState::Completed)
        .await?;

    bot.send_message(
        message.chat.id,
        "👋 С возвращением!\n\nВыберите нужное действие:",
    )
    .reply_markup(main_menu())
    .await?;

    Ok(())
}

/// Обрабатывает номер телефона.
async fn handle_phone(
    bot: Bot,
    dialogue: MyDialogue,
    message: Message,
    state: Arc<AppState>,
) -> HandlerResult {
    let telegram_id = message
        .from
        .as_ref()
        .context("Пользователь отсутствует")?
        .id
        .0 as i64;

    let phone = if let Some(contact) = message.contact() {
        contact.phone_number.clone()
    } else if let Some(text) = message.text() {
        text.trim().to_owned()
    } else {
        bot.send_message(
            message.chat.id,
            "Пожалуйста, отправьте номер телефона.",
        )
        .reply_markup(request_phone())
        .await?;

        return Ok(());
    };

    let user = operations::get_user_by_telegram_id(
        &state.db,
        telegram_id,
    )
    .await?
    .context("Пользователь не найден")?;

    sqlx::query(
        r#"
        UPDATE users
        SET phone = $2
        WHERE id = $1
        "#,
    )
    .bind(user.id)
    .bind(&phone)
    .execute(&state.db)
    .await?;

    dialogue
        .update(RegistrationState::WaitingCity)
        .await?;

    bot.send_message(
        message.chat.id,
        "🏙 Теперь выберите город:",
    )
    .reply_markup(KeyboardRemove::new())
    .await?;

    bot.send_message(
        message.chat.id,
        "Выберите город:",
    )
    .reply_markup(city_keyboard())
    .await?;

    Ok(())
}

/// Запускает новый поиск недвижимости.
async fn start_search(
    bot: Bot,
    dialogue: MyDialogue,
    message: Message,
    telegram_id: i64,
    state: Arc<AppState>,
) -> HandlerResult {
    let user = get_user(
        &state,
        telegram_id,
    )
    .await?;

    if user.is_none() {
        bot.send_message(
            message.chat.id,
            "Сначала зарегистрируйтесь через /start.",
        )
        .await?;

        return Ok(());
    }

    dialogue
        .update(RegistrationState::WaitingCity)
        .await?;

    bot.send_message(
        message.chat.id,
        "🏙 Выберите город для поиска:",
    )
    .reply_markup(city_keyboard())
    .await?;

    Ok(())
}

/// Обрабатывает дополнительные требования.
async fn handle_additional_requirements(
    bot: Bot,
    dialogue: MyDialogue,
    message: Message,
    state: Arc<AppState>,
) -> HandlerResult {
    let telegram_id = message
        .from
        .as_ref()
        .context("Пользователь отсутствует")?
        .id
        .0 as i64;

    let text = message
        .text()
        .unwrap_or("нет")
        .trim();

    let user = get_user(
        &state,
        telegram_id,
    )
    .await?
    .context("Пользователь не найден")?;

    let profile = operations::get_profile(
        &state.db,
        user.id,
    )
    .await?
    .context("Профиль пользователя не найден")?;

    let mut profile = profile;

    profile.additional_requirements =
        if text.eq_ignore_ascii_case("нет") {
            None
        } else {
            Some(text.to_owned())
        };

    operations::update_profile(
        &state.db,
        &profile,
    )
    .await?;

    dialogue
        .update(RegistrationState::Completed)
        .await?;

    bot.send_message(
        message.chat.id,
        "✅ Параметры сохранены!\n\nИщу подходящие варианты...",
    )
    .reply_markup(main_menu())
    .await?;

    search_and_show_first(
        bot,
        message.chat.id,
        user.id,
        state,
    )
    .await?;

    Ok(())
}

/// Выполняет поиск и показывает первый объект.
async fn search_and_show_first(
    bot: Bot,
    chat_id: ChatId,
    user_id: Uuid,
    state: Arc<AppState>,
) -> HandlerResult {
    let profile = operations::get_profile(
        &state.db,
        user_id,
    )
    .await?
    .context("Профиль пользователя не найден")?;

    let city = profile
        .city
        .context("Город не указан")?;

    let district = profile
        .district
        .context("Район не указан")?;

    let budget = profile
        .budget
        .context("Бюджет не указан")?;

    let rooms = profile
        .rooms
        .context("Количество комнат не указано")?;

    let properties = operations::search_properties(
        &state.db,
        &city,
        &district,
        budget,
        rooms,
    )
    .await?;

    if properties.is_empty() {
        bot.send_message(
            chat_id,
            "😔 По вашим параметрам ничего не найдено.\n\n\
             Попробуйте изменить район, бюджет или количество комнат.",
        )
        .await?;

        return Ok(());
    }

    let property = &properties[0];

    send_property_card(
        &bot,
        chat_id,
        property,
    )
    .await?;

    Ok(())
}

/// Показывает карточку объекта.
async fn send_property_card(
    bot: &Bot,
    chat_id: ChatId,
    property: &crate::models::Property,
) -> HandlerResult {
    let text = format!(
        "🏠 {}\n\n\
         {}\n\n\
         💰 Цена: {} ₽\n\
         🛏 Комнат: {}\n\
         📐 Площадь: {:.1} м²\n\
         📍 {}, {}\n\n\
         ID: {}",
        property.title,
        property.description,
        format_price(property.price),
        property.rooms,
        property.area,
        property.city,
        property.district,
        property.id,
    );

    let keyboard =
        property_keyboard(&property.id.to_string());

    match &property.photo_url {
        Some(photo_url) if !photo_url.trim().is_empty() => {
            bot.send_photo(
                chat_id,
                InputFile::url(
                    photo_url
                        .parse()
                        .context("Некорректный URL фотографии")?,
                ),
            )
            .caption(text)
            .reply_markup(keyboard)
            .await?;
        }

        _ => {
            bot.send_message(
                chat_id,
                text,
            )
            .reply_markup(keyboard)
            .await?;
        }
    }

    Ok(())
}

/// Показывает следующий объект.
///
/// Для простоты используется поиск по всем активным объектам
/// и выбирается следующий после текущего ID.
async fn show_next_property(
    bot: Bot,
    message: Option<&teloxide::types::MaybeInaccessibleMessage>,
    telegram_id: i64,
    current_property_id: Uuid,
    state: Arc<AppState>,
) -> HandlerResult {
    let message = match message {
        Some(message) => message,
        None => return Ok(()),
    };

    let user = get_user(
        &state,
        telegram_id,
    )
    .await?
    .context("Пользователь не найден")?;

    let profile = operations::get_profile(
        &state.db,
        user.id,
    )
    .await?
    .context("Профиль пользователя не найден")?;

    let city = match profile.city {
        Some(value) => value,
        None => return Ok(()),
    };

    let district = match profile.district {
        Some(value) => value,
        None => return Ok(()),
    };

    let budget = match profile.budget {
        Some(value) => value,
        None => return Ok(()),
    };

    let rooms = match profile.rooms {
        Some(value) => value,
        None => return Ok(()),
    };

    let properties = operations::search_properties(
        &state.db,
        &city,
        &district,
        budget,
        rooms,
    )
    .await?;

    let current_index = properties
        .iter()
        .position(|property| property.id == current_property_id);

    let next = match current_index {
        Some(index) => properties.get(index + 1),
        None => properties.first(),
    };

    let next = match next {
        Some(property) => property,
        None => {
            bot.send_message(
                message.chat().id,
                "Это был последний подходящий объект.",
            )
            .await?;

            return Ok(());
        }
    };

    send_property_card(
        &bot,
        message.chat().id,
        next,
    )
    .await?;

    Ok(())
}

/// Показывает профиль пользователя.
async fn show_profile(
    bot: Bot,
    message: Message,
    telegram_id: i64,
    state: Arc<AppState>,
) -> HandlerResult {
    let user = get_user(
        &state,
        telegram_id,
    )
    .await?;

    let user = match user {
        Some(user) => user,
        None => {
            bot.send_message(
                message.chat.id,
                "Профиль не найден. Используйте /start.",
            )
            .await?;

            return Ok(());
        }
    };

    let profile = operations::get_profile(
        &state.db,
        user.id,
    )
    .await?;

    let profile_text = match profile {
        Some(profile) => format!(
            "👤 <b>Ваш профиль</b>\n\n\
             Имя: {}\n\
             Телефон: {}\n\
             Город: {}\n\
             Район: {}\n\
             Бюджет: {}\n\
             Комнат: {}\n\
             Дополнительно: {}",
            user.first_name,
            user.phone.as_deref().unwrap_or("не указан"),
            profile.city.as_deref().unwrap_or("не указан"),
            profile.district.as_deref().unwrap_or("не указан"),
            profile
                .budget
                .map(format_price)
                .unwrap_or_else(|| "не указан".to_owned()),
            profile
                .rooms
                .map(|rooms| rooms.to_string())
                .unwrap_or_else(|| "не указано".to_owned()),
            profile
                .additional_requirements
                .as_deref()
                .unwrap_or("нет"),
        ),

        None => format!(
            "👤 <b>Ваш профиль</b>\n\nИмя: {}",
            user.first_name
        ),
    };

    bot.send_message(
        message.chat.id,
        profile_text,
    )
    .parse_mode(teloxide::types::ParseMode::Html)
    .reply_markup(main_menu())
    .await?;

    Ok(())
}

/// Показывает избранные объекты.
async fn show_favorites(
    bot: Bot,
    message: Message,
    telegram_id: i64,
    state: Arc<AppState>,
) -> HandlerResult {
    let user = get_user(
        &state,
        telegram_id,
    )
    .await?;

    let user = match user {
        Some(user) => user,
        None => {
            bot.send_message(
                message.chat.id,
                "Сначала зарегистрируйтесь через /start.",
            )
            .await?;

            return Ok(());
        }
    };

    let favorites = operations::get_favorites(
        &state.db,
        user.id,
    )
    .await?;

    if favorites.is_empty() {
        bot.send_message(
            message.chat.id,
            "⭐ В избранном пока ничего нет.",
        )
        .reply_markup(main_menu())
        .await?;

        return Ok(());
    }

    bot.send_message(
        message.chat.id,
        format!(
            "⭐ В избранном объектов: {}",
            favorites.len()
        ),
    )
    .await?;

    for property in favorites {
        let text = format!(
            "🏠 {}\n\n\
             💰 {} ₽\n\
             🛏 {} комнат\n\
             📐 {:.1} м²\n\
             📍 {}, {}",
            property.title,
            format_price(property.price),
            property.rooms,
            property.area,
            property.city,
            property.district,
        );

        bot.send_message(
            message.chat.id,
            text,
        )
        .reply_markup(
            favorites_keyboard(
                &property.id.to_string()
            )
        )
        .await?;
    }

    Ok(())
}

/// Создаёт заявку на просмотр.
async fn create_viewing_request(
    bot: Bot,
    message: Option<&teloxide::types::MaybeInaccessibleMessage>,
    telegram_id: i64,
    property_id: Uuid,
    state: Arc<AppState>,
) -> HandlerResult {
    let user = get_user(
        &state,
        telegram_id,
    )
    .await?
    .context("Пользователь не найден")?;

    let property = sqlx::query_as::<
        _,
        crate::models::Property,
    >(
        r#"
        SELECT *
        FROM properties
        WHERE id = $1
        "#,
    )
    .bind(property_id)
    .fetch_optional(&state.db)
    .await?
    .context("Объект недвижимости не найден")?;

    let request = ViewingRequest {
        id: Uuid::new_v4(),
        user_id: user.id,
        property_id,
        status: RequestStatus::New,
        comment: None,
        created_at: Utc::now(),
    };

    operations::create_viewing_request(
        &state.db,
        &request,
    )
    .await?;

    // Отправляем уведомление менеджеру.
    let notification = format!(
        "📨 <b>Новая заявка на просмотр</b>\n\n\
         👤 Клиент: {} {}\n\
         🆔 Telegram ID: {}\n\
         📱 Телефон: {}\n\n\
         🏠 Объект: {}\n\
         💰 Цена: {} ₽\n\
         📍 {}, {}\n\
         🆔 ID заявки: {}",
        user.first_name,
        user.last_name.as_deref().unwrap_or(""),
        user.telegram_id,
        user.phone.as_deref().unwrap_or("не указан"),
        property.title,
        format_price(property.price),
        property.city,
        property.district,
        request.id,
    );

    bot.send_message(
        ChatId(state.config.admin_chat_id),
        notification,
    )
    .parse_mode(teloxide::types::ParseMode::Html)
    .await?;

    if let Some(message) = message {
        bot.send_message(
            message.chat().id,
            "✅ Заявка отправлена менеджеру!\n\n\
             С вами свяжутся для согласования времени просмотра.",
        )
        .await?;
    }

    Ok(())
}

/// Получает пользователя по Telegram ID.
async fn get_user(
    state: &AppState,
    telegram_id: i64,
) -> Result<Option<User>> {
    Ok(
        operations::get_user_by_telegram_id(
            &state.db,
            telegram_id,
        )
        .await?
    )
}

/// Получает существующий профиль.
async fn get_profile_for_user(
    state: &AppState,
    telegram_id: i64,
) -> Result<Option<UserProfile>> {
    let user = get_user(
        state,
        telegram_id,
    )
    .await?;

    match user {
        Some(user) => {
            operations::get_profile(
                &state.db,
                user.id,
            )
            .await
        }

        None => Ok(None),
    }
}

/// Получает профиль или создаёт его.
///
/// Используется callback-обработчиками, чтобы не дублировать
/// код создания пользователя.
async fn get_or_create_profile(
    state: &AppState,
    telegram_id: i64,
    first_name: String,
    username: Option<String>,
) -> Result<Option<UserProfile>> {
    let user = match get_user(
        state,
        telegram_id,
    )
    .await? {
        Some(user) => user,

        None => {
            let user = User {
                id: Uuid::new_v4(),
                telegram_id,
                username,
                first_name,
                last_name: None,
                phone: None,
                created_at: Utc::now(),
            };

            operations::create_user(
                &state.db,
                &user,
            )
            .await?;

            user
        }
    };

    let profile = operations::get_profile(
        &state.db,
        user.id,
    )
    .await?;

    if let Some(profile) = profile {
        return Ok(Some(profile));
    }

    let profile = UserProfile {
        id: Uuid::new_v4(),
        user_id: user.id,
        city: None,
        district: None,
        budget: None,
        rooms: None,
        additional_requirements: None,
    };

    operations::create_profile(
        &state.db,
        &profile,
    )
    .await?;

    Ok(Some(profile))
}

/// Удаляет объект из избранного.
async fn remove_favorite(
    state: &AppState,
    user_id: Uuid,
    property_id: Uuid,
) -> Result<()> {
    sqlx::query(
        r#"
        DELETE FROM favorites
        WHERE user_id = $1
          AND property_id = $2
        "#,
    )
    .bind(user_id)
    .bind(property_id)
    .execute(&state.db)
    .await?;

    Ok(())
}

/// Форматирует цену с разделением тысяч.
fn format_price(price: i64) -> String {
    let mut value = price.abs().to_string();
    let mut result = String::new();

    while value.len() > 3 {
        let split = value.split_off(value.len() - 3);

        if result.is_empty() {
            result = split;
        } else {
            result = format!("{} {}", split, result);
        }
    }

    if result.is_empty() {
        result = value;
    } else {
        result = format!("{} {}", value, result);
    }

    if price < 0 {
        format!("-{}", result)
    } else {
        result
    }
}

/// Показывает справку.
async fn show_help(
    bot: Bot,
    message: Message,
) -> HandlerResult {
    bot.send_message(
        message.chat.id,
        "ℹ <b>Помощь</b>\n\n\
         /start — регистрация\n\
         /search — подбор недвижимости\n\
         /profile — ваш профиль\n\
         /favorites — избранные объекты\n\
         /help — помощь\n\n\
         После выбора объекта вы можете добавить его \
         в избранное или отправить заявку на просмотр.",
    )
    .parse_mode(teloxide::types::ParseMode::Html)
    .reply_markup(main_menu())
    .await?;

    Ok(())
}