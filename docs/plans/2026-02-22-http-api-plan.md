# HTTP API Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Add an Axum HTTP API to the shopping bot, running alongside the Telegram bot in the same process.

**Architecture:** Axum HTTP server spawned as a concurrent Tokio task sharing the existing `ListRepo` via `Arc`. Bearer token auth middleware on all endpoints except the docs endpoint. API only starts if `API_TOKEN` env var is set.

**Tech Stack:** Axum, serde, serde_json, tokio

**Design doc:** `docs/plans/2026-02-22-http-api-design.md`

---

### Task 1: Add dependencies

**Files:**
- Modify: `Cargo.toml`

**Step 1: Add axum, serde, serde_json to Cargo.toml**

Add after the `tokio-rusqlite` line in `[dependencies]`:

```toml
axum = "0.8"
serde = { version = "1", features = ["derive"] }
serde_json = "1"
```

**Step 2: Verify it compiles**

Run: `cargo check`
Expected: compiles with no errors

**Step 3: Commit**

```bash
git add Cargo.toml Cargo.lock
git commit -m "add axum, serde, serde_json dependencies"
```

---

### Task 2: Add `add_item_returning_id` to ListRepo

The existing `add_item` returns `Result<(), String>`. The API needs to return the created item's ID. Add a new method rather than changing the existing one (the Telegram bot doesn't need the ID).

**Files:**
- Modify: `src/persistence_sqlite.rs` (after line 54, after `add_item`)

**Step 1: Add the method**

Add this method after the closing brace of `add_item` (after line 54):

```rust
    pub async fn add_item_returning_id(&self, name: &str) -> Result<ListItem, String> {
        if name.is_empty() {
            return Err("Name cannot be empty".to_string());
        }

        if name.len() > 100 {
            return Err("Name is too long (max 100 characters)".to_string());
        }

        let name = name.to_string();
        self.conn
            .call(move |conn| {
                conn.execute(
                    "INSERT INTO list_items (name) VALUES (?1)",
                    params![&name],
                )?;
                let id = conn.last_insert_rowid() as u64;
                Ok(ListItem { id, name })
            })
            .await
            .map_err(|e| format!("Failed to add item: {}", e))
    }
```

**Step 2: Verify it compiles**

Run: `cargo check`
Expected: compiles with no errors

**Step 3: Commit**

```bash
git add src/persistence_sqlite.rs
git commit -m "add add_item_returning_id to ListRepo"
```

---

### Task 3: Extend Config with API settings

**Files:**
- Modify: `src/config.rs`

**Step 1: Add `api_token` and `api_port` fields to Config struct**

Change the `Config` struct (lines 3-6) to:

```rust
pub struct Config {
    token: String,
    allowed_chat_ids: Vec<i64>,
    api_token: Option<String>,
    api_port: u16,
}
```

**Step 2: Load the new fields in `Config::new()`**

Add these lines inside `Config::new()`, before the `Ok(Config {` block (before line 22):

```rust
        let api_token = env::var("API_TOKEN").ok();

        let api_port = env::var("API_PORT")
            .ok()
            .and_then(|p| p.parse().ok())
            .unwrap_or(8080);
```

And update the `Ok(Config { ... })` block to include the new fields:

```rust
        Ok(Config {
            token,
            allowed_chat_ids,
            api_token,
            api_port,
        })
```

**Step 3: Add accessor methods**

Add after `is_chat_allowed` (after line 39):

```rust
    pub fn api_token(&self) -> Option<&str> {
        self.api_token.as_deref()
    }

    pub fn api_port(&self) -> u16 {
        self.api_port
    }
```

**Step 4: Verify it compiles**

Run: `cargo check`
Expected: compiles with no errors

**Step 5: Commit**

```bash
git add src/config.rs
git commit -m "add API_TOKEN and API_PORT to config"
```

---

### Task 4: Create the API module

This is the main task. Create `src/api.rs` with all routes, auth, and the docs endpoint.

**Files:**
- Create: `src/api.rs`

**Step 1: Write the full API module**

Create `src/api.rs` with this content:

```rust
use axum::{
    Router,
    Json,
    extract::{Path, State},
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Response},
    routing::{get, post, delete},
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use crate::persistence_sqlite::ListRepo;

pub struct ApiState {
    pub db: Arc<ListRepo>,
    pub api_token: String,
}

#[derive(Serialize)]
struct ItemResponse {
    id: u64,
    name: String,
}

#[derive(Serialize)]
struct ListResponse {
    items: Vec<ItemResponse>,
}

#[derive(Deserialize)]
struct AddItemRequest {
    name: String,
}

#[derive(Serialize)]
struct ErrorResponse {
    error: String,
}

fn check_auth(headers: &HeaderMap, expected_token: &str) -> Result<(), Response> {
    let auth = headers
        .get("authorization")
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.strip_prefix("Bearer "));

    match auth {
        Some(token) if token == expected_token => Ok(()),
        _ => Err((
            StatusCode::UNAUTHORIZED,
            Json(ErrorResponse { error: "unauthorized".to_string() }),
        ).into_response()),
    }
}

pub fn router(state: Arc<ApiState>) -> Router {
    Router::new()
        .route("/", get(docs))
        .route("/items", get(list_items).post(add_item).delete(clear_items))
        .route("/items/{id}", delete(remove_item))
        .with_state(state)
}

async fn docs() -> impl IntoResponse {
    (
        StatusCode::OK,
        [("content-type", "text/plain")],
        "Shopping List API

Authentication: Bearer token in Authorization header (all endpoints except this one)

Endpoints:
  GET    /items      - List all items. Returns: {\"items\": [{\"id\": number, \"name\": string}]}
  POST   /items      - Add item. Body: {\"name\": string} (max 100 chars). Returns: {\"id\": number, \"name\": string}
  DELETE /items/{id}  - Remove item by ID. Returns: 204 No Content
  DELETE /items      - Clear all items. Returns: 204 No Content

Error responses: {\"error\": string} with appropriate HTTP status code
",
    )
}

async fn list_items(
    headers: HeaderMap,
    State(state): State<Arc<ApiState>>,
) -> Result<impl IntoResponse, Response> {
    check_auth(&headers, &state.api_token)?;

    match state.db.list().await {
        Ok(items) => Ok(Json(ListResponse {
            items: items.into_iter().map(|i| ItemResponse { id: i.id, name: i.name }).collect(),
        })),
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse { error: e }),
        ).into_response()),
    }
}

async fn add_item(
    headers: HeaderMap,
    State(state): State<Arc<ApiState>>,
    Json(body): Json<AddItemRequest>,
) -> Result<Response, Response> {
    check_auth(&headers, &state.api_token)?;

    match state.db.add_item_returning_id(&body.name).await {
        Ok(item) => Ok((
            StatusCode::CREATED,
            Json(ItemResponse { id: item.id, name: item.name }),
        ).into_response()),
        Err(e) => Err((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse { error: e }),
        ).into_response()),
    }
}

async fn remove_item(
    headers: HeaderMap,
    State(state): State<Arc<ApiState>>,
    Path(id): Path<u64>,
) -> Result<Response, Response> {
    check_auth(&headers, &state.api_token)?;

    match state.db.remove_item(id).await {
        Ok(()) => Ok(StatusCode::NO_CONTENT.into_response()),
        Err(e) => Err((
            StatusCode::NOT_FOUND,
            Json(ErrorResponse { error: e }),
        ).into_response()),
    }
}

async fn clear_items(
    headers: HeaderMap,
    State(state): State<Arc<ApiState>>,
) -> Result<Response, Response> {
    check_auth(&headers, &state.api_token)?;

    match state.db.clear().await {
        Ok(()) => Ok(StatusCode::NO_CONTENT.into_response()),
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse { error: e }),
        ).into_response()),
    }
}
```

**Step 2: Verify it compiles**

Run: `cargo check`
Expected: warning about unused module is fine (we haven't wired it into main yet), but no errors

Actually it won't compile yet because it's not declared as a module. Move to Task 5 before checking.

**Step 3: Commit (after Task 5)**

Combined with Task 5.

---

### Task 5: Wire API into main.rs

**Files:**
- Modify: `src/main.rs`

**Step 1: Add the `mod api;` declaration**

Add `mod api;` after line 3 (`mod persistence_sqlite;`).

**Step 2: Restructure main to share ListRepo and spawn API**

The current `main.rs` creates a `CommandProcessor` which owns the `ListRepo`. We need to extract the `ListRepo` creation so it can be shared. Refactor `CommandProcessor::new` to accept an `Arc<ListRepo>` instead of creating one internally, OR extract the DB from the processor.

Simpler approach: extract the DB path logic and `ListRepo` creation into `main`, pass `Arc<ListRepo>` to `CommandProcessor::new`.

Replace the entire `main.rs` with:

```rust
mod config;
mod command;
mod persistence_sqlite;
mod api;

use std::sync::Arc;
use teloxide::prelude::*;

#[tokio::main]
async fn main() {
    pretty_env_logger::init();

    let config = match config::Config::new() {
        Ok(config) => config,
        Err(err) => panic!("{}", err),
    };

    let db_path = if std::path::Path::new("/data").exists() {
        "/data/shopping_list.db"
    } else {
        "/tmp/shopping_list.db"
    };

    let db = Arc::new(
        persistence_sqlite::ListRepo::new(db_path)
            .await
            .expect("Failed to initialize database"),
    );

    if let Some(api_token) = config.api_token() {
        let api_state = Arc::new(api::ApiState {
            db: db.clone(),
            api_token: api_token.to_string(),
        });
        let port = config.api_port();
        let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{}", port))
            .await
            .expect("Failed to bind API port");
        log::info!("API server listening on port {}", port);
        tokio::spawn(async move {
            axum::serve(listener, api::router(api_state)).await.unwrap();
        });
    }

    let bot = teloxide::Bot::new(config.bot_token());

    let me = bot.get_me().send().await.unwrap();
    log::info!("Bot starting as {:?}", me);

    let command_processor = command::CommandProcessor::new(config, db).await;

    command::Command::repl(bot, move |bot, msg, cmd| {
        let processor = command_processor.clone();
        command::CommandProcessor::answer(bot, msg, cmd, processor)
    }).await;
}
```

**Step 3: Update CommandProcessor::new to accept shared ListRepo**

In `src/command.rs`, change `CommandProcessor::new` (lines 29-45) to accept a pre-created `Arc<ListRepo>`:

```rust
    pub async fn new(config: Config, db: Arc<persistence_sqlite::ListRepo>) -> Arc<Self> {
        Arc::new(Self {
            db,
            config: Arc::new(config),
        })
    }
```

This replaces the DB path logic and ListRepo creation that was previously inside `CommandProcessor::new`.

**Step 4: Verify it compiles**

Run: `cargo check`
Expected: compiles with no errors

**Step 5: Commit**

```bash
git add src/api.rs src/main.rs src/command.rs
git commit -m "add HTTP API server alongside Telegram bot"
```

---

### Task 6: Update Docker and docker-compose

**Files:**
- Modify: `docker-compose.yml`
- Modify: `Dockerfile`

**Step 1: Expose API port in Dockerfile**

Add before the `CMD` line in `Dockerfile` (before line 42):

```dockerfile
EXPOSE 8080
```

**Step 2: Add API env vars to docker-compose.yml**

Add these lines after the `ALLOWED_CHAT_IDS` line (after line 16) in `docker-compose.yml`:

```yaml
      # Optional: API token to enable HTTP API
      - API_TOKEN=${API_TOKEN:-}

      # Optional: API port (default 8080)
      - API_PORT=${API_PORT:-8080}
```

And add a ports mapping. Add after the `env_file` block (after line 9):

```yaml
    ports:
      - "${API_PORT:-8080}:${API_PORT:-8080}"
```

**Step 3: Commit**

```bash
git add Dockerfile docker-compose.yml
git commit -m "expose API port in Docker config"
```

---

### Task 7: Manual smoke test

**Step 1: Run the bot locally with API enabled**

```bash
API_TOKEN=test-secret API_PORT=8080 BOT_TOKEN=fake RUST_LOG=info cargo run
```

(The bot will fail to connect to Telegram with a fake token, but the API should bind before that. Alternatively, use a real BOT_TOKEN if available.)

**Step 2: Test the docs endpoint (no auth)**

```bash
curl http://localhost:8080/
```

Expected: plain text API documentation

**Step 3: Test auth rejection**

```bash
curl -s http://localhost:8080/items
```

Expected: `{"error":"unauthorized"}` with 401 status

**Step 4: Test adding an item**

```bash
curl -s -X POST http://localhost:8080/items \
  -H "Authorization: Bearer test-secret" \
  -H "Content-Type: application/json" \
  -d '{"name":"milk"}'
```

Expected: `{"id":1,"name":"milk"}` with 201 status

**Step 5: Test listing items**

```bash
curl -s http://localhost:8080/items \
  -H "Authorization: Bearer test-secret"
```

Expected: `{"items":[{"id":1,"name":"milk"}]}`

**Step 6: Test removing an item**

```bash
curl -s -X DELETE http://localhost:8080/items/1 \
  -H "Authorization: Bearer test-secret"
```

Expected: 204 No Content

**Step 7: Test clearing**

```bash
curl -s -X DELETE http://localhost:8080/items \
  -H "Authorization: Bearer test-secret"
```

Expected: 204 No Content
