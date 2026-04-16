use crate::error::{AppError, AppResult};
use crate::AppState;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::routing::get;
use axum::{Json, Router};
use dbnyan_core::connection::{self, Connection, ConnectionInput};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/connections", get(list).post(create))
        .route(
            "/connections/{id}",
            get(fetch).patch(update).delete(remove),
        )
}

async fn list(State(state): State<AppState>) -> AppResult<Json<Vec<Connection>>> {
    Ok(Json(connection::list(&state.pool).await?))
}

async fn create(
    State(state): State<AppState>,
    Json(input): Json<ConnectionInput>,
) -> AppResult<(StatusCode, Json<Connection>)> {
    let c = connection::create(&state.pool, input).await?;
    Ok((StatusCode::CREATED, Json(c)))
}

async fn fetch(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> AppResult<Json<Connection>> {
    connection::get(&state.pool, &id)
        .await?
        .ok_or_else(|| AppError::not_found("connection not found"))
        .map(Json)
}

async fn update(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(input): Json<ConnectionInput>,
) -> AppResult<Json<Connection>> {
    connection::update(&state.pool, &id, input)
        .await?
        .ok_or_else(|| AppError::not_found("connection not found"))
        .map(Json)
}

async fn remove(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> AppResult<StatusCode> {
    if connection::delete(&state.pool, &id).await? {
        Ok(StatusCode::NO_CONTENT)
    } else {
        Err(AppError::not_found("connection not found"))
    }
}
