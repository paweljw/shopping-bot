# HTTP API Design

## Summary

Add an Axum HTTP API running alongside the Telegram bot in the same process. The API exposes all shopping list operations with Bearer token authentication and an unauthenticated LLM-friendly docs endpoint.

## Architecture

- Axum HTTP server runs as a concurrent Tokio task alongside the Telegram bot REPL
- Shares `ListRepo` (already `Arc`-wrapped) between bot and API via Axum `State`
- API is optional: only starts if `API_TOKEN` env var is set

## Environment Variables

| Variable | Required | Default | Description |
|----------|----------|---------|-------------|
| `API_TOKEN` | Yes (to enable API) | — | Preshared bearer token for authentication |
| `API_PORT` | No | `8080` | Port for the HTTP server |

## Endpoints

| Method | Path | Auth | Body | Response |
|--------|------|------|------|----------|
| `GET` | `/` | No | — | Plain-text API documentation |
| `GET` | `/items` | Yes | — | `{"items": [{"id": number, "name": string}]}` |
| `POST` | `/items` | Yes | `{"name": string}` | `{"id": number, "name": string}` (201) |
| `DELETE` | `/items/:id` | Yes | — | 204 No Content |
| `DELETE` | `/items` | Yes | — | 204 No Content |

## Authentication

- `Authorization: Bearer <API_TOKEN>` header required on all endpoints except `GET /`
- Returns 401 with `{"error": "unauthorized"}` on missing/invalid token
- Docs endpoint is unauthenticated so agents can discover capabilities

## Error Responses

JSON `{"error": "message"}` with HTTP status codes:
- 400: invalid input (empty name, name too long)
- 401: missing or invalid bearer token
- 404: item not found
- 500: internal server error

## New Dependencies

- `axum` — HTTP framework (Tokio-native)
- `serde` + `serde_json` — JSON serialization

## File Changes

| File | Change |
|------|--------|
| `Cargo.toml` | Add axum, serde, serde_json |
| `src/config.rs` | Add `api_token()` and `api_port()` methods |
| `src/api.rs` | New module: routes, auth extractor, handlers, docs endpoint |
| `src/main.rs` | Spawn API server as concurrent Tokio task, share `ListRepo` |
| `src/persistence_sqlite.rs` | Add `add_item_returning_id` method |

## Docs Endpoint (`GET /`)

Returns plain text:

```
Shopping List API

Authentication: Bearer token in Authorization header (all endpoints except this one)

Endpoints:
  GET    /items      - List all items. Returns: {"items": [{"id": number, "name": string}]}
  POST   /items      - Add item. Body: {"name": string} (max 100 chars). Returns: {"id": number, "name": string}
  DELETE /items/:id  - Remove item by ID. Returns: 204 No Content
  DELETE /items      - Clear all items. Returns: 204 No Content

Error responses: {"error": string} with appropriate HTTP status code
```
