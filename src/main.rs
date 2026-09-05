use std::sync::Arc;

use anyhow::Result;
use tokio::net::TcpListener;
use teloxide::prelude::*;

mod admin;
mod bot;
mod config;
mod db;
mod logger;
mod models;

use config::Config;

#[derive(Clone)]
pub struct AppState {
    pub config: Arc<Config>,
    pub db: sqlx::PgPool,
}

#[tokio::main]
async fn main() -> Result<()> {
    logger::init();

    log::info!("Запуск real_estate_bot...");

    let config = Arc::new(Config::from_env()?);

    log::info!("Подключение к PostgreSQL...");
    let db_pool = db::connect(&config.database_url).await?;

    log::info!("Применение миграций...");
    db::migrate(&db_pool).await?;

    let state = Arc::new(AppState {
        config: config.clone(),
        db: db_pool,
    });

    let bot = Bot::new(config.telegram_token.clone());

    let admin_router = admin::router(state.clone());
    let admin_addr = config.socket_addr()?;

    let admin_server = async move {
        let listener = TcpListener::bind(admin_addr).await?;

        log::info!("Админ API запущен на http://{}", admin_addr);

        axum::serve(listener, admin_router)
            .await
            .map_err(anyhow::Error::from)
    };

    let bot_server = async move {
        log::info!("Telegram-бот запущен");
        bot::run(bot, state).await
    };

    tokio::try_join!(admin_server, bot_server)?;

    Ok(())
}