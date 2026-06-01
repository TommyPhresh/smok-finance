\# Smok Finance



> Run your personal financial life like a CEO.



A local-first personal finance application with optional encrypted syncing.



\## Structure

```

/

|--- Cargo.toml    

|--- migrations/    # SQLx migrations run by core::db::migrate

|--- crates/

|    | --- core/    # DB, entity struct defs, errors

|    | --- api/     # Axum REST server

|    | --- sync/    # Sync server

|--- web/           # React frontend

|--- mobile/        # Flutter frontend

```



\## Getting Started

```bash

cp .env.example .env

cargo build

cargo run -p api

```



\## Pricing

\- \*\*Free\*\*: Local app with full features. One device, no account needed

\- \*\*Sync\*\*: One-time payment for full access to encrypted cross-device sync via self-hostable sync server

\- \*\*Power User\*\*: One-time payment for early access to beta features



\## Stack

Backend => Rust (Axum)

DB => SQLite (sqlx)

Web Frontend => React

Mobile Frontend => Flutter

Sync Server => Rust (Axum)





