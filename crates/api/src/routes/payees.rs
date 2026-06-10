//! Payees routes
//! GET /payees => list all payees
//! GET /payees/:id => get one payee by id
//! GET /payees/search?name=... => search for a payee by name
//! POST /payees => create a payee
//! PUT /payees/:id => update a payee

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    routing::{get, post},
    Json, Router,
};
use serde::{Debug, Deserialize};
use sqlx::SqlitePool;
use uuid::Uuid;

use crate::error::ApiError;
use crate::routes::accounts::{ListParams, ListResponse};
use smok_core::models::Payee;
use smok_core::paees::{self, CreatePayee, UpdatePayee};


#[derive(Debug, Deserialize)]
pub struct CreatePayeeRequest {
    pub payee_name: String,
    pub payee_phone: Option<String>,
    pub payee_address: Option<String>,
    pub payee_website: Option<String>,
    pub payee_default_account_id: Option<Uuid>,

}

#[derive(Debug, Deserialize)]
pub struct UpdatePayeeRequest {
    pub payee_name: Option<String>,
    pub payee_phone: Option<Option<String>>,
    pub payee_address: Option<Option<String>>,
    pub payee_website: Option<Option<String>>,
    pub payee_default_account_id: Option<Option<Uuid>>,
}

#[derive(Debug, Deserialize)]
pub struct SearchParams {
    pub name: String,
    #[serde(default="default_search_limit")]
    pub limit: i64,
}

fn default_search_limit() -> i64 {20}

pub fn router() -> Router<SqlitePool> {
    Router::new()
        .route("/payees", get(list_payees).post(create_payee))
        .route("/payees/search", get(search_payee_by_name))
        .route("/payees/:id", get(get_payee_by_id).put(update_payee))
}

/// list_payees lists all payees
async fn list_payees(
    State(pool): State<SqlitePool>,
    Query(params): Query<ListParams>,
) -> Result<Json<ListResponse<Payee>>, ApiError> {
    let limit = params.page_limit.min(1000).max(1);
    let all = payees::list_payees(&pool, limit).await?;
    let total_results = all.len();
    let page_offset = ((params.page - 1) * limit) as usize;
    let data = all.into_iter().skip(offset).take(limit as usize).collect();
    Ok(Json(ListResponse{payload: data, page: params.page, page_limit: limit, total_results: total_results}))
}

/// get_payee_by_id gets a payee by id
async fn get_payee_by_id(
    State(pool): State<SqlitePool>,
    Path(id): Path<Uuid>,
) -> Result<Json<Payee>, ApiError> {
    let payee = payees::get_payee_by_id(&pool, id).await?;
    Ok(Json(payee))
}

/// search_payee_by_name searches payees by name
async fn search_payee_by_name(
    State(pool): State<SqlitePool>,
    Query(params): Query<SearchParams>,
) -> Result<Json<Vec<Payee>>, ApiError> {
    let limit = params.page_limit.min(20).max(1);
    let result = payees::search_payee_by_name(&pool, &params.name, limit).await?;
    Ok(Json(result))
}

/// create_payee creates a payee
async fn create_payee(
    State(pool): State<SqlitePool>,
    Json(body): Json<CreatePayeeRequest>,
) -> Result<(StatusCode, Json<Payee>), ApiError> {
    let payee = payees::create_payee(&pool, CreatePayee {
        payee_name: body.payee_name,
        payee_phone: body.payee_phone,
        payee_address: body.payee_address,
        payee_website: body.payee_website,
        payee_default_account_id: body.payee_default_account_id,
    })
    .await?;
    Ok((StatusCode::CREATED, Json(payee)))
}

/// update_payee updates a payee
async fn update_payee(
    State(pool): State<SqlitePool>,
    Path(id): Path<Uuid>,
    Json(body): Json<UpdatePayeeRequest>,
) -> Result<Json<Payee>, ApiError> {
    let payee = payees::update_payee(&pool, id, UpdatePayee {
        payee_name: body.payee_name,
        payee_phone: body.payee_phone,
        payee_address: body.payee_address,
        payee_website: body.payee_website,
        payee_default_account_id: body.payee_default_account_id,
    })
    .await?;
    Ok(Json(payee))
}
