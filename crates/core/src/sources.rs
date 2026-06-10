//! Payee CRUD operations

use crate::error::{CoreError, Result};
use crate::models::Payee;
use chrono::Utc;
use sqlx::SqlitePool;
use uuid::Uuid;

/// create_payee input struct
pub struct CreatePayee {
    pub payee_name: String,
    pub payee_phone: Option<String>,
    pub payee_address: Option<String>,
    pub payee_website: Option<String>,
    pub payee_default_account_id: Option<Uuid>,
}

/// update_payee input struct
pub struct UpdatePayee {
    pub payee_name: Option<String>,
    pub payee_phone: Option<Option<String>>,
    pub payee_address: Option<Option<String>>,
    pub payee_website: Option<Option<String>>,
    pub payee_default_account_id: Option<Option<Uuid>>,
}

/// list_payees lists all payees
pub async fn list_payees(pool: &SqlitePool, limit: i64) -> Result<Vec<Payee>> {
    let rows = sqlx::query_as!(
        Payee,
        r#"
        SELECT 
            payee_id AS "payee_id!: Uuid",
            payee_name,
            payee_phone,
            payee_address,
            payee_website,
            payee_default_account_id AS "payee_default_account_id?: Uuid",
            payee_created_datetime AS "payee_created_datetime!: _"
        FROM payees
        ORDER BY payee_name ASC
        LIMIT ?
        "#,
        limit
    )
    .fetch_all(pool)
    .await?;
    Ok(rows)
}

/// get_payee_by_id gets a payee by its payee_id
pub async fn get_payee_by_id(pool: &SqlitePool, id: Uuid) -> Result<Payee> {
    let id_str = id.to_string();
    let row = sqlx::query_as!(
        Payee,
        r#"
        SELECT
            payee_id AS "payee_id!: Uuid",
            payee_name,
            payee_phone,
            payee_address,
            payee_website,
            payee_default_account_id AS "payee_default_account_id?: Uuid",
            payee_created_datetime AS "payee_created_datetime!: _"
        FROM payees
        WHERE payee_id = ?
        "#,
        id_str
    )
    .fetch_optional(pool)
    .await?
    .ok_or_else(|| CoreError::NotFound(format!("Payee {} not found!", id)))?;
    Ok(row)
}

/// search_payee_by_name is a case-insensitive fuzzy match
pub async fn search_payee_by_name(pool: &SqlitePool, name: &str, limit: i64) -> Result<Vec<Payee>> {
    let pattern = format!("%{}%", name.trim());
    let rows = sqlx::query_as!(
        Payee,
        r#"
        SELECT
            payee_id AS "payee_id!: Uuid",
            payee_name,
            payee_phone,
            payee_address,
            payee_website,
            payee_default_account_id AS "payee_default_account_id?: Uuid",
            payee_created_datetime AS "payee_created_datetime!: _"
        FROM payees
        WHERE payee_name LIKE ?
        ORDER BY payee_name ASC
        LIMIT ?
        "#,
        pattern,
        limit
    )
    .fetch_all(pool)
    .await?;
    Ok(rows)
}

/// create_payee creates and returns a new payee
pub async fn create_payee(pool: &SqlitePool, input: CreatePayee) -> Result<Payee> {
    let name = input.payee_name.trim().to_string();
    if name.is_empty() {
        return Err(CoreError::Validation("Payee name cannot be empty!".into()));
    }
    let id = Uuid::new_v4();
    let id_str = id.to_string();
    let now = Utc::now().to_rfc3339();
    let account_str = input.payee_default_account_id.map(|u| u.to_string());
    sqlx::query!(
        r#"
        INSERT INTO payees (
            payee_id,
            payee_name,
            payee_phone,
            payee_address,
            payee_website,
            payee_default_account_id,
            payee_created_datetime
        ) VALUE (?, ?, ?, ?, ?, ?, ?)
        "#,
        id_str, name,
        input.payee_phone, input.payee_address,
        input.payee_address, input.payee_website,
        account_str, now,
    )
    .execute(pool)
    .await?;
    get_payee_by_id(pool, id).await
}

/// update_payee updates and returns a payee
pub async fn update_payee(pool: &SqlitePool, id: Uuid, input: UpdatePayee) -> Result<Payee> {
    let existing = get_payee_by_id(pool, id).await?;
    if let Some(ref name) = input.payee_name {
        if name.trim().is_empty() {
            return Err(CoreError::Validation("Payee name cannot be empty!".into()));
        }
    }
    let id_str = id.to_string();
    let name = input.payee_name.unwrap_or(existing.payee_name);
    let phone = match input.payee_phone {
        Some(v) => v,
        None => existing.payee_phone,
    };
    let address = match input.payee_address {
        Some(v) => v,
        None => existing.payee_address,
    };
    let website = match input.payee_website {
        Some(v) => v,
        None => existing.payee_website,
    };
    let account_str = match input.payee_default_account_id {
        Some(v) => v,
        None => existing.payee_default_account_id.map(|u| u.to_string()),
    };
    sqlx::query!(
        r#"
        UPDATE payees SET
            payee_name = ?,
            payee_phone = ?,
            payee_address = ?,
            payee_website = ?,
            payee_default_account_id = ?
        WHERE payee_id = ?
        "#,
        name, phone, address, website, account_str,
        id_str,
    )
    .execute(pool)
    .await?;
    get_payee_by_id(pool, id).await
}
