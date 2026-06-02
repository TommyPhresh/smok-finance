//! CRUD operations for the accounts table

use crate::error::{CoreError, Result};
use crate::models::{Account, AccountType};
use chrono::Utc;
use sqlx::SqlitePool;
use uuid::Uuid;

/// Required fields to create a valid new account
pub struct CreateAccount {
    pub account_name: String,
    pub parent_account_id: Option<Uuid>,
    pub account_type: AccountType,
    pub account_currency: String,
    pub account_sort_order: i32,
    pub account_notes: Option<String>,
}

/// Editable fields in an account
pub struct UpdateAccount {
    pub account_name: Option<String>,
    pub parent_account_id: Option<Option<Uuid>>,
    pub account_type: Option<AccountType>,
    pub account_currency: Option<String>,
    pub account_sort_order: Option<i32>,
    pub account_notes: Option<Option<String>>,
}

/// list_nonarchived_accounts returns all non-archived accounts up to
/// `limit` rows. Results are ordered by account_sort_order, then account_name.
pub async fn list_nonarchived_accounts(pool: &SqlitePool, limit: i64) -> Result<Vec<Account>> {
    let rows = sqlx::query_as!(
        Account,
        r#"
        SELECT
            account_id AS "account_id: Uuid",
            account_name,
            parent_account_id AS "parent_account_id: Uuid",
            account_type AS "account_type: AccountType",
            account_currency,
            account_sort_order,
            account_notes,
            account_created_datetime AS "account_created_datetime: _",
            account_archived_datetime AS "account_archived_datetime: _"
        FROM accounts
        WHERE account_archived_datetime IS NULL
        ORDER BY account_sort_order ASC, account_name ASC
        LIMIT ?
        "#,
        limit
    )
    .fetch_all(pool)
    .await?;
    Ok(rows)
}

/// get_one_account_by_id returns a single account by its account_id.
/// This will return an account even if it has been archived.
pub async fn get_one_account_by_id(pool: &SqlitePool, account_id: Uuid) -> Result<Account> {
    let id_str = account_id.to_string();
    let row = sqlx::query_as!(
        Account,
        r#"
        SELECT
            account_id AS "account_id: Uuid",
            account_name,
            parent_account_id AS "parent_account_id: Uuid",
            account_type AS "account_type: AccountType",
            account_currency,
            account_sort_order,
            account_notes,
            account_created_datetime AS "account_created_datetime: _",
            account_archived_datetime AS "account_archived_datetime: _"
        FROM accounts
        WHERE account_id = ?
        "#,
        id_str
    )
    .fetch_optional(pool)
    .await?
    .ok_or_else(|| CoreError::NotFound(format!("account {} not found", account_id)))?;
    Ok(row)
}

/// create_account creates a new valid account and returns it.
pub async fn create_account(pool: &SqlitePool, input: CreateAccount) -> Result<Account> {
    let account_name = input.account_name.trim().to_string();
    if account_name.is_empty() {
        return Err(CoreError::Validation("account_name cannot be empty!".into()));
    }
    if input.account_currency.trim().is_empty() {
        return Err(CoreError::Validation("account_currency cannot be empty!".into()));
    }
    if let Some(parent_id) = input.parent_account_id {
        let parent = get_one_account_by_id(pool, parent_id).await?;
        if parent.account_archived_datetime.is_some() {
            return Err(CoreError::Validation("cannot assign a child account to an archived parent!".into()));
        }
    }
    let account_id = Uuid::new_v4();
    let id_str = account_id.to_string();
    let parent_str = input.parent_account_id.map(|u| u.to_string());
    let account_type_str = format!("{:?}", input.account_type).to_lowercase();
    let now = Utc::now().to_rfc3339();
    let currency = input.account_currency.trim().to_uppercase();
    sqlx::query!(
        r#"
        INSERT INTO accounts (
            account_id,
            account_name,
            parent_account_id,
            account_type,
            account_currency,
            account_sort_order,
            account_notes,
            account_created_datetime,
            account_archived_datetime
        ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, NULL)
        "#,
        id_str,
        account_name,
        parent_str,
        account_type_str,
        currency,
        input.account_sort_order,
        input.account_notes,
        now,
    )
    .execute(pool)
    .await?;
    get_one_account_by_id(pool, account_id).await
}

/// update_account updates and returns an existing account,
/// only changing provided fields.
pub async fn update_account(pool: &SqlitePool, account_id: Uuid, input: UpdateAccount) -> Result<Account> {
    let existing = get_one_account_by_id(pool, account_id).await?;
    if existing.account_archived_datetime.is_some() {
        return Err(CoreError::Validation("Cannot update an archived account!".into()));
    }

    if let Some(ref account_name) = input.account_name {
        if account_name.trim().is_empty() {
            return Err(CoreError::Validation("account_name cannot be empty on update!".into()));
        }
    }

    if let Some(Some(parent_id)) = input.parent_account_id {
        if parent_id == account_id {
            return Err(CoreError::Validation("An account cannot be updated to be its own parent!".into()));
        }
        let parent = get_one_account_by_id(pool, parent_id).await?;
        if parent.account_archived_datetime.is_some() {
            return Err(CoreError::Validation("Cannot update a child account to an archived parent account!".into()));
        }
    }

    let id_str = account_id.to_string();
    let account_name = input.account_name.unwrap_or(existing.account_name);
    let parent_str = match input.parent_account_id {
        Some(v) => v.map(|u| u.to_string()),
        None    => existing.parent_account_id.map(|u| u.to_string()),
    };
    let account_type = match input.account_type {
        Some(t) => format!("{:?}", t).to_lowercase(),
        None    => format!("{:?}", existing.account_type).to_lowercase(),
    };
    let currency = match input.account_currency {
        Some(c) => c.trim().to_uppercase(),
        None    => existing.account_currency,
    };
    let sort_order = input.account_sort_order.unwrap_or(existing.account_sort_order);
    let notes = match input.account_notes {
        Some(v) => v,
        None    => existing.account_notes,
    };

    sqlx::query!(
        r#"
        UPDATE accounts
        SET
            account_name = ?,
            parent_account_id = ?,
            account_type = ?,
            account_currency = ?,
            account_sort_order = ?,
            account_notes = ?
        WHERE account_id = ?
        "#,
        account_name,
        parent_str,
        account_type,
        currency,
        sort_order,
        notes,
        id_str,
    )
    .execute(pool)
    .await?;
    get_one_account_by_id(pool, account_id).await
}

/// archive_account archives and returns an account (soft delete). Archiving is not a
/// standard delete since the entry still exists, but the account cannot be
/// assigned to. Fails if account has any transaction splits assigned to it
/// or if any of its children are non-archived.
pub async fn archive_account(pool: &SqlitePool, account_id: Uuid) -> Result<Account> {
    let existing = get_one_account_by_id(pool, account_id).await?;
    if existing.account_archived_datetime.is_some() {
        return Err(CoreError::Validation("Cannot archive an already-archived account!".into()));
    }
    
    let id_str = account_id.to_string();

    let child_count: i64 = sqlx::query_scalar!(
        r#"
        SELECT COUNT(*) FROM accounts
        WHERE parent_account_id = ?
        AND account_archived_datetime IS NULL
        "#,
        id_str
    )
    .fetch_one(pool)
    .await?;
    if child_count > 0 {
        return Err(CoreError::Validation("Cannot archive an account with active children!".into()));
    }
    
    let split_count: i64 = sqlx::query_scalar!(
        r#"
        SELECT COUNT(*) FROM transaction_splits WHERE account_id = ?"#,
        id_str,
    )
    .fetch_one(pool)
    .await?;
    if split_count > 0 {
        return Err(CoreError::Validation("Cannot archive an account with recorded transactions; Either re-assign or re-consider!".into()));
    }
    
    let now = Utc::now().to_rfc3339();
    sqlx::query!(
        r#"
        UPDATE accounts
        SET account_archived_datetime = ?
        WHERE account_id = ?
        "#,
        now,
        id_str,
    )
    .execute(pool)
    .await?;
    get_one_account_by_id(pool, account_id).await
}
