use anyhow::{bail, Context, Result};
use async_trait::async_trait;
use sqlx::postgres::PgPoolOptions;
use sqlx::sqlite::SqlitePoolOptions;
use sqlx::{PgPool, SqlitePool};
use std::path::Path;

#[async_trait]
pub trait NotificationStore: Send + Sync {
  async fn init(&self) -> Result<()>;
  async fn is_sent(&self, id: &str) -> Result<bool>;
  async fn mark_sent(&self, id: &str) -> Result<()>;
}

pub struct SqliteStore {
  pool: SqlitePool,
}

pub struct PostgresStore {
  pool: PgPool,
}

pub async fn connect_store(database_url: &str) -> Result<Box<dyn NotificationStore>> {
  if database_url.starts_with("postgres://")
    || database_url.starts_with("postgresql://")
  {
    let pool = PgPoolOptions::new()
      .max_connections(5)
      .connect(database_url)
      .await
      .with_context(|| format!("connect postgres database: {database_url}"))?;
    return Ok(Box::new(PostgresStore { pool }) as Box<dyn NotificationStore>);
  }

  if database_url.starts_with("sqlite://") {
    ensure_sqlite_parent_dir(database_url)?;
    let pool = SqlitePoolOptions::new()
      .max_connections(5)
      .connect(database_url)
      .await
      .with_context(|| format!("connect sqlite database: {database_url}"))?;
    return Ok(Box::new(SqliteStore { pool }) as Box<dyn NotificationStore>);
  }

  bail!("unsupported DATABASE_URL scheme, use sqlite:// or postgres://")
}

#[async_trait]
impl NotificationStore for SqliteStore {
  async fn init(&self) -> Result<()> {
    sqlx::query(
      "CREATE TABLE IF NOT EXISTS sent_notifications (
                id TEXT PRIMARY KEY,
                sent_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
            )",
    )
    .execute(&self.pool)
    .await
    .context("create sent_notifications table in sqlite")?;

    Ok(())
  }

  async fn is_sent(&self, id: &str) -> Result<bool> {
    let exists = sqlx::query_scalar::<_, i64>(
      "SELECT 1 FROM sent_notifications WHERE id = ? LIMIT 1",
    )
    .bind(id)
    .fetch_optional(&self.pool)
    .await
    .context("run sqlite dedupe query")?
    .is_some();

    Ok(exists)
  }

  async fn mark_sent(&self, id: &str) -> Result<()> {
    sqlx::query("INSERT OR IGNORE INTO sent_notifications (id) VALUES (?)")
      .bind(id)
      .execute(&self.pool)
      .await
      .with_context(|| format!("mark notification as sent in sqlite: {id}"))?;

    Ok(())
  }
}

#[async_trait]
impl NotificationStore for PostgresStore {
  async fn init(&self) -> Result<()> {
    sqlx::query(
      "CREATE TABLE IF NOT EXISTS sent_notifications (
                id TEXT PRIMARY KEY,
                sent_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP
            )",
    )
    .execute(&self.pool)
    .await
    .context("create sent_notifications table in postgres")?;

    Ok(())
  }

  async fn is_sent(&self, id: &str) -> Result<bool> {
    let exists = sqlx::query_scalar::<_, i64>(
      "SELECT 1 FROM sent_notifications WHERE id = $1 LIMIT 1",
    )
    .bind(id)
    .fetch_optional(&self.pool)
    .await
    .context("run postgres dedupe query")?
    .is_some();

    Ok(exists)
  }

  async fn mark_sent(&self, id: &str) -> Result<()> {
    sqlx::query(
      "INSERT INTO sent_notifications (id) VALUES ($1) ON CONFLICT (id) DO NOTHING",
    )
    .bind(id)
    .execute(&self.pool)
    .await
    .with_context(|| format!("mark notification as sent in postgres: {id}"))?;

    Ok(())
  }
}

fn ensure_sqlite_parent_dir(database_url: &str) -> Result<()> {
  if database_url == "sqlite::memory:" {
    return Ok(());
  }

  let raw_path = database_url.trim_start_matches("sqlite://");
  if raw_path.is_empty() {
    return Ok(());
  }

  let path = Path::new(raw_path);
  if let Some(parent) = path.parent() {
    if !parent.as_os_str().is_empty() {
      std::fs::create_dir_all(parent)
        .with_context(|| format!("create sqlite dir {}", parent.display()))?;
    }
  }

  Ok(())
}
