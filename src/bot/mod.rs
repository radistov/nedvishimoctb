// src/bot/mod.rs

pub mod handlers;
pub mod keyboards;

use std::sync::Arc;

use anyhow::Result;
use teloxide::{
    dispatching::{
        dialogue::{Dialogue, InMemStorage},
        UpdateHandler,
    },
    dptree,
    prelude::*,
};

use crate::{
    models::{Command, RegistrationState},
    AppState,
};

pub type MyDialogue = Dialogue<RegistrationState, InMemStorage<RegistrationState>>;
pub type HandlerResult = Result<()>;

fn schema() -> UpdateHandler<anyhow::Error> {
    use handlers::*;

    let message_handler = Update::filter_message()
        .enter_dialogue::<Message, InMemStorage<RegistrationState>, RegistrationState>()
        .branch(
            dptree::entry()
                .filter_command::<Command>()
                .endpoint(command_handler),
        )
        .branch(dptree::endpoint(message_handler));

    let callback_handler = Update::filter_callback_query()
        .enter_dialogue::<CallbackQuery, InMemStorage<RegistrationState>, RegistrationState>()
        .endpoint(callback_handler);

    dptree::entry()
        .branch(message_handler)
        .branch(callback_handler)
}

pub async fn run(bot: Bot, state: Arc<AppState>) -> Result<()> {
    let storage = InMemStorage::<RegistrationState>::new();

    Dispatcher::builder(bot, schema())
        .dependencies(dptree::deps![storage, state])
        .enable_ctrlc_handler()
        .default_handler(|update| async move {
            log::debug!("Необработанное обновление: {:?}", update);
        })
        .error_handler(LoggingErrorHandler::with_custom_text(
            "Ошибка Telegram Dispatcher",
        ))
        .build()
        .dispatch()
        .await;

    Ok(())
}