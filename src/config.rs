// src/config.rs

/*!
    Конфигурация приложения.

    Данный модуль отвечает за:

    • загрузку переменных окружения из файла .env;
    • валидацию обязательных параметров;
    • хранение конфигурации в единой структуре Config.

    Благодаря этому в остальных частях приложения не происходит
    прямого обращения к std::env.

    Пример файла .env:

    ----------------------------------------------------
    TELEGRAM_TOKEN=123456:ABC...
    DATABASE_URL=postgres://user:password@localhost/real_estate_bot

    ADMIN_CHAT_ID=123456789
    HOST=0.0.0.0
    PORT=8080

    RUST_LOG=info
    ----------------------------------------------------
*/

use anyhow::{Context, Result};
use dotenvy::dotenv;
use std::env;

/// Основная конфигурация приложения.
///
/// После загрузки экземпляр этой структуры передаётся
/// во всё приложение через Arc<Config>.
#[derive(Debug, Clone)]
pub struct Config {
    /// Токен Telegram-бота.
    pub telegram_token: String,

    /// Строка подключения к PostgreSQL.
    pub database_url: String,

    /// Telegram ID менеджера/администратора.
    ///
    /// На него будут отправляться уведомления
    /// о новых заявках.
    pub admin_chat_id: i64,

    /// Адрес веб-сервера админ-панели.
    pub host: String,

    /// Порт веб-сервера.
    pub port: u16,
}

impl Config {
    /// Загружает конфигурацию из окружения.
    ///
    /// Сначала пытается загрузить файл `.env`,
    /// затем читает переменные окружения.
    pub fn from_env() -> Result<Self> {
        // Игнорируем ошибку, если .env отсутствует.
        let _ = dotenv();

        Ok(Self {
            telegram_token: get_required("TELEGRAM_TOKEN")?,

            database_url: get_required("DATABASE_URL")?,

            admin_chat_id: get_required("ADMIN_CHAT_ID")?
                .parse()
                .context("ADMIN_CHAT_ID должен быть числом")?,

            // Значения по умолчанию позволяют запускать
            // проект без лишней настройки.
            host: get_optional("HOST", "0.0.0.0"),

            port: get_optional("PORT", "8080")
                .parse()
                .context("PORT должен быть числом")?,
        })
    }

    /// Возвращает адрес, который удобно использовать
    /// при запуске веб-сервера.
    ///
    /// Например:
    ///
    /// 0.0.0.0:8080
    pub fn socket_addr(&self) -> String {
        format!("{}:{}", self.host, self.port)
    }
}

/// Читает обязательную переменную окружения.
///
/// Если переменная отсутствует или пустая,
/// возвращается понятная ошибка.
fn get_required(name: &str) -> Result<String> {
    let value = env::var(name)
        .with_context(|| format!("Не найдена обязательная переменная окружения: {name}"))?;

    if value.trim().is_empty() {
        anyhow::bail!("Переменная окружения {name} не может быть пустой");
    }

    Ok(value)
}

/// Читает необязательную переменную окружения.
///
/// Если переменная отсутствует — используется
/// значение по умолчанию.
fn get_optional(name: &str, default: &str) -> String {
    env::var(name).unwrap_or_else(|_| default.to_string())
}