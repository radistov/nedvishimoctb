use std::sync::Arc;

use anyhow::{Context, Result};
use teloxide::{
    dispatching::dialogue::GetChatId,
    prelude::*,
    types::{
        CallbackQuery, ChatId, InputFile, KeyboardRemove, Message, ParseMode,
    },
};

use uuid::Uuid;

use crate::{
    bot::{
        keyboards,
        mod_::{HandlerResult, MyDialogue},
    },
    db::operations,
    models::{Command, RegistrationState},
    AppState,
};

pub async fn command_handler(
    bot: Bot,
    dialogue: MyDialogue,
    message: Message,
    command: Command,
    state: Arc<AppState>,
) -> HandlerResult {
    match command {
        Command::Start => start(bot, dialogue, message, state).await?,
        Command::Profile => show_profile(bot, message, state).await?,
        Command::Search => start_search(bot, dialogue, message, state).await?,
        Command::Favorites => show_favorites(bot, message, state).await?,
        Command::Help => show_help(bot, message).await?,
    }

    Ok(())
}

pub async fn message_handler(
    bot: Bot,
    dialogue: MyDialogue,
    message: Message,
    state: Arc<AppState>,
) -> HandlerResult {
    let text = message.text().unwrap_or_default();

    match text {
        "🏠 Подобрать недвижимость" => {
            start_search(bot, dialogue, message, state).await?
        }
        "⭐ Избранное" => show_favorites(bot, message, state).await?,
        "👤 Профиль" => show_profile(bot, message, state).await?,
        "ℹ️ Помощь" => show_help(bot, message).await?,
        _ => handle_dialogue_message(bot, dialogue, message, state).await?,
    }

    Ok(())
}

pub async fn callback_handler(
    bot: Bot,
    dialogue: MyDialogue,
    query: CallbackQuery,
    state: Arc<AppState>,
) -> HandlerResult {
    let Some(data) = query.data.as_deref() else {
        return Ok(());
    };

    bot.answer_callback_query(query.id.clone()).await?;

    let Some(message) = query.message.as_ref() else {
        return Ok(());
    };

    let chat_id = message.chat().id;

    if let Some(value) = data.strip_prefix("city:") {
        handle_city(
            bot,
            dialogue,
            chat_id,
            value.to_owned(),
            state,
        )
        .await?;
    } else if let Some(value) = data.strip_prefix("district:") {
        handle_district(
            bot,
            dialogue,
            chat_id,
            value.to_owned(),
            state,
        )
        .await?;
    } else if let Some(value) = data.strip_prefix("budget:") {
        handle_budget(
            bot,
            dialogue,
            chat_id,
            value,
            state,
        )
        .await?;
    } else if let Some(value) = data.strip_prefix("rooms:") {
        handle_rooms(
            bot,
            dialogue,
            chat_id,
            value,
            state,
        )
        .await?;
    } else if let Some(value) = data.strip_prefix("favorite:") {
        add_favorite(
            bot,
            chat_id,
            value,
            state,
        )
        .await?;
    } else if let Some(value) = data.strip_prefix("favorite_remove:") {
        remove_favorite(
            bot,
            chat_id,
            value,
            state,
        )
        .await?;
    } else if let Some(value) = data.strip_prefix("view:") {
        create_viewing_request(
            bot,
            chat_id,
            value,
            state,
        )
        .await?;
    } else if let Some(value) = data.strip_prefix("next:") {
        show_next_property(
            bot,
            chat_id,
            value,
            state,
        )
        .await?;
    }

    Ok(())
}

async fn start(
    bot: Bot,
    dialogue: MyDialogue,
    message: Message,
    state: Arc<AppState>,
) -> Result<()> {
    let Some(user) = message.from.as_ref() else {
        return Ok(());
    };

    let telegram_id = user.id.0 as i64;

    let db_user = match operations::get_user_by_telegram_id(
        &state.db,
        telegram_id,
    )
    .await?
    {
        Some(user) => user,
        None => {
            operations::create_user(
                &state.db,
                telegram_id,
                user.username.as_deref(),
                &user.first_name,
                user.last_name.as_deref(),
            )
            .await?
        }
    };

    if operations::get_profile(&state.db, db_user.id)
        .await?
        .is_none()
    {
        operations::create_profile(&state.db, db_user.id).await?;
    }

    if db_user.phone.is_none() {
        dialogue
            .update(RegistrationState::WaitingPhone)
            .await?;

        bot.send_message(
            message.chat.id,
            "Здравствуйте! 👋\n\nДля начала отправьте номер телефона.",
        )
        .reply_markup(keyboards::request_phone())
        .await?;
    } else {
        dialogue
            .update(RegistrationState::Completed)
            .await?;

        bot.send_message(
            message.chat.id,
            "С возвращением! 👋\n\nВыберите нужное действие:",
        )
        .reply_markup(keyboards::main_menu())
        .await?;
    }

    Ok(())
}

async fn start_search(
    bot: Bot,
    dialogue: MyDialogue,
    message: Message,
    state: Arc<AppState>,
) -> Result<()> {
    let Some(user) = get_user(&state, &message).await? else {
        bot.send_message(
            message.chat.id,
            "Сначала выполните команду /start.",
        )
        .await?;

        return Ok(());
    };

    if user.phone.is_none() {
        dialogue
            .update(RegistrationState::WaitingPhone)
            .await?;

        bot.send_message(
            message.chat.id,
            "Перед поиском необходимо указать номер телефона.",
        )
        .reply_markup(keyboards::request_phone())
        .await?;

        return Ok(());
    }

    get_or_create_profile(&state, user.id).await?;

    dialogue
        .update(RegistrationState::WaitingCity)
        .await?;

    bot.send_message(
        message.chat.id,
        "🏙 Выберите город:",
    )
    .reply_markup(keyboards::city_keyboard())
    .await?;

    Ok(())
}

async fn handle_dialogue_message(
    bot: Bot,
    dialogue: MyDialogue,
    message: Message,
    state: Arc<AppState>,
) -> Result<()> {
    let current_state = dialogue.get().await?.unwrap_or_default();

    match current_state {
        RegistrationState::WaitingPhone => {
            handle_phone(bot, dialogue, message, state).await?
        }
        RegistrationState::WaitingAdditionalRequirements => {
            handle_additional_requirements(
                bot,
                dialogue,
                message,
                state,
            )
            .await?
        }
        RegistrationState::Completed => {
            bot.send_message(
                message.chat.id,
                "Используйте кнопки меню или команды /search, /profile, /favorites.",
            )
            .reply_markup(keyboards::main_menu())
            .await?;
        }
        _ => {
            bot.send_message(
                message.chat.id,
                "Пожалуйста, выберите вариант с помощью кнопок.",
            )
            .await?;
        }
    }

    Ok(())
}

async fn handle_phone(
    bot: Bot,
    dialogue: MyDialogue,
    message: Message,
    state: Arc<AppState>,
) -> Result<()> {
    let Some(from) = message.from.as_ref() else {
        return Ok(());
    };

    let phone = if let Some(contact) = message.contact() {
        contact.phone_number.clone()
    } else if let Some(text) = message.text() {
        text.trim().to_owned()
    } else {
        String::new()
    };

    if phone.is_empty() {
        bot.send_message(
            message.chat.id,
            "Не удалось получить номер. Отправьте его через кнопку ниже.",
        )
        .reply_markup(keyboards::request_phone())
        .await?;

        return Ok(());
    }

    let Some(user) = operations::get_user_by_telegram_id(
        &state.db,
        from.id.0 as i64,
    )
    .await?
    else {
        bot.send_message(
            message.chat.id,
            "Пользователь не найден. Выполните /start.",
        )
        .await?;

        return Ok(());
    };

    operations::update_user_phone(
        &state.db,
        user.id,
        &phone,
    )
    .await?;

    dialogue
        .update(RegistrationState::WaitingCity)
        .await?;

    bot.send_message(
        message.chat.id,
        "Телефон сохранён ✅\n\nТеперь выберите город:",
    )
    .reply_markup(KeyboardRemove::new())
    .await?;

    bot.send_message(
        message.chat.id,
        "🏙 Город:",
    )
    .reply_markup(keyboards::city_keyboard())
    .await?;

    Ok(())
}

async fn handle_city(
    bot: Bot,
    dialogue: MyDialogue,
    chat_id: ChatId,
    city: String,
    state: Arc<AppState>,
) -> Result<()> {
    let Some(user) = get_user_by_chat_id(&state, chat_id).await? else {
        return Ok(());
    };

    let profile = get_or_create_profile(&state, user.id).await?;

    operations::update_profile(
        &state.db,
        user.id,
        Some(&city),
        profile.district.as_deref(),
        profile.budget,
        profile.rooms,
        profile.additional_requirements.as_deref(),
    )
    .await?;

    dialogue
        .update(RegistrationState::WaitingDistrict)
        .await?;

    bot.send_message(
        chat_id,
        format!("Город: {city} ✅\n\nВыберите район:"),
    )
    .reply_markup(keyboards::district_keyboard())
    .await?;

    Ok(())
}

async fn handle_district(
    bot: Bot,
    dialogue: MyDialogue,
    chat_id: ChatId,
    district: String,
    state: Arc<AppState>,
) -> Result<()> {
    let Some(user) = get_user_by_chat_id(&state, chat_id).await? else {
        return Ok(());
    };

    let profile = get_or_create_profile(&state, user.id).await?;

    operations::update_profile(
        &state.db,
        user.id,
        profile.city.as_deref(),
        Some(&district),
        profile.budget,
        profile.rooms,
        profile.additional_requirements.as_deref(),
    )
    .await?;

    dialogue
        .update(RegistrationState::WaitingBudget)
        .await?;

    bot.send_message(
        chat_id,
        format!("Район: {district} ✅\n\nВыберите максимальный бюджет:"),
    )
    .reply_markup(keyboards::budget_keyboard())
    .await?;

    Ok(())
}

async fn handle_budget(
    bot: Bot,
    dialogue: MyDialogue,
    chat_id: ChatId,
    value: &str,
    state: Arc<AppState>,
) -> Result<()> {
    let budget = value
        .parse::<i64>()
        .context("Некорректный бюджет")?;

    let Some(user) = get_user_by_chat_id(&state, chat_id).await? else {
        return Ok(());
    };

    let profile = get_or_create_profile(&state, user.id).await?;

    operations::update_profile(
        &state.db,
        user.id,
        profile.city.as_deref(),
        profile.district.as_deref(),
        Some(budget),
        profile.rooms,
        profile.additional_requirements.as_deref(),
    )
    .await?;

    dialogue
        .update(RegistrationState::WaitingRooms)
        .await?;

    bot.send_message(
        chat_id,
        format!(
            "Бюджет до {} ₽ ✅\n\nСколько комнат нужно?",
            format_price(budget)
        ),
    )
    .reply_markup(keyboards::rooms_keyboard())
    .await?;

    Ok(())
}

async fn handle_rooms(
    bot: Bot,
    dialogue: MyDialogue,
    chat_id: ChatId,
    value: &str,
    state: Arc<AppState>,
) -> Result<()> {
    let rooms = value
        .parse::<i32>()
        .context("Некорректное количество комнат")?;

    let Some(user) = get_user_by_chat_id(&state, chat_id).await? else {
        return Ok(());
    };

    let profile = get_or_create_profile(&state, user.id).await?;

    operations::update_profile(
        &state.db,
        user.id,
        profile.city.as_deref(),
        profile.district.as_deref(),
        profile.budget,
        Some(rooms),
        profile.additional_requirements.as_deref(),
    )
    .await?;

    dialogue
        .update(RegistrationState::WaitingAdditionalRequirements)
        .await?;

    bot.send_message(
        chat_id,
        "Есть дополнительные пожелания?\n\nНапишите их сообщением или отправьте «нет».",
    )
    .await?;

    Ok(())
}

async fn handle_additional_requirements(
    bot: Bot,
    dialogue: MyDialogue,
    message: Message,
    state: Arc<AppState>,
) -> Result<()> {
    let Some(user) = get_user(&state, &message).await? else {
        return Ok(());
    };

    let text = message.text().unwrap_or_default().trim();

    let requirements = if text.eq_ignore_ascii_case("нет") {
        None
    } else if text.is_empty() {
        None
    } else {
        Some(text)
    };

    let profile = get_or_create_profile(&state, user.id).await?;

    operations::update_profile(
        &state.db,
        user.id,
        profile.city.as_deref(),
        profile.district.as_deref(),
        profile.budget,
        profile.rooms,
        requirements,
    )
    .await?;

    dialogue
        .update(RegistrationState::Completed)
        .await?;

    bot.send_message(
        message.chat.id,
        "Ищу подходящие варианты... 🔎",
    )
    .await?;

    search_and_show_first(
        &bot,
        message.chat.id,
        user.id,
        &state,
    )
    .await?;

    Ok(())
}

async fn search_and_show_first(
    bot: &Bot,
    chat_id: ChatId,
    user_id: Uuid,
    state: &Arc<AppState>,
) -> Result<()> {
    let Some(profile) = operations::get_profile(&state.db, user_id).await? else {
        bot.send_message(chat_id, "Профиль ещё не заполнен.")
            .await?;

        return Ok(());
    };

    let (Some(city), Some(district), Some(budget), Some(rooms)) = (
        profile.city,
        profile.district,
        profile.budget,
        profile.rooms,
    ) else {
        bot.send_message(
            chat_id,
            "Не удалось определить все параметры поиска.",
        )
        .await?;

        return Ok(());
    };

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
            "К сожалению, подходящих вариантов пока нет. 😔\n\nПопробуйте изменить параметры поиска.",
        )
        .reply_markup(keyboards::main_menu())
        .await?;

        return Ok(());
    }

    send_property_card(
        bot,
        chat_id,
        &properties[0],
    )
    .await?;

    Ok(())
}

async fn show_next_property(
    bot: Bot,
    chat_id: ChatId,
    current_property_id: &str,
    state: Arc<AppState>,
) -> Result<()> {
    let current_id = Uuid::parse_str(current_property_id)
        .context("Некорректный ID объекта")?;

    let Some(user) = get_user_by_chat_id(&state, chat_id).await? else {
        return Ok(());
    };

    let Some(profile) = operations::get_profile(&state.db, user.id).await? else {
        return Ok(());
    };

    let (Some(city), Some(district), Some(budget), Some(rooms)) = (
        profile.city,
        profile.district,
        profile.budget,
        profile.rooms,
    ) else {
        return Ok(());
    };

    let properties = operations::search_properties(
        &state.db,
        &city,
        &district,
        budget,
        rooms,
    )
    .await?;

    let Some(current_index) = properties
        .iter()
        .position(|property| property.id == current_id)
    else {
        return Ok(());
    };

    let Some(next_property) = properties.get(current_index + 1) else {
        bot.send_message(
            chat_id,
            "Это последний подходящий вариант. 🏠",
        )
        .await?;

        return Ok(());
    };

    send_property_card(
        &bot,
        chat_id,
        next_property,
    )
    .await?;

    Ok(())
}

async fn send_property_card(
    bot: &Bot,
    chat_id: ChatId,
    property: &crate::models::Property,
) -> Result<()> {
    let text = format!(
        "🏠 <b>{}</b>\n\n\
         {}\n\n\
         📍 {} — {}\n\
         💰 {} ₽\n\
         🛏 Комнат: {}\n\
         📐 Площадь: {:.1} м²",
        escape_html(&property.title),
        escape_html(&property.description),
        escape_html(&property.city),
        escape_html(&property.district),
        format_price(property.price),
        property.rooms,
        property.area,
    );

    if let Some(photo_url) = property.photo_url.as_deref() {
        match photo_url.parse() {
            Ok(url) => {
                bot.send_photo(
                    chat_id,
                    InputFile::url(url),
                )
                .caption(text)
                .parse_mode(ParseMode::Html)
                .reply_markup(keyboards::property_keyboard(property.id))
                .await?;
            }
            Err(_) => {
                bot.send_message(chat_id, text)
                    .parse_mode(ParseMode::Html)
                    .reply_markup(keyboards::property_keyboard(property.id))
                    .await?;
            }
        }
    } else {
        bot.send_message(chat_id, text)
            .parse_mode(ParseMode::Html)
            .reply_markup(keyboards::property_keyboard(property.id))
            .await?;
    }

    Ok(())
}

async fn add_favorite(
    bot: Bot,
    chat_id: ChatId,
    property_id: &str,
    state: Arc<AppState>,
) -> Result<()> {
    let property_id = Uuid::parse_str(property_id)
        .context("Некорректный ID объекта")?;

    let Some(user) = get_user_by_chat_id(&state, chat_id).await? else {
        return Ok(());
    };

    operations::add_to_favorites(
        &state.db,
        user.id,
        property_id,
    )
    .await?;

    bot.send_message(
        chat_id,
        "⭐ Объект добавлен в избранное.",
    )
    .await?;

    Ok(())
}

async fn remove_favorite(
    bot: Bot,
    chat_id: ChatId,
    property_id: &str,
    state: Arc<AppState>,
) -> Result<()> {
    let property_id = Uuid::parse_str(property_id)
        .context("Некорректный ID объекта")?;

    let Some(user) = get_user_by_chat_id(&state, chat_id).await? else {
        return Ok(());
    };

    operations::remove_from_favorites(
        &state.db,
        user.id,
        property_id,
    )
    .await?;

    bot.send_message(
        chat_id,
        "Объект удалён из избранного.",
    )
    .await?;

    Ok(())
}

async fn show_profile(
    bot: Bot,
    message: Message,
    state: Arc<AppState>,
) -> Result<()> {
    let Some(user) = get_user(&state, &message).await? else {
        bot.send_message(message.chat.id, "Выполните /start.")
            .await?;

        return Ok(());
    };

    let profile = operations::get_profile(&state.db, user.id).await?;

    let text = match profile {
        Some(profile) => format!(
            "👤 <b>Ваш профиль</b>\n\n\
             Имя: {}\n\
             Телефон: {}\n\
             Город: {}\n\
             Район: {}\n\
             Бюджет: {}\n\
             Комнат: {}\n\
             Пожелания: {}",
            escape_html(&user.first_name),
            escape_html(user.phone.as_deref().unwrap_or("не указан")),
            escape_html(profile.city.as_deref().unwrap_or("не указан")),
            escape_html(profile.district.as_deref().unwrap_or("не указан")),
            profile
                .budget
                .map(format_price)
                .map(|value| format!("{value} ₽"))
                .unwrap_or_else(|| "не указан".to_owned()),
            profile
                .rooms
                .map(|value| value.to_string())
                .unwrap_or_else(|| "не указано".to_owned()),
            escape_html(
                profile
                    .additional_requirements
                    .as_deref()
                    .unwrap_or("нет"),
            ),
        ),
        None => "Профиль ещё не создан.".to_owned(),
    };

    bot.send_message(message.chat.id, text)
        .parse_mode(ParseMode::Html)
        .reply_markup(keyboards::main_menu())
        .await?;

    Ok(())
}

async fn show_favorites(
    bot: Bot,
    message: Message,
    state: Arc<AppState>,
) -> Result<()> {
    let Some(user) = get_user(&state, &message).await? else {
        bot.send_message(message.chat.id, "Выполните /start.")
            .await?;

        return Ok(());
    };

    let properties = operations::get_favorites(
        &state.db,
        user.id,
    )
    .await?;

    if properties.is_empty() {
        bot.send_message(
            message.chat.id,
            "⭐ Избранное пока пустое.",
        )
        .reply_markup(keyboards::main_menu())
        .await?;

        return Ok(());
    }

    bot.send_message(
        message.chat.id,
        format!("⭐ Избранное: {} объектов", properties.len()),
    )
    .await?;

    for property in properties {
        let text = format!(
            "🏠 <b>{}</b>\n\n\
             📍 {} — {}\n\
             💰 {} ₽\n\
             🛏 {} комн.\n\
             📐 {:.1} м²\n\n\
             {}",
            escape_html(&property.title),
            escape_html(&property.city),
            escape_html(&property.district),
            format_price(property.price),
            property.rooms,
            property.area,
            escape_html(&property.description),
        );

        if let Some(photo_url) = property.photo_url.as_deref() {
            if let Ok(url) = photo_url.parse() {
                bot.send_photo(
                    message.chat.id,
                    InputFile::url(url),
                )
                .caption(text)
                .parse_mode(ParseMode::Html)
                .reply_markup(keyboards::favorites_keyboard(property.id))
                .await?;

                continue;
            }
        }

        bot.send_message(message.chat.id, text)
            .parse_mode(ParseMode::Html)
            .reply_markup(keyboards::favorites_keyboard(property.id))
            .await?;
    }

    Ok(())
}

async fn create_viewing_request(
    bot: Bot,
    chat_id: ChatId,
    property_id: &str,
    state: Arc<AppState>,
) -> Result<()> {
    let property_id = Uuid::parse_str(property_id)
        .context("Некорректный ID объекта")?;

    let Some(user) = get_user_by_chat_id(&state, chat_id).await? else {
        return Ok(());
    };

    let Some(property) =
        operations::get_property(&state.db, property_id).await?
    else {
        bot.send_message(
            chat_id,
            "Объект больше недоступен.",
        )
        .await?;

        return Ok(());
    };

    let request = operations::create_viewing_request(
        &state.db,
        user.id,
        property.id,
        None,
    )
    .await?;

    let admin_message = format!(
        "📋 <b>Новая заявка на просмотр</b>\n\n\
         🆔 Заявка: <code>{}</code>\n\
         👤 Клиент: {}\n\
         📱 Телефон: {}\n\
         🏠 Объект: {}\n\
         💰 Цена: {} ₽",
        request.id,
        escape_html(&user.first_name),
        escape_html(user.phone.as_deref().unwrap_or("не указан")),
        escape_html(&property.title),
        format_price(property.price),
    );

    bot.send_message(
        ChatId(state.config.admin_chat_id),
        admin_message,
    )
    .parse_mode(ParseMode::Html)
    .await?;

    bot.send_message(
        chat_id,
        "✅ Заявка отправлена!\n\nМенеджер свяжется с вами для согласования просмотра.",
    )
    .reply_markup(keyboards::main_menu())
    .await?;

    Ok(())
}

async fn show_help(
    bot: Bot,
    message: Message,
) -> Result<()> {
    let text = "\
ℹ️ <b>Помощь</b>

\
🏠 <b>Подобрать недвижимость</b> — запустит подбор объекта по вашим параметрам.

\
⭐ <b>Избранное</b> — сохранённые варианты.

\
👤 <b>Профиль</b> — ваши контактные данные и параметры поиска.

\
📅 <b>Записаться</b> — отправляет заявку менеджеру.

\
Команды:
/start — запуск бота
/search — поиск
/profile — профиль
/favorites — избранное
/help — помощь";

    bot.send_message(message.chat.id, text)
        .parse_mode(ParseMode::Html)
        .reply_markup(keyboards::main_menu())
        .await?;

    Ok(())
}

async fn get_user(
    state: &Arc<AppState>,
    message: &Message,
) -> Result<Option<crate::models::User>> {
    let Some(from) = message.from.as_ref() else {
        return Ok(None);
    };

    operations::get_user_by_telegram_id(
        &state.db,
        from.id.0 as i64,
    )
    .await
}

async fn get_user_by_chat_id(
    state: &Arc<AppState>,
    chat_id: ChatId,
) -> Result<Option<crate::models::User>> {
    operations::get_user_by_telegram_id(
        &state.db,
        chat_id.0,
    )
    .await
}

async fn get_or_create_profile(
    state: &Arc<AppState>,
    user_id: Uuid,
) -> Result<crate::models::UserProfile> {
    if let Some(profile) =
        operations::get_profile(&state.db, user_id).await?
    {
        return Ok(profile);
    }

    operations::create_profile(&state.db, user_id).await
}

fn format_price(price: i64) -> String {
    let value = price.to_string();
    let mut result = String::with_capacity(value.len() + value.len() / 3);

    for (index, character) in value.chars().rev().enumerate() {
        if index > 0 && index % 3 == 0 {
            result.push(' ');
        }

        result.push(character);
    }

    result.chars().rev().collect()
}

fn escape_html(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}