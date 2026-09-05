// src/db/mod.rs

/*!
    Модуль работы с базой данных.

    Назначение:

    • создание пула подключений PostgreSQL;
    • применение миграций;
    • экспорт операций из db::operations;
    • единая точка входа для работы с БД.

    В проекте используется SQLx.

    Структура каталога:

    db/
    ├── mod.rs
    └── operations.rs

    Позже рядом появится каталог:

    migrations/
        0001_init.sql
        ...

    После запуска приложения все миграции автоматически применяются.

    Важно:
    Для корректной работы sqlx::migrate! каталог migrations
    должен находиться в корне проекта.
*/

pub mod operations;

use anyhow::{Context, Result};
use sqlx::{
    postgres::PgPoolOptions,
    PgPool,
};
use std::time::Duration;

/// Создание пула соединений с PostgreSQL.
///
/// Используется единственный пул на всё приложение.
///
/// # Аргументы
///
/// * `database_url` - строка подключения к PostgreSQL.
///
/// # Возвращает
///
/// Готовый PgPool.
pub async fn connect(database_url: &str) -> Result<PgPool> {
    log::info!("Подключение к PostgreSQL...");

    let pool = PgPoolOptions::new()
        // Максимальное количество соединений.
        .max_connections(20)

        // Минимальное количество открытых соединений.
        .min_connections(5)

        // Максимальное время ожидания свободного соединения.
        .acquire_timeout(Duration::from_secs(10))

        // Максимальное время жизни соединения.
        .max_lifetime(Duration::from_secs(60 * 30))

        // Если соединение простаивает слишком долго —
        // оно будет пересоздано.
        .idle_timeout(Duration::from_secs(60 * 10))

        // Проверять соединение перед выдачей.
        .test_before_acquire(true)

        .connect(database_url)
        .await
        .context("Не удалось подключиться к PostgreSQL")?;

    // Проверяем соединение простым запросом.
    sqlx::query("SELECT 1")
        .execute(&pool)
        .await
        .context("Проверка подключения к PostgreSQL не удалась")?;

    log::info!("Соединение с PostgreSQL успешно установлено.");

    Ok(pool)
}

/// Применение всех миграций.
///
/// Все SQL-файлы из каталога migrations будут выполнены автоматически.
///
/// Повторный запуск безопасен — уже применённые миграции
/// пропускаются.
pub async fn migrate(pool: &PgPool) -> Result<()> {
    log::info!("Проверка миграций базы данных...");

    sqlx::migrate!("./migrations")
        .run(pool)
        .await
        .context("Ошибка выполнения миграций")?;

    log::info!("Миграции успешно применены.");

    Ok(())
}