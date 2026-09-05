pub mod operations;

use std::time::Duration;

use anyhow::{Context, Result};
use sqlx::{
    postgres::PgPoolOptions,
    PgPool,
};

pub async fn connect(database_url: &str) -> Result<PgPool> {
    let pool = PgPoolOptions::new()
        .min_connections(2)
        .max_connections(20)
        .acquire_timeout(Duration::from_secs(10))
        .idle_timeout(Duration::from_secs(300))
        .max_lifetime(Duration::from_secs(1800))
        .test_before_acquire(true)
        .connect(database_url)
        .await
        .context("Не удалось подключиться к PostgreSQL")?;

    sqlx::query("SELECT 1")
        .execute(&pool)
        .await
        .context("PostgreSQL не отвечает на проверочный запрос")?;

    log::info!("Подключение к PostgreSQL установлено");

    Ok(pool)
}

pub async fn migrate(pool: &PgPool) -> Result<()> {
    sqlx::migrate!("./migrations")
        .run(pool)
        .await
        .context("Не удалось применить миграции PostgreSQL")?;

    log::info!("Миграции PostgreSQL успешно применены");

    Ok(())
}