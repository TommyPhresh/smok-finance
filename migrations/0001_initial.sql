-- Migration 0001: defining schema

PRAGMA journal_mode = WAL;
PRAGMA foreign_keys = ON;

CREATE TABLE IF NOT EXISTS profiles (
    profile_id TEXT PRIMARY KEY,
    profile_display_name TEXT NOT NULL,
    profile_created_datetime TEXT NOT NULL,
    profile_archived_datetime TEXT
);

CREATE TABLE IF NOT EXISTS accounts (
    account_id TEXT PRIMARY KEY,
    account_name TEXT NOT NULL,
    parent_account_id TEXT REFERENCES accounts(account_id) ON DELETE RESTRICT,
    account_type TEXT NOT NULL
        CHECK(account_type IN ('income','expense','asset','liability','equity')),
    account_currency TEXT NOT NULL DEFAULT 'USD',
    account_sort_order INTEGER NOT NULL DEFAULT 0,
    account_notes TEXT,
    account_created_datetime TEXT NOT NULL,
    account_archived_datetime TEXT
);
CREATE INDEX IF NOT EXISTS idx_accounts_parent ON accounts(parent_account_id);

CREATE TABLE IF NOT EXISTS sources (
    source_id TEXT PRIMARY KEY,
    source_name TEXT NOT NULL,
    source_type TEXT NOT NULL
        CHECK(source_type IN ('checking','savings','credit_card','loan','mortgage','cash','investment','other')),
    source_institution_name TEXT,
    source_mask TEXT,
    source_currency TEXT NOT NULL DEFAULT 'USD',
    source_last_known_balance REAL,
    source_balance_as_of_date TEXT,
    source_plaid_account_id TEXT UNIQUE,
    source_plaid_item_id TEXT,
    source_created_datetime TEXT NOT NULL,
    source_archived_datetime TEXT
);

CREATE TABLE IF NOT EXISTS payees (
    payee_id TEXT PRIMARY KEY,
    payee_name TEXT NOT NULL,
    payee_phone TEXT,
    payee_address TEXT,
    payee_website TEXT,
    payee_default_account_id TEXT REFERENCES accounts(account_id) ON DELETE SET NULL,
    payee_created_datetime TEXT NOT NULL
);
CREATE INDEX IF NOT EXISTS idx_payees_name ON payees(payee_name);

CREATE TABLE IF NOT EXISTS recurring_transactions (
    recurring_transaction_id TEXT PRIMARY KEY,
    recurring_transaction_description TEXT NOT NULL,
    recurring_transaction_amount REAL NOT NULL,
    source_id TEXT NOT NULL REFERENCES sources(source_id) ON DELETE RESTRICT,
    account_id TEXT NOT NULL REFERENCES accounts(account_id) ON DELETE RESTRICT,
    payee_id TEXT REFERENCES payees(payee_id) ON DELETE SET NULL,
    recurring_transaction_frequency TEXT NOT NULL CHECK(recurring_transaction_frequency IN ('daily','weekly','biweekly','monthly','quarterly','annually','custom')),
    recurring_transaction_next_date TEXT NOT NULL,
    recurring_transaction_end_date TEXT,
    recurring_transaction_is_active INTEGER NOT NULL DEFAULT 1,
    recurring_transaction_created_datetime TEXT NOT NULL
);
CREATE INDEX IF NOT EXISTS idx_recurring_transaction_source ON recurring_transactions(source_id);
CREATE INDEX IF NOT EXISTS idx_recurring_transaction_account ON recurring_transactions(account_id);

CREATE TABLE IF NOT EXISTS transactions (
    transaction_id TEXT PRIMARY KEY,
    source_id TEXT NOT NULL REFERENCES sources(source_id) ON DELETE RESTRICT,
    recurring_transaction_id TEXT REFERENCES recurring_transactions(recurring_transaction_id) ON DELETE SET NULL,
    transaction_date TEXT NOT NULL,
    transaction_amount REAL NOT NULL,
    transaction_memo TEXT NOT NULL,
    payee_id TEXT REFERENCES payees(payee_id) ON DELETE SET NULL,
    transaction_recording_status TEXT NOT NULL DEFAULT 'not_recorded'
        CHECK(transaction_recording_status IN ('not_recorded','partially_recorded','recorded','excluded')),
    plaid_transaction_id TEXT UNIQUE,
    transaction_details TEXT,
    transaction_recorded_datetime TEXT NOT NULL
);
CREATE INDEX IF NOT EXISTS idx_transactions_source ON transactions(source_id);
CREATE INDEX IF NOT EXISTS idx_transactions_date ON transactions(transaction_date);
CREATE INDEX IF NOT EXISTS idx_transactions_status ON transactions(transaction_recording_status);
CREATE INDEX IF NOT EXISTS idx_transactions_payee ON transactions(payee_id);
CREATE INDEX IF NOT EXISTS idx_transactions_recurring ON transactions(recurring_transaction_id);

CREATE TABLE IF NOT EXISTS transaction_splits (
    transaction_split_id TEXT PRIMARY KEY,
    transaction_id TEXT NOT NULL REFERENCES transactions(transaction_id) ON DELETE CASCADE,
    account_id TEXT NOT NULL REFERENCES accounts(account_id) ON DELETE RESTRICT,
    transaction_split_amount REAL NOT NULL,
    transaction_split_notes TEXT,
    transaction_split_recorded_datetime TEXT NOT NULL
);
CREATE INDEX IF NOT EXISTS idx_lines_transaction ON transaction_splits(transaction_id);
CREATE INDEX IF NOT EXISTS idx_lines_account ON transaction_splits(account_id);

CREATE TABLE IF NOT EXISTS tags (
    tag_id TEXT PRIMARY KEY,
    tag_name TEXT NOT NULL UNIQUE,
    tag_color TEXT,
    tag_created_datetime TEXT NOT NULL
);
CREATE TABLE IF NOT EXISTS transaction_tags (
    transaction_id TEXT NOT NULL REFERENCES transactions(transaction_id) ON DELETE CASCADE,
    tag_id TEXT NOT NULL REFERENCES tags(tag_id) ON DELETE CASCADE,
    PRIMARY KEY (transaction_id, tag_id)
);
CREATE INDEX IF NOT EXISTS idx_transaction_tags_tag ON transaction_tags(tag_id);

CREATE TABLE IF NOT EXISTS budgets (
    budget_id TEXT PRIMARY KEY,
    account_id TEXT NOT NULL REFERENCES accounts(account_id) ON DELETE RESTRICT,
    budget_period_start_date TEXT NOT NULL,
    budget_period_end_date TEXT NOT NULL,
    budget_amount REAL NOT NULL,
    budget_created_datetime TEXT NOT NULL,
    UNIQUE(account_id, budget_period_start_date)
);
CREATE INDEX IF NOT EXISTS idx_budgets_account ON budgets(account_id);
CREATE INDEX IF NOT EXISTS idx_budgets_period ON budgets(budget_period_start_date, budget_period_end_date);

CREATE TABLE IF NOT EXISTS goals (
    goal_id TEXT PRIMARY KEY,
    goal_name TEXT NOT NULL,
    goal_type TEXT NOT NULL CHECK(goal_type in ('savings','debt_payoff','retirement','custom')),
    account_id TEXT NOT NULL REFERENCES accounts(account_id) ON DELETE RESTRICT,
    goal_target_amount REAL NOT NULL,
    goal_current_amount REAL NOT NULL DEFAULT 0,
    goal_target_date TEXT,
    goal_priority INTEGER NOT NULL DEFAULT 0,
    goal_notes TEXT,
    goal_created_datetime TEXT NOT NULL,
    goal_archived_datetime TEXT
);
CREATE INDEX IF NOT EXISTS idx_goals_account ON goals(account_id);
CREATE INDEX IF NOT EXISTS idx_goals_priority ON goals(goal_priority);

CREATE TABLE IF NOT EXISTS goal_conflicts (
    goal_conflict_id TEXT PRIMARY KEY,
    goal_id_a TEXT NOT NULL REFERENCES goals(goal_id) ON DELETE CASCADE,
    goal_id_b TEXT NOT NULL REFERENCES goals(goal_id) ON DELETE CASCADE,
    goal_conflict_detected_datetime TEXT NOT NULL,
    goal_conflict_resolution_status TEXT NOT NULL DEFAULT 'unresolved' CHECK(goal_conflict_resolution_status IN('unresolved','prioritized_a','prioritized_b','deferred_both')),
    goal_conflict_resolved_datetime TEXT
);
CREATE INDEX IF NOT EXISTS idx_conflicts_goal_a ON goal_conflicts(goal_id_a);
CREATE INDEX IF NOT EXISTS idx_conflicts_goal_b ON goal_conflicts(goal_id_b);

CREATE TABLE IF NOT EXISTS assumption_sets (
    assumption_set_id TEXT PRIMARY KEY,
    assumption_set_expected_return_rate REAL NOT NULL DEFAULT 0.07,
    assumption_set_inflation_rate REAL NOT NULL DEFAULT 0.04,
    assumption_set_income_growth_rate REAL NOT NULL DEFAULT 0.02,
    assumption_set_retirement_age INTEGER,
    assumption_set_notes TEXT,
    assumption_set_created_datetime TEXT NOT NULL,
    assumption_set_is_active INTEGER NOT NULL DEFAULT 1
);

CREATE TABLE IF NOT EXISTS life_events (
    life_event_id TEXT PRIMARY KEY,
    life_event_name TEXT NOT NULL,
    life_event_date TEXT NOT NULL,
    life_event_type TEXT NOT NULL CHECK(life_event_type IN('one_time_expense','one_time_income','recurring_change')),
    life_event_amount REAL NOT NULL,
    life_event_monthly_affect REAL,
    account_id TEXT REFERENCES accounts(account_id) ON DELETE SET NULL,
    life_event_notes TEXT,
    life_event_created_datetime TEXT NOT NULL
);
CREATE INDEX IF NOT EXISTS idx_life_event_date ON life_events(life_event_date);

CREATE TABLE IF NOT EXISTS exchanges (
    exchange_id TEXT PRIMARY KEY,
    exchange_from_currency TEXT NOT NULL,
    exchange_to_currency TEXT NOT NULL,
    exchange_rate REAL NOT NULL,
    exchange_effective_date TEXT NOT NULL,
    exchange_created_datetime TEXT NOT NULL,
    UNIQUE(exchange_from_currency, exchange_to_currency, exchange_effective_date)
);
CREATE INDEX IF NOT EXISTS idx_exchanges_pair ON exchanges(exchange_from_currency, exchange_to_currency);
CREATE INDEX IF NOT EXISTS idx_exchanges_date ON exchanges(exchange_effective_date);

CREATE TABLE IF NOT EXISTS sync_state (
    device_id TEXT PRIMARY KEY,
    device_last_synced_datetime TEXT,
    sync_token TEXT
);
