# github-notify-to-tg

A Rust daemon that polls GitHub notifications and forwards new unread items to a Telegram chat via bot API.

## Features

- Polls GitHub Notifications API on a fixed interval
- Uses octocrab SDK for GitHub API access and typed models
- Sends unread notifications to Telegram
- Uses SQLx storage layer (SQLite now, Postgres-ready) to persist sent notification IDs and avoid duplicate sends
- Never marks GitHub notifications as read
- Runs in Docker with restart policy

## Required credentials

- GitHub personal access token:
  - Classic token: `notifications` scope
  - Fine-grained token: read access to notifications
- Telegram bot token (from [@BotFather](https://t.me/BotFather))
- Telegram chat ID (private/user/group)

## Configuration

Copy and edit environment file:

```bash
cp .env.example .env
```

Required:

- `GITHUB_TOKEN`
- `TELEGRAM_BOT_TOKEN`
- `TELEGRAM_CHAT_ID`

Optional:

- `POLL_INTERVAL_SECONDS` (default: `60`)
- `HTTP_TIMEOUT_SECONDS` (default: `15`)
- `GITHUB_API_BASE` (default: `https://api.github.com`)
- `DATABASE_URL` (default: `sqlite://./data/notify.db`)
  - SQLite example: `sqlite://./data/notify.db`
  - Postgres example: `postgres://user:password@localhost:5432/github_notify`

## Run with Docker Compose

```bash
docker compose up -d --build
```

View logs:

```bash
docker compose logs -f
```

Stop:

```bash
docker compose down
```

For SQLite, database file is stored at `./data/notify.db` on the host.

## Local run (without Docker)

```bash
cargo run --release
```

## Notes

- A notification is considered already forwarded when its GitHub thread ID exists in SQLite.
- If Telegram send fails, that notification ID is not recorded and will be retried in the next poll.
