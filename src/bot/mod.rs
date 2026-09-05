// src/bot/mod.rs

/*!
    Модуль запуска Telegram-бота.

    Данный файл является точкой входа для всей логики Telegram.

    Его задачи:

    • создание Dispatcher;
    • регистрация обработчиков команд;
    • подключение FSM (диалогов);
    • подключение callback-кнопок;
    • обработка ошибок;
    • запуск polling.

    Вся бизнес-логика находится в handlers.rs.

    Структура каталога:

    bot/
    ├── mod.rs
    ├── handlers.rs
    └── keyboards.rs
*/

pub mod handlers;
pub mod keyboards;

use anyhow::Result;
use teloxide::{
    dispatching::{
        dialogue::InMemStorage,
        UpdateFilterExt,
    },
    dptree,
    prelude::*,
};

use crate::{
    models::{Command, RegistrationState},
    AppState,
};

/// Тип хранилища состояний FSM.
///
/// На первом этапе используется память процесса.
/// В дальнейшем без изменений кода можно перейти
/// на RedisStorage или PostgresStorage.
pub type Dialogue = teloxide::dispatching::dialogue::Dialogue<
    RegistrationState,
    InMemStorage<RegistrationState>,
>;

/// Тип результата обработчиков.
///
/// Все обработчики будут возвращать именно HandlerResult.
pub type HandlerResult = Result<()>;

/// Запуск Telegram-бота.
///
/// Вызывается один раз из main.rs.
pub async fn run(bot: Bot, state: AppState) -> Result<()> {
    log::info!("Инициализация Dispatcher...");

    // Хранилище состояний FSM.
    let storage = InMemStorage::<RegistrationState>::new();

    /*
        Схема обработки обновлений.

        Update
            │
            ├── сообщения
            │      │
            │      ├── команды (/start, /profile ...)
            │      └── обычный текст
            │
            └── callback-кнопки
    */

    let command_handler = Update::filter_message()
        .filter_command::<Command>()
        .endpoint(handlers::command_handler);

    let message_handler = Update::filter_message()
        .endpoint(handlers::message_handler);

    let callback_handler = Update::filter_callback_query()
        .endpoint(handlers::callback_handler);

    let handler = dptree::entry()
        .branch(command_handler)
        .branch(message_handler)
        .branch(callback_handler);

    log::info!("Dispatcher успешно создан.");

    Dispatcher::builder(bot, handler)
        // Передаём общее состояние приложения
        // во все обработчики.
        .dependencies(dptree::deps![
            storage,
            state,
        ])
        // Если обработчик вернул ошибку —
        // выводим её в лог.
        .default_handler(|update| async move {
            log::debug!("Необработанное обновление: {:?}", update);
        })
        .error_handler(LoggingErrorHandler::with_custom_text(
            "Ошибка Dispatcher",
        ))
        .enable_ctrlc_handler()
        .build()
        .dispatch()
        .await;

    log::info!("Dispatcher остановлен.");

    Ok(())
}