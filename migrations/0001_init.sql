-- migrations/0001_init.sql

CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

CREATE TABLE IF NOT EXISTS users (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    telegram_id BIGINT NOT NULL UNIQUE,
    username TEXT,
    first_name TEXT NOT NULL,
    last_name TEXT,
    phone TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS user_profiles (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    user_id UUID NOT NULL UNIQUE REFERENCES users(id) ON DELETE CASCADE,
    city TEXT,
    district TEXT,
    budget BIGINT,
    rooms INTEGER,
    additional_requirements TEXT
);

CREATE TABLE IF NOT EXISTS properties (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    title TEXT NOT NULL,
    description TEXT NOT NULL,
    city TEXT NOT NULL,
    district TEXT NOT NULL,
    price BIGINT NOT NULL CHECK (price > 0),
    rooms INTEGER NOT NULL CHECK (rooms > 0),
    area REAL NOT NULL CHECK (area > 0),
    photo_url TEXT,
    is_active BOOLEAN NOT NULL DEFAULT TRUE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS favorites (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    property_id UUID NOT NULL REFERENCES properties(id) ON DELETE CASCADE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE (user_id, property_id)
);

CREATE TABLE IF NOT EXISTS viewing_requests (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    property_id UUID NOT NULL REFERENCES properties(id) ON DELETE CASCADE,
    status TEXT NOT NULL DEFAULT 'New',
    comment TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_users_telegram_id
    ON users(telegram_id);

CREATE INDEX IF NOT EXISTS idx_properties_search
    ON properties(city, district, price, rooms)
    WHERE is_active = TRUE;

CREATE INDEX IF NOT EXISTS idx_properties_active
    ON properties(is_active);

CREATE INDEX IF NOT EXISTS idx_favorites_user_id
    ON favorites(user_id);

CREATE INDEX IF NOT EXISTS idx_viewing_requests_user_id
    ON viewing_requests(user_id);

CREATE INDEX IF NOT EXISTS idx_viewing_requests_status
    ON viewing_requests(status);

INSERT INTO properties (
    id,
    title,
    description,
    city,
    district,
    price,
    rooms,
    area,
    photo_url,
    is_active
)
VALUES
(
    '10000000-0000-0000-0000-000000000001',
    'Светлая квартира рядом с центром',
    'Уютная квартира с хорошим ремонтом и развитой инфраструктурой рядом.',
    'Москва',
    'Центральный',
    12000000,
    2,
    54.5,
    NULL,
    TRUE
),
(
    '10000000-0000-0000-0000-000000000002',
    'Двухкомнатная квартира у метро',
    'Современная квартира недалеко от метро, магазинов и общественного транспорта.',
    'Москва',
    'Северный',
    9800000,
    2,
    49.0,
    NULL,
    TRUE
),
(
    '10000000-0000-0000-0000-000000000003',
    'Уютная квартира в новом доме',
    'Квартира в новом жилом комплексе с закрытой территорией.',
    'Санкт-Петербург',
    'Центральный',
    8500000,
    2,
    52.0,
    NULL,
    TRUE
),
(
    '10000000-0000-0000-0000-000000000004',
    'Просторная квартира для семьи',
    'Трёхкомнатная квартира с большой кухней и удобной планировкой.',
    'Казань',
    'Советский',
    7600000,
    3,
    78.5,
    NULL,
    TRUE
),
(
    '10000000-0000-0000-0000-000000000005',
    'Однокомнатная квартира',
    'Компактная квартира с современным ремонтом.',
    'Екатеринбург',
    'Центральный',
    5100000,
    1,
    38.0,
    NULL,
    TRUE
)
ON CONFLICT (id) DO NOTHING;