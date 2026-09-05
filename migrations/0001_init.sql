-- migrations/0001_init.sql

-- ============================================================================
-- Первая миграция проекта
--
-- Создает все основные таблицы системы:
--
-- • users
-- • user_profiles
-- • properties
-- • favorites
-- • viewing_requests
--
-- Миграция может быть выполнена повторно без ошибок.
-- ============================================================================

CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

--------------------------------------------------------------------------------
-- Пользователи
--------------------------------------------------------------------------------

CREATE TABLE IF NOT EXISTS users
(
    id UUID PRIMARY KEY,

    telegram_id BIGINT NOT NULL UNIQUE,

    username TEXT,

    first_name TEXT NOT NULL,

    last_name TEXT,

    phone TEXT,

    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_users_telegram
ON users(telegram_id);

--------------------------------------------------------------------------------
-- Профиль пользователя
--------------------------------------------------------------------------------

CREATE TABLE IF NOT EXISTS user_profiles
(
    id UUID PRIMARY KEY,

    user_id UUID NOT NULL UNIQUE,

    city TEXT,

    district TEXT,

    budget BIGINT,

    rooms INTEGER,

    additional_requirements TEXT,

    CONSTRAINT fk_profile_user
        FOREIGN KEY(user_id)
        REFERENCES users(id)
        ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_profile_city
ON user_profiles(city);

CREATE INDEX IF NOT EXISTS idx_profile_district
ON user_profiles(district);

--------------------------------------------------------------------------------
-- Объекты недвижимости
--------------------------------------------------------------------------------

CREATE TABLE IF NOT EXISTS properties
(
    id UUID PRIMARY KEY,

    title TEXT NOT NULL,

    description TEXT NOT NULL,

    city TEXT NOT NULL,

    district TEXT NOT NULL,

    price BIGINT NOT NULL,

    rooms INTEGER NOT NULL,

    area REAL NOT NULL,

    photo_url TEXT,

    is_active BOOLEAN NOT NULL DEFAULT TRUE,

    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_property_city
ON properties(city);

CREATE INDEX IF NOT EXISTS idx_property_district
ON properties(district);

CREATE INDEX IF NOT EXISTS idx_property_price
ON properties(price);

CREATE INDEX IF NOT EXISTS idx_property_rooms
ON properties(rooms);

CREATE INDEX IF NOT EXISTS idx_property_active
ON properties(is_active);

--------------------------------------------------------------------------------
-- Избранное
--------------------------------------------------------------------------------

CREATE TABLE IF NOT EXISTS favorites
(
    id UUID PRIMARY KEY,

    user_id UUID NOT NULL,

    property_id UUID NOT NULL,

    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    CONSTRAINT fk_favorite_user
        FOREIGN KEY(user_id)
        REFERENCES users(id)
        ON DELETE CASCADE,

    CONSTRAINT fk_favorite_property
        FOREIGN KEY(property_id)
        REFERENCES properties(id)
        ON DELETE CASCADE,

    CONSTRAINT uq_favorite
        UNIQUE(user_id, property_id)
);

CREATE INDEX IF NOT EXISTS idx_favorites_user
ON favorites(user_id);

CREATE INDEX IF NOT EXISTS idx_favorites_property
ON favorites(property_id);

--------------------------------------------------------------------------------
-- Заявки на просмотр
--------------------------------------------------------------------------------

CREATE TABLE IF NOT EXISTS viewing_requests
(
    id UUID PRIMARY KEY,

    user_id UUID NOT NULL,

    property_id UUID NOT NULL,

    status TEXT NOT NULL DEFAULT 'New',

    comment TEXT,

    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    CONSTRAINT fk_request_user
        FOREIGN KEY(user_id)
        REFERENCES users(id)
        ON DELETE CASCADE,

    CONSTRAINT fk_request_property
        FOREIGN KEY(property_id)
        REFERENCES properties(id)
        ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_requests_user
ON viewing_requests(user_id);

CREATE INDEX IF NOT EXISTS idx_requests_property
ON viewing_requests(property_id);

CREATE INDEX IF NOT EXISTS idx_requests_status
ON viewing_requests(status);

--------------------------------------------------------------------------------
-- Тестовые объекты недвижимости
--
-- На первом запуске бот уже сможет выполнять поиск.
--------------------------------------------------------------------------------

INSERT INTO properties
(
    id,
    title,
    description,
    city,
    district,
    price,
    rooms,
    area,
    photo_url
)
VALUES
(
    uuid_generate_v4(),
    '1-комнатная квартира',
    'Современная квартира с ремонтом.',
    'Москва',
    'Центральный',
    4800000,
    1,
    38.5,
    'https://picsum.photos/800/600'
)
ON CONFLICT DO NOTHING;

INSERT INTO properties
(
    id,
    title,
    description,
    city,
    district,
    price,
    rooms,
    area,
    photo_url
)
VALUES
(
    uuid_generate_v4(),
    '2-комнатная квартира',
    'Дом бизнес-класса.',
    'Москва',
    'Центральный',
    7300000,
    2,
    59.8,
    'https://picsum.photos/800/601'
)
ON CONFLICT DO NOTHING;

INSERT INTO properties
(
    id,
    title,
    description,
    city,
    district,
    price,
    rooms,
    area,
    photo_url
)
VALUES
(
    uuid_generate_v4(),
    '3-комнатная квартира',
    'Рядом метро и парк.',
    'Москва',
    'Северный',
    9800000,
    3,
    82.3,
    'https://picsum.photos/800/602'
)
ON CONFLICT DO NOTHING;

INSERT INTO properties
(
    id,
    title,
    description,
    city,
    district,
    price,
    rooms,
    area,
    photo_url
)
VALUES
(
    uuid_generate_v4(),
    'Студия',
    'Отличный вариант для аренды.',
    'Санкт-Петербург',
    'Центральный',
    3600000,
    1,
    28.0,
    'https://picsum.photos/800/603'
)
ON CONFLICT DO NOTHING;

INSERT INTO properties
(
    id,
    title,
    description,
    city,
    district,
    price,
    rooms,
    area,
    photo_url
)
VALUES
(
    uuid_generate_v4(),
    '4-комнатная квартира',
    'Панорамные окна, подземный паркинг.',
    'Казань',
    'Западный',
    11900000,
    4,
    116.5,
    'https://picsum.photos/800/604'
)
ON CONFLICT DO NOTHING;