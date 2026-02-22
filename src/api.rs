use axum::{
    Router,
    Json,
    extract::{Path, State},
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Response},
    routing::{get, delete},
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use crate::persistence_sqlite::ListRepo;

pub struct ApiState {
    pub db: Arc<ListRepo>,
    pub api_token: String,
}

#[derive(Serialize)]
struct ItemResponse {
    id: u64,
    name: String,
}

#[derive(Serialize)]
struct ListResponse {
    items: Vec<ItemResponse>,
}

#[derive(Deserialize)]
struct AddItemRequest {
    name: String,
}

#[derive(Serialize)]
struct ErrorResponse {
    error: String,
}

fn check_auth(headers: &HeaderMap, expected_token: &str) -> Result<(), Response> {
    let auth = headers
        .get("authorization")
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.strip_prefix("Bearer "));

    match auth {
        Some(token) if token == expected_token => Ok(()),
        _ => Err((
            StatusCode::UNAUTHORIZED,
            Json(ErrorResponse { error: "unauthorized".to_string() }),
        ).into_response()),
    }
}

pub fn router(state: Arc<ApiState>) -> Router {
    Router::new()
        .route("/", get(docs))
        .route("/items", get(list_items).post(add_item).delete(clear_items))
        .route("/items/{id}", delete(remove_item))
        .with_state(state)
}

async fn docs() -> impl IntoResponse {
    (
        StatusCode::OK,
        [("content-type", "text/plain")],
        "Shopping List API

Authentication: Bearer token in Authorization header (all endpoints except this one)

Endpoints:
  GET    /items      - List all items. Returns: {\"items\": [{\"id\": number, \"name\": string}]}
  POST   /items      - Add item. Body: {\"name\": string} (max 100 chars). Returns: {\"id\": number, \"name\": string}
  DELETE /items/{id}  - Remove item by ID. Returns: 204 No Content
  DELETE /items      - Clear all items. Returns: 204 No Content

Error responses: {\"error\": string} with appropriate HTTP status code
",
    )
}

async fn list_items(
    headers: HeaderMap,
    State(state): State<Arc<ApiState>>,
) -> Result<impl IntoResponse, Response> {
    check_auth(&headers, &state.api_token)?;

    match state.db.list().await {
        Ok(items) => Ok(Json(ListResponse {
            items: items.into_iter().map(|i| ItemResponse { id: i.id, name: i.name }).collect(),
        })),
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse { error: e }),
        ).into_response()),
    }
}

async fn add_item(
    headers: HeaderMap,
    State(state): State<Arc<ApiState>>,
    Json(body): Json<AddItemRequest>,
) -> Result<Response, Response> {
    check_auth(&headers, &state.api_token)?;

    match state.db.add_item_returning_id(&body.name).await {
        Ok(item) => Ok((
            StatusCode::CREATED,
            Json(ItemResponse { id: item.id, name: item.name }),
        ).into_response()),
        Err(e) => Err((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse { error: e }),
        ).into_response()),
    }
}

async fn remove_item(
    headers: HeaderMap,
    State(state): State<Arc<ApiState>>,
    Path(id): Path<u64>,
) -> Result<Response, Response> {
    check_auth(&headers, &state.api_token)?;

    match state.db.remove_item(id).await {
        Ok(()) => Ok(StatusCode::NO_CONTENT.into_response()),
        Err(e) => Err((
            StatusCode::NOT_FOUND,
            Json(ErrorResponse { error: e }),
        ).into_response()),
    }
}

async fn clear_items(
    headers: HeaderMap,
    State(state): State<Arc<ApiState>>,
) -> Result<Response, Response> {
    check_auth(&headers, &state.api_token)?;

    match state.db.clear().await {
        Ok(()) => Ok(StatusCode::NO_CONTENT.into_response()),
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse { error: e }),
        ).into_response()),
    }
}
