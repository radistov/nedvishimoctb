use std::sync::Once;

use chrono::Local;
use env_logger::Env;
use log::Level;

static INIT: Once = Once::new();

pub fn init() {
    INIT.call_once(|| {
        env_logger::Builder::from_env(
            Env::default().default_filter_or("info"),
        )
        .format(|buf, record| {
            let timestamp = Local::now().format("%Y-%m-%d %H:%M:%S");

            let level = match record.level() {
                Level::Error => "ERROR",
                Level::Warn => "WARN",
                Level::Info => "INFO",
                Level::Debug => "DEBUG",
                Level::Trace => "TRACE",
            };

            writeln!(
                buf,
                "[{}] [{}] {}",
                timestamp,
                level,
                record.args()
            )
        })
        .init();

        log::info!("Логирование инициализировано");
    });
}