use anyhow::{bail, Context, Result};
use chrono::{DateTime, Duration as ChronoDuration, Utc};
mod db;

use db::{connect_store, NotificationStore};
use octocrab::models::activity::Notification as GitHubNotification;
use octocrab::Octocrab;
use reqwest::Client;
use serde_json::json;
use std::cmp::Ordering;
use std::env;
use std::time::Duration;

#[derive(Debug, Clone)]
struct Config {
  github_token: String,
  telegram_bot_token: String,
  telegram_chat_id: String,
  poll_interval: Duration,
  http_timeout: Duration,
  database_url: String,
}

#[tokio::main(flavor = "multi_thread")]
async fn main() -> Result<()> {
  rustls::crypto::aws_lc_rs::default_provider()
    .install_default()
    .map_err(|_| anyhow::anyhow!("install rustls aws-lc-rs provider"))?;

  let cfg = load_config()?;

  let http = Client::builder()
    .timeout(cfg.http_timeout)
    .build()
    .context("build http client")?;

  let octocrab = Octocrab::builder()
    .personal_token(cfg.github_token.clone())
    .build()
    .context("build octocrab client")?;

  let store = connect_store(&cfg.database_url).await?;
  store.init().await?;

  println!(
    "daemon started, poll_interval={}s, database_url={}",
    cfg.poll_interval.as_secs(),
    cfg.database_url
  );

  let mut since_cursor: Option<DateTime<Utc>> = None;

  loop {
    tokio::select! {
      signal = tokio::signal::ctrl_c() => {
        signal.context("listen for ctrl-c")?;
        break;
      }
      result = poll_once(&cfg, &http, &octocrab, store.as_ref(), since_cursor) => {
        match result {
          Ok(latest_seen) => {
            if let Some(ts) = latest_seen {
              since_cursor = Some(ts - ChronoDuration::seconds(1));
            }
          }
          Err(err) => {
            eprintln!("poll failed: {err:#}");
          }
        }
      }
    }

    tokio::select! {
      signal = tokio::signal::ctrl_c() => {
        signal.context("listen for ctrl-c")?;
        break;
      }
      _ = tokio::time::sleep(cfg.poll_interval) => {}
    }
  }

  println!("daemon stopped");
  Ok(())
}

fn load_config() -> Result<Config> {
  let github_token = required_env("GITHUB_TOKEN")?;
  let telegram_bot_token = required_env("TELEGRAM_BOT_TOKEN")?;
  let telegram_chat_id = required_env("TELEGRAM_CHAT_ID")?;

  let poll_interval_secs = parse_u64_env_or_default("POLL_INTERVAL_SECONDS", 60)?;
  let http_timeout_secs = parse_u64_env_or_default("HTTP_TIMEOUT_SECONDS", 15)?;
  let database_url = env_or_default("DATABASE_URL", "sqlite://./data/notify.db");

  if poll_interval_secs == 0 {
    bail!("POLL_INTERVAL_SECONDS must be > 0");
  }
  if http_timeout_secs == 0 {
    bail!("HTTP_TIMEOUT_SECONDS must be > 0");
  }

  Ok(Config {
    github_token,
    telegram_bot_token,
    telegram_chat_id,
    poll_interval: Duration::from_secs(poll_interval_secs),
    http_timeout: Duration::from_secs(http_timeout_secs),
    database_url,
  })
}

async fn poll_once(
  cfg: &Config,
  http: &Client,
  octocrab: &Octocrab,
  store: &dyn NotificationStore,
  since: Option<DateTime<Utc>>,
) -> Result<Option<DateTime<Utc>>> {
  let mut notifications = fetch_notifications(octocrab, since)
    .await
    .context("fetch notifications from github")?;

  let latest_seen = notifications.iter().map(|n| n.updated_at).max();

  notifications.sort_by(|a, b| {
    if a.updated_at == b.updated_at {
      Ordering::Equal
    } else if a.updated_at < b.updated_at {
      Ordering::Less
    } else {
      Ordering::Greater
    }
  });

  let mut sent_count = 0u32;

  for notification in notifications {
    let notification_id = notification.id.to_string();

    if !notification.unread {
      continue;
    }

    if store.is_sent(&notification_id).await? {
      continue;
    }

    let message = format_message(&notification);
    if let Err(err) = send_telegram(cfg, http, &message).await {
      eprintln!("telegram send failed for {notification_id}: {err:#}");
      continue;
    }

    store.mark_sent(&notification_id).await?;
    sent_count += 1;
  }

  if sent_count > 0 {
    println!("forwarded {sent_count} notification(s)");
  }

  Ok(latest_seen)
}

async fn fetch_notifications(
  octocrab: &Octocrab,
  since: Option<DateTime<Utc>>,
) -> Result<Vec<GitHubNotification>> {
  let mut all = Vec::new();
  let mut page = 1u8;

  loop {
    let mut req = octocrab
      .activity()
      .notifications()
      .list()
      .all(false)
      .participating(false)
      .per_page(50u8)
      .page(page);

    if let Some(since) = since {
      req = req.since(since);
    }

    let page_items = req
      .send()
      .await
      .with_context(|| format!("request github notifications page {page}"))?;

    let item_count = page_items.items.len();
    all.extend(page_items.items);

    if item_count < 50 || page == u8::MAX {
      break;
    }

    page += 1;
  }

  Ok(all)
}

async fn send_telegram(cfg: &Config, http: &Client, message: &str) -> Result<()> {
  let url = format!(
    "https://api.telegram.org/bot{}/sendMessage",
    cfg.telegram_bot_token
  );

  let payload = json!({
      "chat_id": cfg.telegram_chat_id,
      "text": message,
      "disable_web_page_preview": true
  });

  let resp = http
    .post(url)
    .json(&payload)
    .send()
    .await
    .context("request telegram sendMessage")?;

  if !resp.status().is_success() {
    let status = resp.status();
    let body = resp
      .text()
      .await
      .unwrap_or_else(|_| "<failed to read body>".to_string());
    bail!("telegram send status={status} body={body}");
  }

  Ok(())
}

fn format_message(n: &GitHubNotification) -> String {
  let repo_name = n
    .repository
    .full_name
    .as_deref()
    .unwrap_or("unknown/unknown");

  format!(
      "🔔 GitHub Notification\nRepo: {}\nType: {}\nReason: {}\nTitle: {}\nUpdated: {}\nThread: https://github.com/notifications/threads/{}\nInbox: https://github.com/notifications",
        repo_name,
        n.subject.r#type,
        n.reason,
        n.subject.title,
      n.updated_at.to_rfc3339(),
      n.id
    )
}

fn required_env(name: &str) -> Result<String> {
  let value = env::var(name).unwrap_or_default().trim().to_string();
  if value.is_empty() {
    bail!("missing {name}");
  }
  Ok(value)
}

fn env_or_default(name: &str, default_value: &str) -> String {
  let value = env::var(name).unwrap_or_default().trim().to_string();
  if value.is_empty() {
    default_value.to_string()
  } else {
    value
  }
}

fn parse_u64_env_or_default(name: &str, default_value: u64) -> Result<u64> {
  let raw = env::var(name).unwrap_or_else(|_| default_value.to_string());
  raw
    .trim()
    .parse::<u64>()
    .with_context(|| format!("invalid {name}: {raw}"))
}
