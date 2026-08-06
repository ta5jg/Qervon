// =============================================================================
// File:           backend/apps/migration-runner/src/main.rs
// Project:        Qervon
// Author:         USDTG GROUP TECHNOLOGY LLC
// Developer:      Irfan Gedik
// Created Date:   2026-08-05
// Version:        0.1.0
//
// Description:
//   Applies governed SQL migrations under backend/migrations in order.
//
// Specification:
//   QAS-000006, QES-000002, QES-000006.
//
// License:
//   Qervon License v1.0 — see LICENSE in the repository root.
// =============================================================================

use std::path::{Path, PathBuf};

use qervon_infrastructure::postgres::PgPoolOptions;
use sqlx::PgPool;

const BOOKKEEPING: &str = "public._qervon_migrations";

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "qervon_migration_runner=info".into()),
        )
        .init();

    let database_url = std::env::var("DATABASE_URL").map_err(|_| "DATABASE_URL is required")?;
    let migrations_dir = std::env::var("MIGRATIONS_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../migrations"));

    let pool = PgPoolOptions::new().connect(&database_url).await?;
    let applied = run_migrations(&pool, Path::new(&migrations_dir)).await?;
    tracing::info!(applied, "migration run complete");
    Ok(())
}

async fn run_migrations(pool: &PgPool, dir: &Path) -> Result<usize, sqlx::Error> {
    sqlx::query(&format!(
        "CREATE TABLE IF NOT EXISTS {BOOKKEEPING} (file text PRIMARY KEY, applied_at timestamptz NOT NULL DEFAULT now())"
    ))
    .execute(pool)
    .await?;

    let mut files = collect_sql_files(dir)?;
    files.sort();

    let mut applied = 0;
    for file in files {
        let key = file.to_string_lossy().to_string();
        let already_applied = sqlx::query_scalar::<_, bool>(&format!(
            "SELECT EXISTS(SELECT 1 FROM {BOOKKEEPING} WHERE file = $1)"
        ))
        .bind(&key)
        .fetch_one(pool)
        .await?;
        if already_applied {
            continue;
        }

        let sql = std::fs::read_to_string(&file)?;
        if is_effectively_empty(&sql) {
            continue;
        }

        let mut tx = pool.begin().await?;
        sqlx::raw_sql(&sql).execute(&mut *tx).await?;
        sqlx::query(&format!("INSERT INTO {BOOKKEEPING} (file) VALUES ($1)"))
            .bind(&key)
            .execute(&mut *tx)
            .await?;
        tx.commit().await?;
        applied += 1;
        tracing::info!(file = %file.to_string_lossy(), "applied migration");
    }
    Ok(applied)
}

fn collect_sql_files(dir: &Path) -> Result<Vec<PathBuf>, std::io::Error> {
    let mut files = Vec::new();
    for entry in std::fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            files.extend(collect_sql_files(&path)?);
        } else if path.extension().is_some_and(|ext| ext == "sql") {
            files.push(path);
        }
    }
    Ok(files)
}

fn is_effectively_empty(sql: &str) -> bool {
    sql.lines()
        .all(|line| line.trim().is_empty() || line.trim_start().starts_with("--"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_comment_only_migration() {
        assert!(is_effectively_empty("-- hello\n-- world\n"));
        assert!(!is_effectively_empty("-- hello\nCREATE TABLE x (id int);"));
        assert!(is_effectively_empty("\n\n  \n"));
    }
}
