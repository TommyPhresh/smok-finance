//! Domain model struct definitions.

use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

// Profiles - users
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Profile {
    pub profile_id: Uuid,
    pub profile_display_name: String,
    pub profile_created_datetime: DateTime<Utc>,
    pub profile_archived_datetime: Option<DateTime<Utc>>,
}

// Accounts - user-defined chart of accounts
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Account {
    pub account_id: Uuid,
    pub account_name: String,
    pub parent_account_id: Option<Uuid>,
    pub account_type: AccountType,
    pub account_currency: String,
    pub account_sort_order: i32,
    pub account_notes: Option<String>,
    pub account_created_datetime: DateTime<Utc>,
    pub account_archived_datetime: Option<DateTime<Utc>>,
}

// AccountType - basic account categories
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name="TEXT")]
#[sqlx(rename_all="snake_case")]
pub enum AccountType {
    Income,
    Expense,
    Asset,
    Liability,
    Equity,
}

// Sources - places Smok Finance will pull transactions from (non-cash)
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Source {
    pub source_id: Uuid,
    pub source_name: String,
    pub source_type: SourceType,
    pub source_institution_name: Option<String>,
    pub source_mask: Option<String>,
    pub source_currency: String,
    pub source_last_known_balance: Option<f64>,
    pub source_balance_as_of_date: Option<NaiveDate>,
    pub source_plaid_account_id: Option<String>,
    pub source_plaid_item_id: Option<String>,
    pub source_created_datetime: DateTime<Utc>,
    pub source_archived_datetime: Option<DateTime<Utc>>,
}

// SourceType - basic bank account categories
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name="TEXT")]
#[sqlx(rename_all="snake_case")]
pub enum SourceType {
    Checking,
    Savings,
    CreditCard,
    Loan,
    Mortgage,
    Cash,
    Investment,
    Other,
}

// Payees - people/vendors user spends money at
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Payees {
    pub payee_id: Uuid,
    pub payee_name: String,
    pub payee_phone: Option<String>,
    pub payee_address: Option<String>,
    pub payee_website: Option<String>,
    pub payee_default_account_id: Option<Uuid>,
    pub payee_created_datetime: DateTime<Utc>,
}

// Transactions - what user spends money on
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Transaction {
    pub transaction_id: Uuid,
    pub source_id: Uuid,
    pub recurring_transaction_id: Option<Uuid>,
    pub transaction_date: NaiveDate,
    pub transaction_amount: f64,
    pub transaction_memo: String,
    pub payee_id: Option<Uuid>,
    pub plaid_transaction_id: Option<String>,
    pub transaction_details: Option<String>,
    pub transaction_recording_status: TransactionRecordingStatus,
    pub transaction_recorded_datetime: DateTime<Utc>,
}

// Transaction Recording Status - how far along the user is on sorting it
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name="TEXT")]
#[sqlx(rename_all="snake_case")]
pub enum TransactionRecordingStatus {
    NotRecorded,
    PartiallyRecorded,
    Recorded,
    Excluded,
}

// Transaction Splits - relationship between a transaction and an account; 
// even singleton transactions will have a split
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct TransactionSplit {
    pub transaction_split_id: Uuid,
    pub transaction_id: Uuid,
    pub account_id: Uuid,
    pub transaction_split_amount: f64,
    pub transaction_split_notes: Option<String>,
    pub transaction_split_recorded_datetime: DateTime<Utc>,
}

// Tags - Freeform user-defined labels to be applied to transactions
// independent of accounts (i.e. "Boston vacation")
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Tag {
    pub tag_id: Uuid,
    pub tag_name: String,
    pub tag_color: Option<String>,
    pub tag_created_datetime: DateTime<Utc>,
}

// Transaction Tags - the relationship between a Tag and a Transaction
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct TransactionTag {
    pub transaction_id: Uuid,
    pub tag_id: Uuid,
}

// Budgets - same as account but has an expected value for some predefined time period
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Budget {
    pub budget_id: Uuid,
    pub account_id: Uuid,
    pub budget_period_start_date: NaiveDate,
    pub budget_period_end_date: NaiveDate,
    pub budget_amount: f64,
    pub budget_created_datetime: DateTime<Utc>,
}

// Recurring Transactions - transactions with start/end dates and frequencies so
// users don't have to sort them every cycle
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct RecurringTransaction {
    pub recurring_transaction_id: Uuid,
    pub recurring_transaction_description: String,
    pub recurring_transaction_amount: f64,
    pub source_id: Uuid,
    pub account_id: Uuid,
    pub payee_id: Option<Uuid>,
    pub recurring_transaction_frequency: RecurringTransactionFrequency,
    pub recurring_transaction_next_date: NaiveDate,
    pub recurring_transaction_end_date: Option<NaiveDate>,
    pub recurring_transaction_is_active: bool,
    pub recurring_transaction_created_datetime: DateTime<Utc>,
}

// Recurring Transaction Frequency - frequency the recurring transaction will recur at
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name="TEXT")]
#[sqlx(rename_all="snake_case")]
pub enum RecurringTransactionFrequency {
    Daily,
    Weekly,
    Biweekly,
    Monthly,
    Quarterly,
    Annually,
    Custom,
}

// Goals - users can define these and see how their performance stacks up against
// their own expectations; how it hurts and how it helps them
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Goal {
    pub goal_id: Uuid,
    pub goal_name: String,
    pub goal_type: GoalType,
    pub account_id: Uuid,
    pub goal_target_amount: f64,
    pub goal_current_amount: f64,
    pub goal_target_date: Option<NaiveDate>,
    pub goal_priority: i32,
    pub goal_notes: Option<String>,
    pub goal_created_datetime: DateTime<Utc>,
    pub goal_archived_datetime: Option<DateTime<Utc>>,
}

// Goal Type - default options and custom
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name="TEXT")]
#[sqlx(rename_all="snake_case")]
pub enum GoalType {
    Savings,
    DebtPayoff,
    Retirement,
    Custom,
}

// Goal Conflicts - an explicit record of when two goals couldn't work at the
// same time, and the choice the user made
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct GoalConflict {
    pub goal_conflict_id: Uuid,
    pub goal_id_a: Uuid,
    pub goal_id_b: Uuid,
    pub goal_conflict_detected_datetime: DateTime<Utc>,
    pub goal_conflict_resolution_status: GoalConflictResolutionStatus,
    pub goal_conflict_resolved_datetime: Option<DateTime<Utc>>,
}

// Goal Conflict Resolution Statuses - which way did the user go
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name="TEXT")]
#[sqlx(rename_all="snake_case")]
pub enum GoalConflictResolutionStatus {
    Unresolved,
    PrioritizedA,
    PrioritizedB,
    DeferredBoth,
}

// Assumption Sets - forecast inputs the user can define 
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct AssumptionSet {
    pub assumption_set_id: Uuid,
    pub assumption_set_expected_return_rate: f64,
    pub assumption_set_inflation_rate: f64,
    pub assumption_set_income_growth_rate: f64,
    pub assumption_set_retirement_age: Option<i32>,
    pub assumption_set_notes: Option<String>,
    pub assumption_set_created_datetime: DateTime<Utc>,
    pub assumption_set_is_active: bool,
}

// Life Events - drastic life-altering events, good or bad, that will shift
// the user's life going forward
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct LifeEvent {
    pub life_event_id: Uuid,
    pub life_event_name: String,
    pub life_event_date: NaiveDate,
    pub life_event_type: LifeEventType,
    pub life_event_amount: f64,
    pub life_event_monthly_affect: Option<f64>,
    pub account_id: Option<Uuid>,
    pub life_event_notes: Option<String>,
    pub life_event_created_datetime: DateTime<Utc>,
}

// Life Event Types - basic sorting; one-time or recurring?
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name="TEXT")]
#[sqlx(rename_all="snake_case")]
pub enum LifeEventType {
    OneTimeExpense,
    OneTimeIncome,
    RecurringChange,
}

// Exchanges - conversion between one currency to another thru time
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Exchange {
    pub exchange_id: Uuid,
    pub exchange_from_currency: String,
    pub exchange_to_currency: String,
    pub exchange_rate: f64,
    pub exchange_effective_date: NaiveDate,
    pub exchange_created_datetime: DateTime<Utc>,
}

// Sync State - sync info for user current device
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct SyncState {
    pub device_id: Uuid,
    pub device_last_synced_datetime: Option<DateTime<Utc>>,
    pub sync_token: Option<String>,
}


