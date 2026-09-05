// src/logger.rs

/*!
    Модуль инициализации логирования.

    Назначение:

    • Настройка глобального логгера приложения.
    • Поддержка переменной окружения RUST_LOG.
    • Красивый вывод времени, уровня сообщения и текста.
    • Единая точка настройки логирования для всего проекта.

    Пример использования:

        logger::init();

        log::info!("Бот успешно запущен.");
        log::warn!("Соединение с БД потеряно.");
        log::error!("Ошибка при сохранении профиля.");

    Пример переменной окружения:

        RUST_LOG=info

    Возможные уровни:

        error
        warn
        info
        debug
        trace

    Например:

        RUST_LOG=debug

    или

        RUST_LOG=real_estate_bot=trace,teloxide=info
*/

use chrono::Local;
use env_logger::{Builder, Env};
use log::Level;
use std::io::Write;
use std::sync::Once;

/// Гарантирует однократную инициализацию логгера.
///
/// Даже если logger::init() будет вызван несколько раз,
/// реальная инициализация произойдёт только один раз.
static INIT: Once = Once::new();

/// Инициализация логирования.
///
/// Вызывается единожды из `main.rs`.
pub fn init() {
    INIT.call_once(|| {
        Builder::from_env(Env::default().default_filter_or("info"))
            .format(|buf, record| {
                let timestamp = Local::now().format("%Y-%m-%d %H:%M:%S");

                // Цвета уровней логирования.
                let level = match record.level() {
                    Level::Error => "\x1b[31mERROR\x1b[0m",
                    Level::Warn => "\x1b[33mWARN \x1b[0m",
                    Level::Info => "\x1b[32mINFO \x1b[0m",
                    Level::Debug => "\x1b[34mDEBUG\x1b[0m",
                    Level::Trace => "\x1b[35mTRACE\x1b[0m",
                };

                writeln!(
                    buf,
                    "[{}] [{}] [{}] {}",
                    timestamp,
                    level,
                    record.target(),
                    record.args()
                )
            })
            .init();

        log::info!("==========================================");
        log::info!(" Логирование успешно инициализировано");
        log::info!("==========================================");
    });
}