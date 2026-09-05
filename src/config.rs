use std::{env, net::SocketAddr};

use anyhow::{Context, Result};

#[derive(Debug, Clone)]
pub struct Config {
    pub telegram_token: String,
    pub database_url: String,
    pub admin_chat_id: i64,
    pub host: String,
    pub port: u16,
}

impl Config {
    pub fn from_env() -> Result<Self> {
        dotenvy::dotenv().ok();

        let telegram_token = required("TELEGRAM_TOKEN")?;
        let database_url = required("DATABASE_URL")?;

        let admin_chat_id = required("ADMIN_CHAT_ID")?
            .parse::<i64>()
            .context("ADMIN_CHAT_ID должен быть целым числом")?;

        let host = optional("HOST", "0.0.0.0");

        let port = optional("PORT", "8080")
            .parse::<u16>()
            .context("PORT должен быть числом от 0 до 65535")?;

        Ok(Self {
            telegram_token,
            database_url,
            admin_chat_id,
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
    env::var(name)
        .with_context(|| format!("Переменная окружения {name} не задана"))
}

fn optional(name: &str, default: &str) -> String {
    env::var(name).unwrap_or_else(|_| default.to_owned())
}