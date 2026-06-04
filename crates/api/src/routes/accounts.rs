//! Accounts routes
//! GET /accounts 		=> list all active accounts (limit 1000 rows)
//! GET /accounts/:id 		=> get one account
//! POST /accounts		=> create an account
//! PUT /accounts/:id		=> update an account
//! POST .accounts/:id/archive 	=> soft delete (archive) an account 

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    routing::{get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;
use uuid::Uuid;

use crate::error::ApiError;
use smok_core::accounts::{self, CreateAccount, UpdateAccount};
use smok_core::models::AccountType;

#[derive(Debug, Deserialize)]
pub struct ListParams {
    /// Page number, starting at 1 [1]
    #[serde(default="default_page")]
    pub page: i64,
    /// Rows per page. Max 1000 [1000]
    #[serde(default="default_limit")]
    pub page_limit: i64,
}
fn default_page() -> i64 {1}
fn default_limit() -> i64 {1000}

#[derive(Debug, Serialize)]
pub struct ListResponse<T> {
    pub payload: Vec<T>,
    pub page: i64,
    pub page_limit: i64,
    pub total_results: usize,
}

#[derive(Debug, Deserialize)]
pub struct CreateAccountRequest {
    pub account_name: String,
    pub parent_account_id: Option<Uuid>,
    pub account_type: AccountType,
    pub account_currency: Option<String>,
    pub account_sort_order: Option<i64>,
    pub account_notes: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateAccountRequest {
    pub account_name: Option<String>,
    pub parent_account_id: Option<Option<Uuid>>,
    pub account_type: Option<AccountType>,
    pub account_currency: Option<String>,
    pub account_sort_order: Option<i64>,
    pub account_notes: Option<String>,
}

pub fn router() -> Router<SqlitePool> {
    Router::new()
        .route("/accounts", get(list_accounts).post(create_account))
        .route("/accounts/:id", get(get_account).put(update_account))
        .route("/accounts/:id/archive", post(archive_account))
}

/// GET /accounts returns all active accounts
async fn list_accounts(
    State(pool): State<SqlitePool>,
    Query(params): Query<ListParams>,
) -> Result<Json<ListResponse<smok_core::models::Account>>, ApiError> {
    let limit = params.page_limit.min(1000).max(1);
    let all = accounts::list_nonarchived_accounts(&pool, limit).await?;
    let total_results = all.len();
    let page_offset = ((params.page - 1) * limit) as usize;
    let data = all.into_iter().skip(page_offset).take(limit as usize).collect();
    Ok(Json(ListResponse {payload:data, page:params.page, page_limit:limit, total_results:total_results}))
}

/// GET /accounts/:id returns either one account with the ID or an error
async fn get_account(
    State(pool): State<SqlitePool>,
    Path(id): Path<Uuid>,
) -> Result<Json<smok_core::models::Account>, ApiError> {
    let account = accounts::get_one_account_by_id(&pool, id).await?;
    Ok(Json(account))
}

/// POST /accounts creates a new account
async fn create_account(
    State(pool): State<SqlitePool>,
    Json(body): Json<CreateAccountRequest>,
) -> Result<(StatusCode, Json<smok_core::models::Account>), ApiError> {
    let account = accounts::create_account(&pool, CreateAccount {
        account_name: body.account_name,
        parent_account_id: body.parent_account_id,
        account_type: body.account_type,
        account_currency: body.account_currency.unwrap_or_else(|| "USD".into()),
        account_sort_order: body.account_sort_order.unwrap_or(0),
        account_notes: body.account_notes,
    })
    .await?;
    Ok((StatusCode::CREATED, Json(account)))
}

/// PUT /accounts/:id updates an account
async fn update_account(
    State(pool): State<SqlitePool>,
    Path(id): Path<Uuid>,
    Json(body): Json<UpdateAccountRequest>,
) -> Result<Json<smok_core::models::Account>, ApiError> {
    let account = accounts::update_account(&pool, id, UpdateAccount {
        account_name: body.account_name,
        parent_account_id: body.parent_account_id,
        account_type: body.account_type,
        account_currency: body.account_currency,
        account_sort_order: body.account_sort_order,
        account_notes: Some(body.account_notes),
    })
    .await?;
    Ok(Json(account))
}

/// POST /accounts/:id/archive soft deletes (archives) an account
async fn archive_account(
    State(pool): State<SqlitePool>,
    Path(id): Path<Uuid>,
) -> Result<Json<smok_core::models::Account>, ApiError> {
    let account = accounts::archive_account(&pool, id).await?;
    Ok(Json(account))
}
