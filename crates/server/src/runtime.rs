use crate::error::{AppError, AppResult};
use crate::AppState;
use axum::extract::{Path, Query, State};
use axum::routing::{delete, get, post};
use axum::{Json, Router};
use dbnyan_core::{
    connection, favorites, history, mysql,
    query::{self, QueryResult},
    session::Session,
};
use serde::Deserialize;
use serde_json::{json, Value};
use std::sync::Arc;
use std::time::Instant;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/connections/{id}/databases", get(list_databases))
        .route("/connections/{id}/databases/{db}/tables", get(list_tables))
        .route(
            "/connections/{id}/databases/{db}/tables/{table}/schema",
            get(describe_table),
        )
        .route(
            "/connections/{id}/databases/{db}/tables/{table}/rows",
            get(preview_rows),
        )
        .route("/connections/{id}/query", post(run_query))
        .route(
            "/connections/{id}/history",
            get(list_history).delete(clear_history),
        )
        .route(
            "/connections/{id}/history/{hid}",
            delete(delete_history_entry),
        )
        .route(
            "/connections/{id}/favorites",
            get(list_favorites).post(create_favorite),
        )
        .route(
            "/connections/{id}/favorites/{fid}",
            axum::routing::patch(update_favorite).delete(delete_favorite),
        )
        .route("/connections/{id}/session", delete(close_session))
}

async fn open_session(state: &AppState, id: &str) -> AppResult<Arc<Session>> {
    let conn = connection::get(&state.pool, id)
        .await?
        .ok_or_else(|| AppError::not_found("connection not found"))?;
    Ok(state.sessions.get_or_open(&conn).await?)
}

async fn list_databases(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> AppResult<Json<Vec<String>>> {
    let session = open_session(&state, &id).await?;
    Ok(Json(mysql::list_databases(&session.pool).await?))
}

async fn list_tables(
    State(state): State<AppState>,
    Path((id, db)): Path<(String, String)>,
) -> AppResult<Json<Vec<mysql::TableInfo>>> {
    let session = open_session(&state, &id).await?;
    Ok(Json(mysql::list_tables(&session.pool, &db).await?))
}

async fn describe_table(
    State(state): State<AppState>,
    Path((id, db, table)): Path<(String, String, String)>,
) -> AppResult<Json<Vec<mysql::ColumnInfo>>> {
    let session = open_session(&state, &id).await?;
    Ok(Json(
        mysql::describe_table(&session.pool, &db, &table).await?,
    ))
}

#[derive(Deserialize)]
struct PreviewQuery {
    #[serde(default = "default_limit")]
    limit: u32,
    #[serde(default)]
    sort: Option<String>,
    #[serde(default)]
    dir: Option<String>,
}
fn default_limit() -> u32 {
    200
}

async fn preview_rows(
    State(state): State<AppState>,
    Path((id, db, table)): Path<(String, String, String)>,
    Query(q): Query<PreviewQuery>,
) -> AppResult<Json<mysql::RowSet>> {
    let session = open_session(&state, &id).await?;
    let sort_desc = matches!(q.dir.as_deref(), Some("desc") | Some("DESC"));
    Ok(Json(
        mysql::preview_rows(&session.pool, &db, &table, q.limit, q.sort.as_deref(), sort_desc)
            .await?,
    ))
}

#[derive(Deserialize)]
struct QueryBody {
    sql: String,
    /// Optional — purely informational, used for history grouping.
    /// MySQL session keeps its own current db; this does not change it.
    database: Option<String>,
}

#[derive(serde::Serialize)]
struct QueryReply {
    #[serde(flatten)]
    result: QueryResult,
    duration_ms: i64,
}

async fn run_query(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(body): Json<QueryBody>,
) -> AppResult<Json<QueryReply>> {
    let session = open_session(&state, &id).await?;
    let started = Instant::now();
    let result = query::run(&session.pool, &body.sql).await;
    let duration_ms = i64::try_from(started.elapsed().as_millis()).unwrap_or(i64::MAX);

    match result {
        Ok(qr) => {
            let (rows_affected, rows_returned) = match &qr {
                QueryResult::Rows { returned, .. } => (None, Some(*returned as i64)),
                QueryResult::Affected { rows_affected, .. } => {
                    (Some(*rows_affected as i64), None)
                }
            };
            let _ = history::record(
                &state.pool,
                &id,
                body.database.as_deref(),
                &body.sql,
                true,
                None,
                rows_affected,
                rows_returned,
                duration_ms,
            )
            .await;
            Ok(Json(QueryReply {
                result: qr,
                duration_ms,
            }))
        }
        Err(e) => {
            let msg = format!("{e:#}");
            let _ = history::record(
                &state.pool,
                &id,
                body.database.as_deref(),
                &body.sql,
                false,
                Some(&msg),
                None,
                None,
                duration_ms,
            )
            .await;
            Err(AppError::from(e))
        }
    }
}

#[derive(Deserialize)]
struct HistoryQuery {
    #[serde(default = "default_history_limit")]
    limit: u32,
}
fn default_history_limit() -> u32 {
    50
}

async fn list_history(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Query(q): Query<HistoryQuery>,
) -> AppResult<Json<Vec<history::HistoryEntry>>> {
    Ok(Json(history::list(&state.pool, &id, q.limit).await?))
}

async fn delete_history_entry(
    State(state): State<AppState>,
    Path((_id, hid)): Path<(String, String)>,
) -> AppResult<Json<Value>> {
    let deleted = history::delete(&state.pool, &hid).await?;
    Ok(Json(json!({ "deleted": deleted })))
}

async fn clear_history(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> AppResult<Json<Value>> {
    let n = history::clear(&state.pool, &id).await?;
    Ok(Json(json!({ "cleared": n })))
}

async fn list_favorites(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> AppResult<Json<Vec<favorites::Favorite>>> {
    Ok(Json(favorites::list(&state.pool, &id).await?))
}

#[derive(Deserialize)]
struct CreateFavoriteBody {
    name: String,
    sql: String,
}

async fn create_favorite(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(body): Json<CreateFavoriteBody>,
) -> AppResult<Json<favorites::Favorite>> {
    Ok(Json(
        favorites::create(&state.pool, &id, &body.name, &body.sql).await?,
    ))
}

#[derive(Deserialize)]
struct UpdateFavoriteBody {
    #[serde(default)]
    name: Option<String>,
    #[serde(default)]
    sql: Option<String>,
}

async fn update_favorite(
    State(state): State<AppState>,
    Path((_id, fid)): Path<(String, String)>,
    Json(body): Json<UpdateFavoriteBody>,
) -> AppResult<Json<favorites::Favorite>> {
    favorites::update(&state.pool, &fid, body.name.as_deref(), body.sql.as_deref())
        .await?
        .ok_or_else(|| AppError::not_found("favorite not found"))
        .map(Json)
}

async fn delete_favorite(
    State(state): State<AppState>,
    Path((_id, fid)): Path<(String, String)>,
) -> AppResult<Json<Value>> {
    let deleted = favorites::delete(&state.pool, &fid).await?;
    Ok(Json(json!({ "deleted": deleted })))
}

async fn close_session(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> AppResult<Json<Value>> {
    let was_active = state.sessions.close(&id).await;
    Ok(Json(json!({ "closed": was_active })))
}
