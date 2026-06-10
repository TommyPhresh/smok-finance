//! Sources routes
//! GET /sources => list all nonarchived sources
//! GET /sources:id => get a single source by its id
//! POST /sources => create a new source
//! PUT /sources:id => update a source
//! POST /sources:id/archive => soft delete/archive a source

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    routing::{get, post},
    Json, Router
};
use chrono::NaiveDate;
use serde::{Deserialize};
use sqlx::SqlitePool;
use uuid::Uuid;

use crate::error::ApiError;
use crate::routes::accounts::{ListParams, ListResponse};
use smok_core::models::{Source, SourceType};
use smok_core::sources::{self, CreateSource, UpdateSource};


#[derive(Debug, Deserialize)]
pub struct CreateSourceRequest {
    pub source_name: String,
    pub source_type: SourceType,
    pub source_institution_name: Option<String>,
    pub source_mask: Option<String>,
    pub source_currency: Option<String>,
    pub source_last_known_balance: Option<f64>,
    pub source_balance_as_of_date: Option<NaiveDate>,
    pub source_plaid_account_id: Option<String>,
    pub source_plaid_item_id: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateSourceRequest {
    pub source_name: Option<String>,
    pub source_type: Option<SourceType>,
    pub source_institution_name: Option<Option<String>>,
    pub source_mask: Option<Option<String>>,
    pub source_currency: Option<String>,
    pub source_last_known_balance: Option<Option<f64>>,
    pub source_balance_as_of_date: Option<Option<NaiveDate>>,
    pub source_plaid_account_id: Option<Option<String>>,
    pub source_plaid_item_id: Option<Option<String>>,
}

pub fn router() -> Router<SqlitePool> {
    Router::new()
        .route("/sources", get(list_nonarchived_sources).post(create_source))
        .route("/sources/:id", get(get_source_by_id).put(update_source))
        .route("/sources/:id/archive", post(archive_source))
}

/// GET /sources lists all nonarchived sources
async fn list_nonarchived_sources(
    State(pool): State<SqlitePool>,
    Query(params): Query<ListParams>,
) -> Result<Json<ListResponse<Source>>, ApiError> {
    let limit = params.page_limit.min(1000).max(1);
    let all = sources::list_nonarchived_sources(&pool, limit).await?;
    let total_results = all.len();
    let page_offset = ((params.page - 1) * limit) as usize;
    let data = all.into_iter().skip(page_offset).take(limit as usize).collect();
    Ok(Json(ListResponse {payload: data, page: params.page, page_limit: limit, total_results: total_results}))
}

/// GET /sources/:id retrieves one source by its source_id
async fn get_source_by_id(
    State(pool): State<SqlitePool>,
    Path(id): Path<Uuid>,
) -> Result<Json<Source>, ApiError> {
    let source = sources::get_source_by_id(&pool, id).await?;
    Ok(Json(source))
}

/// POST /sources creates a new source and returns for validation
async fn create_source(
    State(pool): State<SqlitePool>,
    Json(body): Json<CreateSourceRequest>,
) -> Result<(StatusCode, Json<Source>), ApiError> {
    let source = sources::create_source(&pool, CreateSource {
        source_name: body.source_name,
        source_type: body.source_type,
        source_institution_name: body.source_institution_name,
        source_mask: body.source_mask,
        source_currency: body.source_currency.unwrap_or_else(|| "USD".into()),
        source_last_known_balance: body.source_last_known_balance,
        source_balance_as_of_date: body.source_balance_as_of_date,
        source_plaid_account_id: body.source_plaid_account_id,
        source_plaid_item_id: body.source_plaid_item_id,
    })
    .await?;
    Ok((StatusCode::CREATED, Json(source)))
}

/// PUT /sources/:id updates a source
async fn update_source(
    State(pool): State<SqlitePool>,
    Path(id): Path<Uuid>,
    Json(body): Json<UpdateSourceRequest>,
) -> Result<Json<Source>, ApiError> {
    let source = sources::update_source(&pool, id, UpdateSource {
        source_name: body.source_name,
        source_type: body.source_type,
        source_institution_name: body.source_institution_name,
        source_mask: body.source_mask,
        source_currency: body.source_currency,
        source_last_known_balance: body.source_last_known_balance,
        source_balance_as_of_date: body.source_balance_as_of_date,
        source_plaid_account_id: body.source_plaid_account_id,
        source_plaid_item_id: body.source_plaid_item_id,
    })
    .await?;
    Ok(Json(source))
}

/// POST /sources/:id/archive archives a source
async fn archive_source(
    State(pool): State<SqlitePool>,
    Path(id): Path<Uuid>,
) -> Result<Json<Source>, ApiError> {
    let source = sources::archive_source(&pool, id).await?;
    Ok(Json(source))
}

