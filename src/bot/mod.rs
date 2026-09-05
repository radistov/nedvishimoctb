// src/bot/mod.rs

pub mod handlers;
pub mod keyboards;

use anyhow::Result;
use teloxide::{
    dispatching::{
        dialogue::{Dialogue, InMemStorage},
        UpdateHandler,
        UpdateFilterExt,
    },
    dptree,
    prelude::*,
};

use crate::{
    models::{Command, RegistrationState},
    AppState,
};

/// Хранилище состояний пользователя.
///
/// На первом этапе используется память процесса.
/// Позже его можно заменить на RedisStorage или PostgresStorage
/// без изменения логики обработчиков.
pub type MyDialogue = Dialogue<
    RegistrationState,
    InMemStorage<RegistrationState>,
>;

/// Тип результата любого обработчика.
pub type HandlerResult = Result<()>;

/// Создание дерева обработки обновлений.
///
/// Все маршруты Telegram находятся здесь.
/// Благодаря этому при росте проекта будет легко добавлять новые
/// ветки (например WebApp, платежи, геолокацию и т.д.).
fn schema() -> UpdateHandler<anyhow::Error> {
    use handlers::*;

    let message_handler = Update::filter_message()
        .enter_dialogue::<Message, InMemStorage<RegistrationState>, RegistrationState>()
        .branch(
            dptree::entry()
                .filter_command::<Command>()
                .endpoint(command_handler),
        )
        .branch(
            dptree::endpoint(message_handler),
        );

    let callback_handler = Update::filter_callback_query()
        .enter_dialogue::<CallbackQuery, InMemStorage<RegistrationState>, RegistrationState>()
        .endpoint(callback_handler);

    dptree::entry()
        .branch(message_handler)
        .branch(callback_handler)
}

/// Запуск Telegram-бота.
pub async fn run(bot: Bot, state: AppState) -> Result<()> {
    log::info!("Создание Dispatcher...");

    let storage = InMemStorage::<RegistrationState>::new();

    Dispatcher::builder(bot, schema())
        .dependencies(dptree::deps![
            storage,
            state,
        ])
        .enable_ctrlc_handler()
        .default_handler(|upd| async move {
            log::debug!("Необработанное обновление: {:?}", upd);
        })
        .error_handler(
            LoggingErrorHandler::with_custom_text(
                "Ошибка Dispatcher",
            ),
        )
        .build()
        .dispatch()
        .await;

    Ok(())
}