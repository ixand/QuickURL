# QuickURL

A URL shortener REST API written in Rust. Paste a long URL, get a 6-character token back, redirect from anywhere — or let it expire automatically.

**Stack:** Axum 0.7 · Tokio async runtime · SQLite via sqlx · single binary, no runtime dependencies

---

## Quick start

```bash
git clone https://github.com/ixand/QuickURL.git
cd QuickURL
cargo run
```

The server starts on `http://0.0.0.0:3000`. On first run it creates `quickurl.db` and applies migrations automatically — no setup required.

---

## API

### POST /shorten — Create a short URL

```json
{
  "url":        "https://example.com/very/long/path",
  "title":      "Optional label",
  "expires_at": "2025-12-31T23:59:59Z"
}
```

`url` is required and must start with `http://` or `https://`. `title` and `expires_at` are optional — if `expires_at` is omitted, the link expires in 30 days.

**Response `201 Created`**

```json
{
  "id":           "uuid-v4",
  "token":        "aB3xYz",
  "original_url": "https://example.com/very/long/path",
  "short_url":    "http://localhost:3000/aB3xYz",
  "title":        null,
  "created_at":   "2025-01-01T00:00:00Z",
  "expires_at":   "2025-01-31T00:00:00Z",
  "click_count":  0
}
```

---

### GET /:token — Redirect

Returns `308 Permanent Redirect` to the original URL and increments `click_count`. Returns `410 Gone` if the link has expired.

```bash
curl -L http://localhost:3000/aB3xYz
```

---

### GET /urls — List all URLs

Returns all stored URLs ordered by `created_at` descending.

```json
{ "urls": [ /* UrlInfo[] */ ] }
```

---

### GET /urls/:token — Get URL info

Returns the full `UrlInfo` object for a token. `404` if it doesn't exist.

---

### DELETE /urls/:token — Delete a URL

Returns `204 No Content` on success. `404` if the token doesn't exist.

---

### GET /health — Health check

```json
{ "status": "healthy", "service": "QuickURL", "version": "0.1.0" }
```

---

## Schema

| Column | Type | Notes |
|---|---|---|
| `id` | TEXT | UUID v4, primary key |
| `token` | TEXT UNIQUE | 6-char alphanumeric `[A-Za-z0-9]`, indexed |
| `original_url` | TEXT | Must start with `http://` or `https://` |
| `title` | TEXT | Optional label, nullable |
| `created_at` | DATETIME | UTC |
| `expires_at` | DATETIME | UTC; default: now + 30 days; indexed |
| `click_count` | INTEGER | Incremented on each redirect |

Migrations live in `./migrations/` and run automatically at startup via `sqlx::migrate!`.

---

## Errors

All errors return JSON: `{ "error": "message" }`

| Status | Meaning |
|---|---|
| `400` | URL must start with `http://` or `https://` |
| `404` | Token not found |
| `410` | URL has expired |
| `500` | Database error |

---

## Token generation

Tokens are 6 characters drawn from `[A-Za-z0-9]` (62 characters), giving 62⁶ ≈ 56 billion combinations. Generated with `rand::thread_rng` — no sequential IDs, no predictable patterns.

To change the length, use `TokenGenerator::with_length(n)` in `src/token.rs`.
