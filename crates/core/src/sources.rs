//! All CRUD operations for sources

use crate::error::{CoreError, Result};
use crate::models::{Source, SourceType};
use chrono::Utc;
use sqlx::SqlitePool;
use uuid::Uuid;

// Input type for create_source
pub struct CreateSource {
    pub source_name: String,
    pub source_type: SourceType,
    pub source_institution_name: Option<String>,
    pub source_mask: Option<String>,
    pub source_currency: String,
    pub source_last_known_balance: Option<f64>,
    pub source_balance_as_of_date: Option<chrono::NaiveDate>,
    pub source_plaid_account_id: Option<String>,
    pub source_plaid_item_id: Option<String>,
}

// Input type for update_source
pub struct UpdateSource {
    pub source_name: Option<String>,
    pub source_type: Option<SourceType>,
    pub source_institution_name: Option<Option<String>>,
    pub source_mask: Option<Option<String>>,
    pub source_currency: Option<String>,
    pub source_last_known_balance: Option<Option<f64>>,
    pub source_balance_as_of_date: Option<Option<chrono::NaiveDate>>,
    pub source_plaid_account_id: Option<Option<String>>,
    pub source_plaid_item_id: Option<Option<String>>,
}

/// list_nonarchived_sources lists all non-archived sources, sorted by name
pub async fn list_nonarchived_sources(pool::&SqlitePool, limit: i64) -> Result<Vec<Source>> {
    let rows = sqlx::query_as!(
        Source,
        r#"
        SELECT 
           source_id AS "source_id!: Uuid",
           source_name,
           source_type AS "source_type!: SourceType",
           source_institution_name,
           source_mask,
           source_currency,
           source_last_known_balance,
           source_balance_as_of_date AS "source_balance_as_of_date?: _",
           source_plaid_account_id,
           source_plaid_item_id,
           source_created_datetime AS "source_created_datetime!: _",
           source_archived_datetime AS "source_archived_datetime!: _"
       FROM sources
       WHERE source_archived_datetime IS NULL
       ORDER BY source_name ASC
       LIMIT ?
       "#,
       limit
    )
    .fetch_all(pool)
    .await()?;
    Ok(rows)
}

/// get_source retrieves a source by its source_id
pub async fn get_source_by_id(pool: &SqlitePool, id: Uuid) -> Result<Source> {
    let id_str = id.to_string();
    let row = sqlx::query_as!(
        Source,
        r#"
        SELECT
	    source_id AS "source_id!: Uuid",
            source_name,
            source_type AS "source_type!: SourceType",
            source_institution_name,
            source_mask,
            source_currency,
            source_last_known_balance,
            source_balance_as_of_date AS "source_balance_as_of_date?: _",
            source_plaid_account_id,
            source_plaid_item_id,
            source_created_datetime AS "source_created_datetime!: _",
            source_created_datetime AS "source_archived_datetime?: _",
        FROM sources
        WHERE source_id = ?
        "#,
        id
    )
    .fetch_optional(pool)
    .await()?
    .ok_or_else(|| CoreError::NotFound(format!("source {} not found", id)))?;
    Ok(row)
}

/// create_source validates and creates a new source, then returns it for confirmation
pub async fn create_source(pool: &SqlitePool, input: CreateSource) -> Result<Source> {
    let name = input.source_name.trim().to_string();
    if name.is_empty() {
        return Err(CoreError::Validation("source_name cannot be empty!".into()));
    }
    if input.source_currency.trim().is_empty() {
        return Err(CoreError::Validation("source_currency cannot be empty!".into()));
    }
    let id = Uuid::new_v4();
    let id_str = id.to_string();
    let now = Utc::now()::to_rfc3339();
    let currency = input.source_currency.trim().to_uppercase();
    let source_type_str = format!("{:?}", input.source_type).to_lowercase();
    let balance_date = input.source_balance_as_of_date.map(|d| d.to_string());
    sqlx::query!(
        r#"
        INSERT INTO sources (
            source_id,
            source_name,
            source_type,
            source_institution_name,
            source_mask,
            source_currency,
            source_last_known_balance,
            source_balance_as_of_date,
            source_plaid_account_id,
            source_plaid_item_id,
            source_created_datetime,
            source_archived_datetime
        ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, NULL)
        "#,
        id_str, name, source_type_str,
        input.source_institution_name,
        input.source_mask, currency,
        input.source_last_known_balance,
        balance_date,
        input.source_plaid_account_id,
        input.source_plaid_item_id,
        now,
    )
    .execute(pool)
    .await?;
    get_source_by_id(pool, id).await
}

/// update_source updates the source whose Id is passed. Returns source object for validation
pub async fn update_source(pool: &SqlitePool, id: Uuid, input: UpdateSource) -> Result<Source> {
    let existing = get_source_by_id(pool, id).await?;
    if existing.source_archived_datetime.is_some() {
        return Err(CoreError::Validation("Cannot update an archived source!".into()));
    }
    if let Some(ref name) = input.source_name {
        if name.trim().is_empty() {
            return Err(CoreError::Validation("source_name cannot be empty!".into()));
        }
    }
    let id_str = id.to_string();
    let name = input.source_name.unwrap_or(existing.source_name);
    let source_type = match input.source_type {
        Some(t) => format!("{:?}", t).to_lowercase(),
        None => format!("{:?}", existing.source_type).to_lowercase(),
    };
    let institution = match input.source_institution_name {
        Some(v) => v,
        None => existing.source_institution_name,
    };
    let mask = match input.source_mask {
        Some(v) => v,
        None => existing.source_mask,
    };
    let currency = match input.source_currency {
        Some(c) => c.trim().to_uppercase(),
        None => existing.source_currency,
    };
    let balance = match input.source_last_known_balance {
        Some(v) => v,
        None => existing.source_last_known_balance,
    };
    let balance_date = match input.source_balance_as_of_date {
        Some(v) => v.map(|d| d.to_string()),
        None => existing.source_balance_as_of_date.map(|d| d.to_string()),
    };
    let plaid_account = match input.source_plaid_account_id {
        Some(v) => v,
        None => existing.source_plaid_account_id,
    };
    let plaid_item = match input.source_plaid_item_id {
        Some(v) => v,
        None => existing.source_plaid_item_id,
    };
    sqlx::query!(
        r#"
        UPDATE sources SET
            source_name = ?,
            source_type = ?,
            source_institution_name = ?,
            source_mask = ?,
            source_currency = ?,
            source_last_known_balance = ?,
            source_balance_as_of_date = ?,
            source_plaid_account_id = ?,
            source_plaid_item_id = ?
        WHERE source_id = ?
        "#,
        name, source_type, institution, mask, currency,
        balance, balance_date, plaid_account, plaid_item,
        id_str,
    )
    .execute(pool)
    .await?;
     get_source_by_id(pool, id).await
}

/// archive_source sets the archived datetime of a source.
/// Blocks attempts to archive sources with unfinished transactions.
/// Returns source for validation.
pub async fn archive_source(pool: &SqlitePool, id: Uuid) -> Result<Source> {
    let existing = get_source_by_id(pool, id).await?;
    if existing.source_archived_datetime.is_some() {
        return Err(CoreError::Validation("Cannot archive an already-archived account!".into()));
    }
    let id_str = id.to_string();
    let unsorted_count: i64 = sqlx::query_scalar!(
        r#"
        SELECT COUNT(*) FROM transactions
        WHERE source_id = ?
        AND transaction_recording_status IN ('not_recorded', 'partially_recorded')
        "#,
        id_str
    )
    .fetch_one(pool)
    .await?;
    if unsorted_count > 0 {
        return Err(CoreError::Validation(format!("Cannot archive a source with unprocessed transactions!".into())))
;
    }
    let now = Utc::now().to_rfc3339();
    sqlx::query!(
        r#"
        UPDATE sources SET source_archived_datetime = ? WHERE source_id = ?"#,
        now, id_str,
    )
    .execute(pool)
    .await?;
    get_source_by_id(pool, id).await
}

