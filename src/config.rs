// src/config.rs
// Конфигурация приложения. Секреты читаются только из переменных окружения.

use std::{env, net::SocketAddr};

use anyhow::{Context, Result};

#[derive(Debug, Clone)]
pub struct Config {
    pub telegram_token: String,
    pub database_url: String,
    pub admin_chat_id: i64,
    pub admin_api_token: String,
    pub host: String,
    pub port: u16,
}

impl Config {
    pub fn from_env() -> Result<Self> {
        dotenvy::dotenv().ok();

        let telegram_token = required("TELEGRAM_TOKEN")?;
        let database_url = required("DATABASE_URL")?;
        let admin_api_token = required("ADMIN_API_TOKEN")?;

        if admin_api_token.len() < 32 {
            anyhow::bail!("ADMIN_API_TOKEN должен содержать минимум 32 символа");
        }

        let admin_chat_id = required("ADMIN_CHAT_ID")?
            .parse::<i64>()
            .context("ADMIN_CHAT_ID должен быть целым числом")?;

        let host = optional("HOST", "127.0.0.1");

        let port = optional("PORT", "8080")
            .parse::<u16>()
            .context("PORT должен быть числом от 0 до 65535")?;

        Ok(Self {
            telegram_token,
            database_url,
            admin_chat_id,
            admin_api_token,
            host,
            port,
        })
    }

    pub fn socket_addr(&self) -> Result<SocketAddr> {
        format!("{}:{}", self.host, self.port)
            .parse()
            .context("Некорректный HOST или PORT")
    }
}

fn required(name: &str) -> Result<String> {
    let value = env::var(name)
        .with_context(|| format!("Переменная окружения {name} не задана"))?;

    if value.trim().is_empty() {
        anyhow::bail!("Переменная окружения {name} пустая");
    }

    Ok(value)
}

fn optional(name: &str, default: &str) -> String {
    env::var(name).unwrap_or_else(|_| default.to_owned())
}