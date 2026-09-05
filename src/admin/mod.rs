// src/admin/mod.rs

use std::sync::Arc;

use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{Html, IntoResponse},
    routing::{delete, get, post, put},
    Json, Router,
};
use chrono::Utc;
use serde::Deserialize;
use uuid::Uuid;

use crate::{
    db::operations,
    models::{Property, RequestStatus},
    AppState,
};

/// Данные для создания или изменения объекта недвижимости.
#[derive(Debug, Deserialize)]
pub struct PropertyRequest {
    pub title: String,
    pub description: String,
    pub city: String,
    pub district: String,
    pub price: i64,
    pub rooms: i32,
    pub area: f32,
    pub photo_url: Option<String>,
    pub is_active: Option<bool>,
}

/// Проверяет базовые параметры объекта перед записью в БД.
fn validate_property(data: &PropertyRequest) -> Result<(), String> {
    if data.title.trim().is_empty() {
        return Err("Название объекта не может быть пустым".into());
    }

    if data.description.trim().is_empty() {
        return Err("Описание объекта не может быть пустым".into());
    }

    if data.city.trim().is_empty() {
        return Err("Город не может быть пустым".into());
    }

    if data.district.trim().is_empty() {
        return Err("Район не может быть пустым".into());
    }

    if data.price <= 0 {
        return Err("Цена должна быть больше нуля".into());
    }

    if !(1..=20).contains(&data.rooms) {
        return Err("Количество комнат должно быть от 1 до 20".into());
    }

    if data.area <= 0.0 {
        return Err("Площадь должна быть больше нуля".into());
    }

    Ok(())
}

/// Создает объект недвижимости.
async fn create_property(
    State(state): State<Arc<AppState>>,
    Json(data): Json<PropertyRequest>,
) -> impl IntoResponse {
    if let Err(error) = validate_property(&data) {
        return (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({
                "error": error
            })),
        );
    }

    let property = Property {
        id: Uuid::new_v4(),
        title: data.title.trim().to_owned(),
        description: data.description.trim().to_owned(),
        city: data.city.trim().to_owned(),
        district: data.district.trim().to_owned(),
        price: data.price,
        rooms: data.rooms,
        area: data.area,
        photo_url: data.photo_url,
        is_active: data.is_active.unwrap_or(true),
        created_at: Utc::now(),
    };

    let result = sqlx::query(
        r#"
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
            photo_url,
            is_active,
            created_at
        )
        VALUES
        (
            $1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11
        )
        "#,
    )
    .bind(property.id)
    .bind(&property.title)
    .bind(&property.description)
    .bind(&property.city)
    .bind(&property.district)
    .bind(property.price)
    .bind(property.rooms)
    .bind(property.area)
    .bind(&property.photo_url)
    .bind(property.is_active)
    .bind(property.created_at)
    .execute(&state.db)
    .await;

    match result {
        Ok(_) => (
            StatusCode::CREATED,
            Json(serde_json::json!({
                "id": property.id,
                "message": "Объект создан"
            })),
        ),

        Err(error) => {
            log::error!("Ошибка создания объекта: {error}");

            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({
                    "error": "Не удалось создать объект"
                })),
            )
        }
    }
}

/// Изменяет существующий объект недвижимости.
async fn update_property(
    State(state): State<Arc<AppState>>,
    Path(property_id): Path<Uuid>,
    Json(data): Json<PropertyRequest>,
) -> impl IntoResponse {
    if let Err(error) = validate_property(&data) {
        return (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({
                "error": error
            })),
        );
    }

    let result = sqlx::query(
        r#"
        UPDATE properties
        SET
            title = $2,
            description = $3,
            city = $4,
            district = $5,
            price = $6,
            rooms = $7,
            area = $8,
            photo_url = $9,
            is_active = $10
        WHERE id = $1
        "#,
    )
    .bind(property_id)
    .bind(data.title.trim())
    .bind(data.description.trim())
    .bind(data.city.trim())
    .bind(data.district.trim())
    .bind(data.price)
    .bind(data.rooms)
    .bind(data.area)
    .bind(&data.photo_url)
    .bind(data.is_active.unwrap_or(true))
    .execute(&state.db)
    .await;

    match result {
        Ok(result) if result.rows_affected() == 0 => (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({
                "error": "Объект не найден"
            })),
        ),

        Ok(_) => (
            StatusCode::OK,
            Json(serde_json::json!({
                "message": "Объект обновлен"
            })),
        ),

        Err(error) => {
            log::error!("Ошибка изменения объекта {property_id}: {error}");

            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({
                    "error": "Не удалось изменить объект"
                })),
            )
        }
    }
}

/// Удаляет объект недвижимости.
async fn delete_property(
    State(state): State<Arc<AppState>>,
    Path(property_id): Path<Uuid>,
) -> impl IntoResponse {
    let result = sqlx::query(
        r#"
        DELETE FROM properties
        WHERE id = $1
        "#,
    )
    .bind(property_id)
    .execute(&state.db)
    .await;

    match result {
        Ok(result) if result.rows_affected() == 0 => (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({
                "error": "Объект не найден"
            })),
        ),

        Ok(_) => (
            StatusCode::OK,
            Json(serde_json::json!({
                "message": "Объект удален"
            })),
        ),

        Err(error) => {
            log::error!("Ошибка удаления объекта {property_id}: {error}");

            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({
                    "error": "Не удалось удалить объект"
                })),
            )
        }
    }
}

/// Возвращает список объектов недвижимости.
async fn list_properties(
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    let result = sqlx::query_as::<_, Property>(
        r#"
        SELECT *
        FROM properties
        ORDER BY created_at DESC
        "#,
    )
    .fetch_all(&state.db)
    .await;

    match result {
        Ok(properties) => (
            StatusCode::OK,
            Json(properties),
        ),

        Err(error) => {
            log::error!("Ошибка получения объектов: {error}");

            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(Vec::<Property>::new()),
            )
        }
    }
}

/// Возвращает список заявок менеджеру.
async fn list_requests(
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    match operations::get_requests(&state.db).await {
        Ok(requests) => (
            StatusCode::OK,
            Json(requests),
        ),

        Err(error) => {
            log::error!("Ошибка получения заявок: {error}");

            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(Vec::new()),
            )
        }
    }
}

/// Данные для изменения статуса заявки.
#[derive(Debug, Deserialize)]
struct UpdateRequestStatus {
    status: RequestStatus,
}

/// Изменяет статус заявки.
async fn update_request(
    State(state): State<Arc<AppState>>,
    Path(request_id): Path<Uuid>,
    Json(data): Json<UpdateRequestStatus>,
) -> impl IntoResponse {
    match operations::update_request_status(
        &state.db,
        request_id,
        data.status,
    )
    .await
    {
        Ok(_) => (
            StatusCode::OK,
            Json(serde_json::json!({
                "message": "Статус заявки обновлен"
            })),
        ),

        Err(error) => {
            log::error!(
                "Ошибка изменения статуса заявки {request_id}: {error}"
            );

            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({
                    "error": "Не удалось обновить статус заявки"
                })),
            )
        }
    }
}

/// Простая HTML-страница админ-панели.
///
/// Позже она будет заменена полноценными HTML-шаблонами.
async fn index() -> Html<&'static str> {
    Html(
        r#"<!DOCTYPE html>
<html lang="ru">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Real Estate Bot — Admin</title>
    <style>
        body {
            font-family: Arial, sans-serif;
            max-width: 1000px;
            margin: 40px auto;
            padding: 0 20px;
        }

        h1 {
            margin-bottom: 30px;
        }

        .card {
            padding: 20px;
            margin-bottom: 15px;
            border: 1px solid #ddd;
            border-radius: 8px;
        }

        code {
            background: #f4f4f4;
            padding: 3px 6px;
            border-radius: 4px;
        }
    </style>
</head>
<body>
    <h1>🏠 Управление недвижимостью</h1>

    <div class="card">
        <h2>Объекты</h2>
        <p>
            Получить список:
            <code>GET /api/properties</code>
        </p>
        <p>
            Создать:
            <code>POST /api/properties</code>
        </p>
        <p>
            Изменить:
            <code>PUT /api/properties/{id}</code>
        </p>
        <p>
            Удалить:
            <code>DELETE /api/properties/{id}</code>
        </p>
    </div>

    <div class="card">
        <h2>Заявки</h2>
        <p>
            Список заявок:
            <code>GET /api/requests</code>
        </p>
        <p>
            Изменение статуса:
            <code>PUT /api/requests/{id}</code>
        </p>
    </div>
</body>
</html>"#,
    )
}

/// Создает Router административной части.
pub fn router(state: Arc<AppState>) -> Router {
    Router::new()
        .route("/", get(index))
        .route(
            "/api/properties",
            get(list_properties).post(create_property),
        )
        .route(
            "/api/properties/{property_id}",
            put(update_property).delete(delete_property),
        )
        .route(
            "/api/requests",
            get(list_requests),
        )
        .route(
            "/api/requests/{request_id}",
            put(update_request),
        )
        .with_state(state)
}