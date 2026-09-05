// README.md
// Документация проекта.

# Real Estate Bot

Telegram-бот для подбора недвижимости на Rust.

## Стек

- Rust 2024
- Tokio
- Teloxide
- PostgreSQL
- SQLx
- Axum
- Askama
- dotenvy

## Запуск

### 1. Требования

Установи:

- Rust stable
- PostgreSQL
- Telegram-бота через BotFather

### 2. Настрой окружение

Скопируй `.env.example` в `.env`:

```bash
cp .env.example .env

Заполни реальные значения:

TELEGRAM_TOKEN=...
DATABASE_URL=...
ADMIN_CHAT_ID=...
ADMIN_API_TOKEN=...

.env не должен попадать в GitHub.

3. Создай базу данных

Создай PostgreSQL-базу:

CREATE DATABASE real_estate_bot;

Лучше использовать отдельного пользователя PostgreSQL с минимальными правами.

4. Запуск

cargo run

При запуске бот автоматически:

1. подключается к PostgreSQL;


2. применяет миграции;


3. запускает Telegram-бота;


4. запускает HTTP API админки.



Проверка

После запуска в логах должно появиться:

Подключение к PostgreSQL установлено
Миграции PostgreSQL успешно применены
Telegram-бот запущен
Админ API запущен

Безопасность

Никогда не публикуй:

.env;

Telegram Bot Token;

пароль PostgreSQL;

ADMIN_API_TOKEN;

приватные ключи;

реальные пользовательские данные.


Перед первым git push проверь:

git status

В списке не должно быть .env.

Если секрет уже попал в GitHub, его нужно считать скомпрометированным и заменить.

Структура

real_estate_bot/
├── src/
│   ├── admin/
│   ├── bot/
│   ├── db/
│   ├── config.rs
│   ├── logger.rs
│   ├── models.rs
│   └── main.rs
├── migrations/
│   └── 0001_init.sql
├── .env.example
├── .gitignore
├── Cargo.toml
└── README.md

Статус

Проект находится в разработке.

Перед использованием на публичном сервере необходимо дополнительно настроить:

авторизацию админ API;

HTTPS;

ограничение доступа к PostgreSQL;

безопасное хранение секретов;

резервное копирование базы;

rate limiting;

обработку ошибок и аудит действий администратора.